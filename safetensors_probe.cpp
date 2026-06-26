#include <algorithm>
#include <cctype>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <map>
#include <string>
#include <string_view>
#include <vector>

namespace {

std::uint64_t read_le64(const unsigned char bytes[8]) {
    std::uint64_t value = 0;
    for (int i = 7; i >= 0; --i) {
        value = (value << 8) | bytes[i];
    }
    return value;
}

std::string parse_json_string(std::string_view text, std::size_t& pos) {
    std::string out;
    if (pos >= text.size() || text[pos] != '"') {
        return out;
    }
    ++pos;
    while (pos < text.size()) {
        const char c = text[pos++];
        if (c == '"') {
            break;
        }
        if (c == '\\' && pos < text.size()) {
            out.push_back(text[pos++]);
        } else {
            out.push_back(c);
        }
    }
    return out;
}

std::vector<std::string> top_level_keys(std::string_view json) {
    std::vector<std::string> keys;
    int depth = 0;
    for (std::size_t pos = 0; pos < json.size();) {
        const char c = json[pos];
        if (c == '"') {
            const int before_depth = depth;
            const std::size_t key_start = pos;
            std::string s = parse_json_string(json, pos);
            std::size_t probe = pos;
            while (probe < json.size() && std::isspace(static_cast<unsigned char>(json[probe]))) {
                ++probe;
            }
            if (before_depth == 1 && probe < json.size() && json[probe] == ':') {
                keys.push_back(std::move(s));
            }
            if (pos == key_start) {
                ++pos;
            }
            continue;
        }
        if (c == '{' || c == '[') {
            ++depth;
        } else if (c == '}' || c == ']') {
            --depth;
        }
        ++pos;
    }
    return keys;
}

std::string category_for(const std::string& key) {
    if (key == "__metadata__") return "__metadata__";
    if (key.rfind("lm_head.", 0) == 0) return "lm_head";
    if (key.rfind("model.embed_tokens.", 0) == 0) return "text.embed_tokens";
    if (key.rfind("model.norm.", 0) == 0) return "text.final_norm";
    if (key.rfind("model.layers.", 0) == 0) {
        if (key.find(".mlp.experts.") != std::string::npos) return "text.moe.routed_experts";
        if (key.find(".mlp.shared_experts.") != std::string::npos) return "text.moe.shared_experts";
        if (key.find(".mlp.gate.") != std::string::npos) return "text.moe.router";
        if (key.find(".mlp.") != std::string::npos) return "text.dense_mlp";
        if (key.find(".self_attn.") != std::string::npos) return "text.attention";
        if (key.find("layernorm") != std::string::npos) return "text.layer_norms";
        return "text.layers.other";
    }
    if (key.rfind("model.sam_model.", 0) == 0) return "vision.sam_vit_b";
    if (key.rfind("model.vision_model.", 0) == 0) return "vision.clip_l";
    if (key.rfind("model.projector.", 0) == 0) return "vision.projector";
    if (key == "model.image_newline" || key == "model.view_seperator") return "vision.special_embeddings";
    return "other";
}

}  // namespace

int main(int argc, char** argv) {
    if (argc != 2) {
        std::cerr << "usage: safetensors_probe <model.safetensors>\n";
        return 2;
    }

    std::ifstream in(argv[1], std::ios::binary | std::ios::ate);
    if (!in) {
        std::cerr << "error: cannot open " << argv[1] << "\n";
        return 1;
    }
    const std::uint64_t file_size = static_cast<std::uint64_t>(in.tellg());
    in.seekg(0, std::ios::beg);

    unsigned char len_bytes[8]{};
    in.read(reinterpret_cast<char*>(len_bytes), sizeof(len_bytes));
    if (!in) {
        std::cerr << "error: cannot read safetensors header length\n";
        return 1;
    }
    const std::uint64_t header_len = read_le64(len_bytes);
    if (header_len > file_size || header_len > (512ULL << 20)) {
        std::cerr << "error: unreasonable safetensors header length: " << header_len << "\n";
        return 1;
    }

    std::string header(static_cast<std::size_t>(header_len), '\0');
    in.read(header.data(), static_cast<std::streamsize>(header.size()));
    if (!in) {
        std::cerr << "error: cannot read safetensors header body\n";
        return 1;
    }

    const auto keys = top_level_keys(header);
    std::map<std::string, std::size_t> categories;
    std::map<std::string, std::size_t> dtype_counts;
    for (const auto& key : keys) {
        ++categories[category_for(key)];
    }

    std::size_t pos = 0;
    while ((pos = header.find("\"dtype\"", pos)) != std::string::npos) {
        std::size_t value_start = header.find('"', pos + 7);
        if (value_start == std::string::npos) break;
        ++value_start;
        std::size_t value_end = header.find('"', value_start);
        if (value_end == std::string::npos) break;
        ++dtype_counts[header.substr(value_start, value_end - value_start)];
        pos = value_end + 1;
    }

    std::cout << "file: " << argv[1] << "\n";
    std::cout << "file_size_bytes: " << file_size << "\n";
    std::cout << "header_bytes: " << header_len << "\n";
    std::cout << "top_level_entries: " << keys.size() << "\n";
    std::cout << "tensor_entries: " << (keys.empty() ? 0 : keys.size() - categories["__metadata__"]) << "\n";

    std::cout << "\ncategory_counts:\n";
    for (const auto& [name, count] : categories) {
        std::cout << "  " << std::left << std::setw(30) << name << " " << count << "\n";
    }

    std::cout << "\ndtype_counts:\n";
    for (const auto& [name, count] : dtype_counts) {
        std::cout << "  " << std::left << std::setw(8) << name << " " << count << "\n";
    }

    return 0;
}
