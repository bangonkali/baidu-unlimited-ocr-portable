#include "gemma/gemma_image_processor.hpp"

#include <algorithm>
#include <cmath>
#include <numeric>
#include <stdexcept>

#if defined(AGUS_OCR_USE_OPENCV_MOBILE) && __has_include(<opencv2/highgui/highgui.hpp>)
#include <opencv2/highgui/highgui.hpp>
#else
#include <opencv2/imgcodecs.hpp>
#endif
#include <opencv2/imgproc.hpp>

#include "gemma/gemma_common.hpp"

namespace agus_ocr {
namespace {

constexpr int kPatchSize = 16;
constexpr int kPoolingKernelSize = 3;
constexpr int kChannels = 3;
constexpr float kRescaleFactor = 1.0f / 255.0f;

cv::Mat DecodeImage(const agus_ocr_image_t& image) {
  std::vector<unsigned char> bytes(image.bytes, image.bytes + image.length);
  cv::Mat decoded = cv::imdecode(bytes, cv::IMREAD_COLOR);
  if (decoded.empty()) {
    throw std::invalid_argument("failed to decode image bytes");
  }
  return decoded;
}

std::vector<float> NormalizeToChw(const cv::Mat& bgr) {
  cv::Mat rgb;
  cv::cvtColor(bgr, rgb, cv::COLOR_BGR2RGB);
  std::vector<float> out(static_cast<size_t>(3 * rgb.rows * rgb.cols));
  const int plane = rgb.rows * rgb.cols;
  const float mean[3] = {0.485f, 0.456f, 0.406f};
  const float stddev[3] = {0.229f, 0.224f, 0.225f};
  for (int y = 0; y < rgb.rows; ++y) {
    const cv::Vec3b* row = rgb.ptr<cv::Vec3b>(y);
    for (int x = 0; x < rgb.cols; ++x) {
      const int offset = y * rgb.cols + x;
      for (int c = 0; c < 3; ++c) {
        out[static_cast<size_t>(c * plane + offset)] =
            (static_cast<float>(row[x][c]) * kRescaleFactor - mean[c]) /
            stddev[c];
      }
    }
  }
  return out;
}

cv::Mat ResizeShortAndCrop(const cv::Mat& image, int short_size,
                           int crop_size) {
  const int h = image.rows;
  const int w = image.cols;
  const float scale =
      static_cast<float>(short_size) / static_cast<float>(std::min(h, w));
  const int new_h = std::max(1, static_cast<int>(std::round(h * scale)));
  const int new_w = std::max(1, static_cast<int>(std::round(w * scale)));
  cv::Mat resized;
  cv::resize(image, resized, cv::Size(new_w, new_h), 0, 0, cv::INTER_LINEAR);
  const int x = std::max(0, (resized.cols - crop_size) / 2);
  const int y = std::max(0, (resized.rows - crop_size) / 2);
  const int width = std::min(crop_size, resized.cols - x);
  const int height = std::min(crop_size, resized.rows - y);
  cv::Mat crop = resized(cv::Rect(x, y, width, height)).clone();
  if (crop.cols != crop_size || crop.rows != crop_size) {
    cv::resize(crop, crop, cv::Size(crop_size, crop_size));
  }
  return crop;
}

cv::Mat RotateImage(const cv::Mat& image, int angle) {
  if (angle == 0) {
    return image.clone();
  }
  const int h = image.rows;
  const int w = image.cols;
  const cv::Point2f center(w / 2.0f, h / 2.0f);
  cv::Mat rot = cv::getRotationMatrix2D(center, angle, 1.0);
  const double abs_cos = std::abs(rot.at<double>(0, 0));
  const double abs_sin = std::abs(rot.at<double>(0, 1));
  const int new_w = static_cast<int>(h * abs_sin + w * abs_cos);
  const int new_h = static_cast<int>(h * abs_cos + w * abs_sin);
  rot.at<double>(0, 2) += (new_w - w) / 2.0;
  rot.at<double>(1, 2) += (new_h - h) / 2.0;
  cv::Mat rotated;
  cv::warpAffine(image, rotated, rot, cv::Size(new_w, new_h),
                 cv::INTER_CUBIC);
  return rotated;
}

std::pair<int, int> AspectRatioPreservingSize(int height, int width,
                                              int max_soft_tokens) {
  const int max_patches =
      max_soft_tokens * kPoolingKernelSize * kPoolingKernelSize;
  const double target_px =
      static_cast<double>(max_patches) * kPatchSize * kPatchSize;
  const double factor =
      std::sqrt(target_px / static_cast<double>(height * width));
  const int side_mult = kPoolingKernelSize * kPatchSize;
  int target_h =
      static_cast<int>(std::floor((factor * height) / side_mult)) * side_mult;
  int target_w =
      static_cast<int>(std::floor((factor * width) / side_mult)) * side_mult;
  if (target_h == 0 && target_w == 0) {
    throw std::runtime_error("Gemma image resize target became 0x0");
  }
  const int max_side =
      (max_patches / (kPoolingKernelSize * kPoolingKernelSize)) * side_mult;
  if (target_h == 0) {
    target_h = side_mult;
    target_w =
        std::min(static_cast<int>(std::floor(static_cast<double>(width) /
                                            std::max(1, height))) *
                     side_mult,
                 max_side);
  } else if (target_w == 0) {
    target_w = side_mult;
    target_h =
        std::min(static_cast<int>(std::floor(static_cast<double>(height) /
                                            std::max(1, width))) *
                     side_mult,
                 max_side);
  }
  return {target_h, target_w};
}

void Patchify(const cv::Mat& rgb, int max_soft_tokens,
              GemmaImageInputs* output) {
  const int max_patches =
      max_soft_tokens * kPoolingKernelSize * kPoolingKernelSize;
  const int patch_dim = kPatchSize * kPatchSize * kChannels;
  const int patches_h = rgb.rows / kPatchSize;
  const int patches_w = rgb.cols / kPatchSize;
  const int patches = patches_h * patches_w;
  output->soft_tokens =
      patches / (kPoolingKernelSize * kPoolingKernelSize);
  output->max_patches = max_patches;
  output->pixel_shape = {1, max_patches, patch_dim};
  output->position_shape = {1, max_patches, 2};
  output->pixel_values.assign(static_cast<size_t>(max_patches * patch_dim),
                              0.0f);
  output->position_ids.assign(static_cast<size_t>(max_patches * 2), -1);

  size_t out = 0;
  for (int ph = 0; ph < patches_h; ++ph) {
    for (int pw = 0; pw < patches_w; ++pw) {
      for (int dy = 0; dy < kPatchSize; ++dy) {
        const cv::Vec3b* row = rgb.ptr<cv::Vec3b>(ph * kPatchSize + dy);
        for (int dx = 0; dx < kPatchSize; ++dx) {
          const cv::Vec3b pixel = row[pw * kPatchSize + dx];
          for (int c = 0; c < kChannels; ++c) {
            output->pixel_values[out++] =
                static_cast<float>(pixel[c]) * kRescaleFactor;
          }
        }
      }
    }
  }

  size_t pos = 0;
  for (int row = 0; row < patches_h; ++row) {
    for (int col = 0; col < patches_w; ++col) {
      output->position_ids[pos++] = col;
      output->position_ids[pos++] = row;
    }
  }
}

void EncodeOverlay(const cv::Mat& image, GemmaImageInputs* output) {
  std::vector<unsigned char> encoded;
  const std::vector<int> jpeg_params = {cv::IMWRITE_JPEG_QUALITY, 90};
  if (cv::imencode(".jpg", image, encoded, jpeg_params)) {
    output->overlay_mime_type = "image/jpeg";
    output->overlay_image_base64 = GemmaBase64Encode(encoded);
    return;
  }
  if (cv::imencode(".png", image, encoded)) {
    output->overlay_mime_type = "image/png";
    output->overlay_image_base64 = GemmaBase64Encode(encoded);
    return;
  }
  throw std::runtime_error("failed to encode Gemma preview image");
}

int Argmax(const std::vector<float>& values) {
  if (values.empty()) {
    return -1;
  }
  return static_cast<int>(
      std::distance(values.begin(),
                    std::max_element(values.begin(), values.end())));
}

}  // namespace

GemmaImageProcessor::GemmaImageProcessor(
    const GemmaMarkdownBundleCheck& bundle,
    agus_ocr_backend_t backend,
    int32_t cpu_threads,
    bool enable_profiling)
    : doc_orientation_(bundle.doc_orientation_model_path, backend, cpu_threads,
                       enable_profiling, "gemma_doc_orientation") {}

int GemmaImageProcessor::DetectOrientation(const cv::Mat& page) const {
  cv::Mat resized = ResizeShortAndCrop(page, 256, 224);
  auto input = NormalizeToChw(resized);
  if (doc_orientation_.inputs().empty()) {
    throw std::runtime_error("Gemma orientation model has no inputs");
  }
  auto outputs = doc_orientation_.Run(
      {GemmaTensor::Float(doc_orientation_.inputs().front().name,
                          {1, 3, 224, 224}, std::move(input))});
  if (outputs.empty()) {
    return -1;
  }
  const int class_id = Argmax(outputs[0].floats);
  static constexpr int kAngles[4] = {0, 90, 180, 270};
  return class_id >= 0 && class_id < 4 ? kAngles[class_id] : -1;
}

GemmaImageInputs GemmaImageProcessor::Process(
    const agus_ocr_image_t& image,
    int32_t visual_token_budget,
    bool use_doc_orientation) const {
  const int soft_token_budget =
      std::max(70, visual_token_budget > 0 ? visual_token_budget : 280);
  GemmaImageInputs output;
  output.source_page = DecodeImage(image);
  output.oriented_page = output.source_page.clone();

  if (use_doc_orientation) {
    const auto start = std::chrono::steady_clock::now();
    output.doc_angle = DetectOrientation(output.source_page);
    if (output.doc_angle >= 0) {
      output.oriented_page = RotateImage(output.source_page, output.doc_angle);
    }
    GemmaLogInfo("core gemma doc orientation angle=" +
                 std::to_string(output.doc_angle) + " elapsedMs=" +
                 std::to_string(GemmaElapsedMs(
                     start, std::chrono::steady_clock::now())));
  }

  cv::Mat rgb;
  cv::cvtColor(output.oriented_page, rgb, cv::COLOR_BGR2RGB);
  const auto target =
      AspectRatioPreservingSize(rgb.rows, rgb.cols, soft_token_budget);
  if (target.first != rgb.rows || target.second != rgb.cols) {
    cv::resize(rgb, rgb, cv::Size(target.second, target.first), 0, 0,
               cv::INTER_CUBIC);
  }
  Patchify(rgb, soft_token_budget, &output);
  EncodeOverlay(output.oriented_page, &output);
  return output;
}

}  // namespace agus_ocr
