#include "document_markdown/document_markdown_engine.hpp"

#include <algorithm>
#include <chrono>
#include <cmath>
#include <numeric>
#include <sstream>
#include <stdexcept>

#include "document_markdown/document_markdown_common.hpp"

namespace trapo_ocr {
namespace {

constexpr int64_t kImageTokenId = 258880;
constexpr int64_t kHiddenSize = 2560;
constexpr int32_t kDefaultMaxNewTokens = 1024;

bool StartsWith(const std::string& value, const std::string& prefix) {
  return value.rfind(prefix, 0) == 0;
}

const DocumentMarkdownTensor& FindOutput(const std::vector<DocumentMarkdownTensor>& outputs,
                              const std::string& name,
                              size_t fallback) {
  for (const DocumentMarkdownTensor& output : outputs) {
    if (output.name == name) {
      return output;
    }
  }
  if (fallback < outputs.size()) {
    return outputs[fallback];
  }
  throw std::runtime_error("Document Markdown ONNX output missing: " + name);
}

std::string RuntimeSummary(trapo_ocr_backend_t backend, int32_t cpu_threads) {
  std::ostringstream out;
  out << "document-markdown onnxruntime ";
  if (backend == TRAPO_OCR_BACKEND_DIRECTML) {
    out << "directml";
  } else if (backend == TRAPO_OCR_BACKEND_CUDA) {
    out << "cuda";
  } else {
    out << "cpu";
  }
  out << " opencv";
  if (backend == TRAPO_OCR_BACKEND_DIRECTML) {
    out << " vision=cpu decoder=directml";
  } else if (backend == TRAPO_OCR_BACKEND_CUDA) {
    out << " vision=cuda decoder=cuda";
  }
  if (cpu_threads > 0) {
    out << " threads=" << cpu_threads;
  }
  return out.str();
}

trapo_ocr_backend_t DocumentMarkdownVisionBackend(trapo_ocr_backend_t backend) {
  return backend == TRAPO_OCR_BACKEND_DIRECTML ? TRAPO_OCR_BACKEND_CPU : backend;
}

struct KvCache {
  std::string input_name;
  std::vector<int64_t> shape;
  std::vector<float> values;
};

std::vector<KvCache> InitialKvCache(const DocumentMarkdownOnnxSession& decoder) {
  std::vector<KvCache> caches;
  for (const DocumentMarkdownInputInfo& input : decoder.inputs()) {
    if (!StartsWith(input.name, "past_key_values.")) {
      continue;
    }
    if (input.shape.size() != 4 || input.shape[1] <= 0 ||
        input.shape[3] <= 0) {
      throw std::runtime_error("unexpected DocumentMarkdown KV-cache input shape");
    }
    KvCache cache;
    cache.input_name = input.name;
    cache.shape = {1, input.shape[1], 0, input.shape[3]};
    caches.push_back(std::move(cache));
  }
  if (caches.empty()) {
    throw std::runtime_error("Document Markdown decoder exposes no KV-cache inputs");
  }
  return caches;
}

}  // namespace

DocumentMarkdownEngine::DocumentMarkdownEngine(
    const DocumentMarkdownBundleCheck& bundle,
    const trapo_ocr_runtime_options_t& runtime,
    trapo_ocr_backend_t active_backend)
    : active_backend_(active_backend),
      cpu_threads_(runtime.cpu_threads),
      enable_profiling_(runtime.enable_ort_profiling != 0),
      tokenizer_(bundle.tokenizer_path),
      vision_backend_(DocumentMarkdownVisionBackend(active_backend_)),
      image_processor_(bundle, vision_backend_, cpu_threads_, enable_profiling_),
      vision_session_(bundle.vision_model_path, vision_backend_, cpu_threads_,
                      enable_profiling_, "document_markdown_vision"),
      embed_session_(bundle.embed_model_path, active_backend_, cpu_threads_,
                     enable_profiling_, "document_markdown_embed"),
      decoder_session_(bundle.decoder_model_path, active_backend_, cpu_threads_,
                       enable_profiling_, "document_markdown_decoder"),
      random_(std::random_device{}()) {}

DocumentMarkdownEngine::~DocumentMarkdownEngine() = default;

std::string DocumentMarkdownEngine::runtime_summary() const {
  return RuntimeSummary(active_backend_, cpu_threads_);
}

std::string DocumentMarkdownEngine::BuildPrompt(
    int image_tokens,
    const std::string& markdown_prompt) const {
  std::ostringstream out;
  out << "<bos><|turn>user\n\n\n<|image>";
  for (int i = 0; i < image_tokens; ++i) {
    out << "<|image|>";
  }
  out << "<image|>\n\n" << markdown_prompt << "<turn|>\n<|turn>model\n";
  return out.str();
}

DocumentMarkdownEngine::GenerationResult DocumentMarkdownEngine::Generate(
    const DocumentMarkdownImageInputs& image,
    const trapo_ocr_run_options_t& options) {
  const auto generation_start = std::chrono::steady_clock::now();
  GenerationResult result;
  const int32_t max_new_tokens =
      options.max_new_tokens > 0 ? options.max_new_tokens : kDefaultMaxNewTokens;
  const float temperature = options.temperature;

  const auto vision_start = std::chrono::steady_clock::now();
  auto vision_outputs = vision_session_.Run({
      DocumentMarkdownTensor::Float("pixel_values", image.pixel_shape, image.pixel_values),
      DocumentMarkdownTensor::Int64("pixel_position_ids", image.position_shape,
                         image.position_ids),
  });
  const DocumentMarkdownTensor& image_features =
      FindOutput(vision_outputs, "image_features", 0);
  result.vision_ms =
      DocumentMarkdownElapsedMs(vision_start, std::chrono::steady_clock::now());
  if (image_features.floats.empty() ||
      image_features.floats.size() % kHiddenSize != 0) {
    std::ostringstream message;
    message << "Document Markdown vision output size is invalid: floats="
            << image_features.floats.size() << " hiddenSize=" << kHiddenSize;
    throw std::runtime_error(message.str());
  }
  result.vision_tokens =
      static_cast<int32_t>(image_features.floats.size() / kHiddenSize);
  if (result.vision_tokens <= 0) {
    throw std::runtime_error("Document Markdown vision encoder returned no image tokens");
  }

  const std::string prompt = BuildPrompt(
      result.vision_tokens,
      options.markdown_prompt != nullptr && options.markdown_prompt[0] != '\0'
          ? options.markdown_prompt
          : "Convert the document image to clean Markdown. Output only Markdown.");
  std::vector<int64_t> input_ids = tokenizer_.Encode(prompt);
  std::vector<int64_t> attention_mask(input_ids.size(), 1);
  std::vector<int64_t> position_ids(input_ids.size());
  std::iota(position_ids.begin(), position_ids.end(), int64_t{0});
  std::vector<int64_t> generated;
  std::vector<KvCache> kv_cache = InitialKvCache(decoder_session_);
  result.prompt_tokens = static_cast<int32_t>(input_ids.size());

  for (int32_t step = 0; step < max_new_tokens; ++step) {
    const int64_t seq_len = static_cast<int64_t>(input_ids.size());
    auto embed_outputs = embed_session_.Run({DocumentMarkdownTensor::Int64(
        "input_ids", {1, seq_len}, input_ids)});
    DocumentMarkdownTensor inputs_embeds = FindOutput(embed_outputs, "inputs_embeds", 0);
    DocumentMarkdownTensor per_layer_inputs =
        FindOutput(embed_outputs, "per_layer_inputs", 1);

    if (step == 0) {
      int image_feature_index = 0;
      for (size_t token_index = 0; token_index < input_ids.size();
           ++token_index) {
        if (input_ids[token_index] != kImageTokenId) {
          continue;
        }
        const size_t embed_offset = token_index * kHiddenSize;
        const size_t image_offset =
            static_cast<size_t>(image_feature_index) * kHiddenSize;
        if (image_offset + kHiddenSize > image_features.floats.size()) {
          std::ostringstream message;
          message << "DocumentMarkdown prompt has more image tokens than vision features: "
                  << "promptImageTokens=" << (image_feature_index + 1)
                  << " visionTokens=" << result.vision_tokens
                  << " processorSoftTokens=" << image.soft_tokens;
          throw std::runtime_error(message.str());
        }
        std::copy(image_features.floats.begin() + image_offset,
                  image_features.floats.begin() + image_offset + kHiddenSize,
                  inputs_embeds.floats.begin() + embed_offset);
        ++image_feature_index;
      }
      if (image_feature_index != result.vision_tokens) {
        std::ostringstream message;
        message << "DocumentMarkdown prompt image-token count did not match vision features: "
                << "promptImageTokens=" << image_feature_index
                << " visionTokens=" << result.vision_tokens
                << " processorSoftTokens=" << image.soft_tokens;
        throw std::runtime_error(message.str());
      }
    }

    std::vector<DocumentMarkdownTensor> decoder_inputs;
    decoder_inputs.push_back(DocumentMarkdownTensor::Float(
        "inputs_embeds", std::move(inputs_embeds.shape),
        std::move(inputs_embeds.floats)));
    decoder_inputs.push_back(DocumentMarkdownTensor::Int64(
        "attention_mask", {1, static_cast<int64_t>(attention_mask.size())},
        attention_mask));
    decoder_inputs.push_back(DocumentMarkdownTensor::Int64(
        "position_ids", {1, static_cast<int64_t>(position_ids.size())},
        position_ids));
    decoder_inputs.push_back(
        DocumentMarkdownTensor::Int64("num_logits_to_keep", {}, {1}));
    decoder_inputs.push_back(DocumentMarkdownTensor::Float(
        "per_layer_inputs",
        std::move(per_layer_inputs.shape),
        std::move(per_layer_inputs.floats)));
    for (const KvCache& cache : kv_cache) {
      decoder_inputs.push_back(
          DocumentMarkdownTensor::Float(cache.input_name, cache.shape, cache.values));
    }

    auto decoder_outputs = decoder_session_.Run(decoder_inputs);
    const DocumentMarkdownTensor& logits = FindOutput(decoder_outputs, "logits", 0);
    const int64_t next_token = SelectNextToken(logits.floats, temperature);
    generated.push_back(next_token);

    if (decoder_outputs.size() < kv_cache.size() + 1) {
      throw std::runtime_error("Document Markdown decoder returned too few KV outputs");
    }
    for (size_t i = 0; i < kv_cache.size(); ++i) {
      const DocumentMarkdownTensor& present = decoder_outputs[i + 1];
      kv_cache[i].shape = present.shape;
      kv_cache[i].values = present.floats;
    }

    if (tokenizer_.IsEos(next_token)) {
      break;
    }
    input_ids = {next_token};
    attention_mask.push_back(1);
    position_ids = {position_ids.empty() ? 0 : position_ids.back() + 1};
  }

  result.generated_tokens = static_cast<int32_t>(generated.size());
  result.markdown = tokenizer_.Decode(generated, true);
  result.generation_ms =
      DocumentMarkdownElapsedMs(generation_start, std::chrono::steady_clock::now());
  return result;
}

}  // namespace trapo_ocr
