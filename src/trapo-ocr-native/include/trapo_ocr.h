#ifndef TRAPO_OCR_H_
#define TRAPO_OCR_H_

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32)
#define TRAPO_OCR_EXPORT __declspec(dllexport)
#else
#define TRAPO_OCR_EXPORT __attribute__((visibility("default")))
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct trapo_ocr_engine_t trapo_ocr_engine_t;

typedef enum trapo_ocr_status_t {
  TRAPO_OCR_OK = 0,
  TRAPO_OCR_UNAVAILABLE = 1,
  TRAPO_OCR_INVALID_ARGUMENT = 2,
  TRAPO_OCR_INTERNAL_ERROR = 3
} trapo_ocr_status_t;

typedef enum trapo_ocr_backend_t {
  TRAPO_OCR_BACKEND_AUTO = 0,
  TRAPO_OCR_BACKEND_CPU = 1,
  TRAPO_OCR_BACKEND_XNNPACK = 2,
  TRAPO_OCR_BACKEND_COREML = 3,
  TRAPO_OCR_BACKEND_DIRECTML = 4,
  TRAPO_OCR_BACKEND_WEBASSEMBLY = 5,
  TRAPO_OCR_BACKEND_WEBGPU = 6,
  TRAPO_OCR_BACKEND_NNAPI = 7,
  TRAPO_OCR_BACKEND_QNN = 8,
  TRAPO_OCR_BACKEND_CUDA = 9
} trapo_ocr_backend_t;

typedef enum trapo_ocr_pipeline_t {
  TRAPO_OCR_PIPELINE_PPOCRV6 = 0,
  TRAPO_OCR_PIPELINE_PADDLEOCR_VL16 = 1,
  TRAPO_OCR_PIPELINE_DOCUMENT_MARKDOWN = 2
} trapo_ocr_pipeline_t;

typedef enum trapo_ocr_generative_backend_t {
  TRAPO_OCR_GEN_BACKEND_AUTO = 0,
  TRAPO_OCR_GEN_BACKEND_CPU = 1,
  TRAPO_OCR_GEN_BACKEND_VULKAN = 2,
  TRAPO_OCR_GEN_BACKEND_CUDA = 3,
  TRAPO_OCR_GEN_BACKEND_OPENCL = 4
} trapo_ocr_generative_backend_t;

typedef struct trapo_ocr_runtime_options_t {
  size_t struct_size;
  trapo_ocr_backend_t backend;
  int32_t cpu_threads;
  int32_t enable_ort_profiling;
  trapo_ocr_generative_backend_t generative_backend;
  int32_t generative_gpu_layers;
  int32_t force_cpu_only;
} trapo_ocr_runtime_options_t;

typedef struct trapo_ocr_run_options_t {
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
} trapo_ocr_run_options_t;

typedef struct trapo_ocr_init_options_t {
  size_t struct_size;
  trapo_ocr_pipeline_t pipeline;
  const char* model_root;
  const char* external_model_root;
  const char* vl_model_path;
  const char* vl_mmproj_path;
  trapo_ocr_runtime_options_t runtime;
  trapo_ocr_run_options_t defaults;
} trapo_ocr_init_options_t;

typedef struct trapo_ocr_image_t {
  size_t struct_size;
  const uint8_t* bytes;
  size_t length;
  const char* mime_type;
} trapo_ocr_image_t;

typedef struct trapo_ocr_result_t {
  size_t struct_size;
  char* json;
  size_t json_length;
} trapo_ocr_result_t;

TRAPO_OCR_EXPORT trapo_ocr_status_t
trapo_ocr_create(const trapo_ocr_init_options_t* options,
                trapo_ocr_engine_t** out_engine);

TRAPO_OCR_EXPORT trapo_ocr_status_t
trapo_ocr_get_runtime_capabilities(trapo_ocr_result_t** out_result);

TRAPO_OCR_EXPORT trapo_ocr_status_t
trapo_ocr_recognize_image(trapo_ocr_engine_t* engine,
                         const trapo_ocr_image_t* image,
                         const trapo_ocr_run_options_t* options,
                         trapo_ocr_result_t** out_result);

TRAPO_OCR_EXPORT void trapo_ocr_free_result(trapo_ocr_result_t* result);

TRAPO_OCR_EXPORT void trapo_ocr_destroy(trapo_ocr_engine_t* engine);

TRAPO_OCR_EXPORT const char* trapo_ocr_last_error(void);

TRAPO_OCR_EXPORT const char* trapo_ocr_engine_runtime_summary(
    trapo_ocr_engine_t* engine);

#ifdef __cplusplus
}  // extern "C"
#endif

#endif  // TRAPO_OCR_H_
