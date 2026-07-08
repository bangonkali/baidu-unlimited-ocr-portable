#ifndef AGUS_OCR_H_
#define AGUS_OCR_H_

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32)
#define AGUS_OCR_EXPORT __declspec(dllexport)
#else
#define AGUS_OCR_EXPORT __attribute__((visibility("default")))
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct agus_ocr_engine_t agus_ocr_engine_t;

typedef enum agus_ocr_status_t {
  AGUS_OCR_OK = 0,
  AGUS_OCR_UNAVAILABLE = 1,
  AGUS_OCR_INVALID_ARGUMENT = 2,
  AGUS_OCR_INTERNAL_ERROR = 3
} agus_ocr_status_t;

typedef enum agus_ocr_backend_t {
  AGUS_OCR_BACKEND_AUTO = 0,
  AGUS_OCR_BACKEND_CPU = 1,
  AGUS_OCR_BACKEND_XNNPACK = 2,
  AGUS_OCR_BACKEND_COREML = 3,
  AGUS_OCR_BACKEND_DIRECTML = 4,
  AGUS_OCR_BACKEND_WEBASSEMBLY = 5,
  AGUS_OCR_BACKEND_WEBGPU = 6,
  AGUS_OCR_BACKEND_NNAPI = 7,
  AGUS_OCR_BACKEND_QNN = 8,
  AGUS_OCR_BACKEND_CUDA = 9
} agus_ocr_backend_t;

typedef enum agus_ocr_pipeline_t {
  AGUS_OCR_PIPELINE_PPOCRV6 = 0,
  AGUS_OCR_PIPELINE_PADDLEOCR_VL16 = 1,
  AGUS_OCR_PIPELINE_GEMMA_MARKDOWN = 2
} agus_ocr_pipeline_t;

typedef enum agus_ocr_generative_backend_t {
  AGUS_OCR_GEN_BACKEND_AUTO = 0,
  AGUS_OCR_GEN_BACKEND_CPU = 1,
  AGUS_OCR_GEN_BACKEND_VULKAN = 2,
  AGUS_OCR_GEN_BACKEND_CUDA = 3,
  AGUS_OCR_GEN_BACKEND_OPENCL = 4
} agus_ocr_generative_backend_t;

typedef struct agus_ocr_runtime_options_t {
  size_t struct_size;
  agus_ocr_backend_t backend;
  int32_t cpu_threads;
  int32_t enable_ort_profiling;
  agus_ocr_generative_backend_t generative_backend;
  int32_t generative_gpu_layers;
  int32_t force_cpu_only;
} agus_ocr_runtime_options_t;

typedef struct agus_ocr_run_options_t {
  size_t struct_size;
  int32_t use_doc_orientation;
  int32_t use_doc_unwarping;
  int32_t use_textline_orientation;
  int32_t text_detection_limit_side_len;
  const char* text_detection_limit_type;
  float text_detection_threshold;
  float text_detection_box_threshold;
  float text_detection_unclip_ratio;
  float text_recognition_score_threshold;
  int32_t enable_source_box_estimation;
  int32_t generate_markdown;
  int32_t max_new_tokens;
  float temperature;
  int32_t min_pixels;
  int32_t max_pixels;
  const char* markdown_prompt;
  int32_t visual_token_budget;
} agus_ocr_run_options_t;

typedef struct agus_ocr_init_options_t {
  size_t struct_size;
  agus_ocr_pipeline_t pipeline;
  const char* model_root;
  const char* external_model_root;
  const char* vl_model_path;
  const char* vl_mmproj_path;
  agus_ocr_runtime_options_t runtime;
  agus_ocr_run_options_t defaults;
} agus_ocr_init_options_t;

typedef struct agus_ocr_image_t {
  size_t struct_size;
  const uint8_t* bytes;
  size_t length;
  const char* mime_type;
} agus_ocr_image_t;

typedef struct agus_ocr_result_t {
  size_t struct_size;
  char* json;
  size_t json_length;
} agus_ocr_result_t;

AGUS_OCR_EXPORT agus_ocr_status_t
agus_ocr_create(const agus_ocr_init_options_t* options,
                agus_ocr_engine_t** out_engine);

AGUS_OCR_EXPORT agus_ocr_status_t
agus_ocr_get_runtime_capabilities(agus_ocr_result_t** out_result);

AGUS_OCR_EXPORT agus_ocr_status_t
agus_ocr_recognize_image(agus_ocr_engine_t* engine,
                         const agus_ocr_image_t* image,
                         const agus_ocr_run_options_t* options,
                         agus_ocr_result_t** out_result);

AGUS_OCR_EXPORT void agus_ocr_free_result(agus_ocr_result_t* result);

AGUS_OCR_EXPORT void agus_ocr_destroy(agus_ocr_engine_t* engine);

AGUS_OCR_EXPORT const char* agus_ocr_last_error(void);

AGUS_OCR_EXPORT const char* agus_ocr_engine_runtime_summary(
    agus_ocr_engine_t* engine);

#ifdef __cplusplus
}  // extern "C"
#endif

#endif  // AGUS_OCR_H_
