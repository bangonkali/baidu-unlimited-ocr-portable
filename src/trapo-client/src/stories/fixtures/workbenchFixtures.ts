import type {
  DocumentSummary,
  IngestRunRecord,
  LogRecord,
  ModelsPayload,
  OverlayBox,
  PageTextRecord,
  SettingsPayload,
} from '../../api/types';

export const fixtureDocuments: DocumentSummary[] = [
  {
    display_name: 'invoice-014.png',
    file_hash: 'hash-invoice-014',
    current_page: 1,
    page_count: 1,
    processed_pages: 1,
    progress_percent: 100,
    regions: 2,
    relative_path: 'finance/invoice-014.png',
    status: 'completed',
    total_pages: 1,
  },
  {
    display_name: 'Sample 0001.pdf',
    file_hash: 'hash-shipping-form',
    current_page: 2,
    page_count: 3,
    processed_pages: 1,
    progress_percent: 33.3,
    regions: 8,
    relative_path: 'dataset/Sample 0001.pdf',
    status: 'running',
    total_pages: 3,
  },
];

export const fixtureBoxes: OverlayBox[] = [
  {
    height_percent: 8,
    label: 'Invoice total',
    content_markdown: 'Invoice total: 1,240.00',
    left_percent: 18,
    page_no: 1,
    region_id: 'reg-total',
    top_percent: 34,
    width_percent: 26,
  },
  {
    height_percent: 7,
    label: 'Supplier',
    content_markdown: 'Supplier',
    left_percent: 16,
    page_no: 1,
    region_id: 'reg-supplier',
    top_percent: 18,
    width_percent: 30,
  },
];

export const fixturePages: PageTextRecord[] = [
  {
    page_no: 1,
    spans: [
      { end: 8, page_no: 1, region_id: 'reg-supplier', start: 0 },
      { end: 31, page_no: 1, region_id: 'reg-total', start: 17 },
    ],
    text: 'Supplier\n\nInvoice total: 1,240.00',
  },
];

export const fixtureRuns: IngestRunRecord[] = [
  {
    current_page: 8,
    progress_percent: 16.3,
    processed_pages: 7,
    queued_files: 18,
    root_path: 'C:\\data\\incoming',
    run_id: 'run-20260629-01',
    status: 'running',
    total_pages: 43,
  },
];

export const fixtureLogs: LogRecord[] = [
  {
    component: 'server',
    level: 'INFO',
    message: 'listening http://127.0.0.1:8765/',
    timestamp: '2026-06-29T04:10:00Z',
  },
  {
    component: 'pdf',
    level: 'INFO',
    message: 'rendering dataset/Sample 0001.pdf at 200 DPI with MuPDF',
    timestamp: '2026-06-29T04:10:12Z',
  },
];

export const fixtureModels: ModelsPayload = {
  provider_label: 'Sahil Chachra Unlimited-OCR GGUF',
  provider_repo: 'sahilchachra/Unlimited-OCR-GGUF',
  selected_model_id: 'unlimited-ocr-q4-k-m',
  shared_mmproj_file: 'mmproj-Unlimited-OCR-F16.gguf',
  models: [
    {
      auth_available: true,
      auth_source: 'HF_TOKEN',
      bits: 4,
      display_name: 'Unlimited-OCR Q4_K_M',
      downloaded_bytes: 4_900_000_000,
      downloaded_file_count: 2,
      files: [
        {
          downloaded_bytes: 4_000_000_000,
          file_id: 'model',
          file_name: 'Unlimited-OCR-Q4_K_M.gguf',
          percent: 100,
          status: 'downloaded',
          total_bytes: 4_000_000_000,
        },
        {
          downloaded_bytes: 900_000_000,
          file_id: 'mmproj',
          file_name: 'mmproj-Unlimited-OCR-F16.gguf',
          percent: 100,
          status: 'downloaded',
          total_bytes: 900_000_000,
        },
      ],
      hardware_tier: 'Most CUDA GPUs',
      model_id: 'unlimited-ocr-q4-k-m',
      notes: 'Default practical size and quality choice.',
      overall_downloaded_bytes: 4_900_000_000,
      overall_percent: 100,
      overall_total_bytes: 4_900_000_000,
      quality: 'Recommended balance',
      quantization: 'Q4_K_M',
      recommended: true,
      repo_id: 'sahilchachra/Unlimited-OCR-GGUF',
      revision: 'main',
      selected: true,
      status: 'downloaded',
      total_file_count: 2,
      total_required_bytes: 2_762_203_232,
      total_bytes: 4_900_000_000,
    },
    {
      auth_available: true,
      auth_source: 'HF_TOKEN',
      bits: 2,
      display_name: 'Unlimited-OCR IQ2_M',
      downloaded_bytes: 811_876_448,
      downloaded_file_count: 1,
      files: [
        {
          downloaded_bytes: 0,
          file_id: 'model',
          file_name: 'Unlimited-OCR-IQ2_M.gguf',
          percent: 0,
          status: 'missing',
          total_bytes: 1_232_148_224,
        },
        {
          downloaded_bytes: 811_876_448,
          file_id: 'mmproj',
          file_name: 'mmproj-Unlimited-OCR-F16.gguf',
          percent: 100,
          status: 'downloaded',
          total_bytes: 811_876_448,
        },
      ],
      hardware_tier: 'Very tight memory',
      model_id: 'unlimited-ocr-iq2-m',
      notes: 'Smallest option; quality tradeoffs are expected.',
      overall_downloaded_bytes: 811_876_448,
      overall_percent: 39.7,
      overall_total_bytes: 2_044_024_672,
      quality: 'Smallest experimental',
      quantization: 'IQ2_M',
      repo_id: 'sahilchachra/Unlimited-OCR-GGUF',
      revision: 'main',
      selected: false,
      status: 'missing',
      total_file_count: 2,
      total_required_bytes: 2_044_024_672,
    },
  ],
  profiles: [
    {
      default_max_tokens: 8192,
      description: 'Higher avg similarity diagnostic profile.',
      engine_name: 'llamacpp-q4_k_m-uocr-rswa-noimgend-noeos-full',
      key: 'experimental-exact-prefill-q4',
      label: 'Experimental exact-prefill Q4',
    },
    {
      default_max_tokens: 8192,
      description: 'Zero-empty fallback profile.',
      engine_name: 'llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full',
      key: 'best-zero-empty-q4',
      label: 'Practical zero-empty Q4',
    },
  ],
};

const downloadedFixtureModel = fixtureModels.models[0] ?? {
  display_name: 'Unlimited-OCR Q4_K_M',
  model_id: 'unlimited-ocr-q4-k-m',
  status: 'missing',
};

export const fixtureDownloadingModels: ModelsPayload = {
  ...fixtureModels,
  models: [
    {
      ...downloadedFixtureModel,
      bytes_per_second: 11_800_000,
      current_file: 'Unlimited-OCR-Q4_K_M.gguf',
      downloaded_bytes: 1_800_000_000,
      eta_seconds: 263,
      files: [
        {
          bytes_per_second: 11_800_000,
          downloaded_bytes: 1_800_000_000,
          eta_seconds: 186,
          file_id: 'model',
          file_name: 'Unlimited-OCR-Q4_K_M.gguf',
          percent: 45,
          status: 'downloading',
          total_bytes: 4_000_000_000,
        },
        {
          downloaded_bytes: 0,
          file_id: 'mmproj',
          file_name: 'mmproj-Unlimited-OCR-F16.gguf',
          percent: 0,
          status: 'missing',
          total_bytes: 900_000_000,
        },
      ],
      overall_downloaded_bytes: 1_800_000_000,
      overall_percent: 36.7,
      status: 'downloading',
      status_message: 'Downloading Unlimited-OCR-Q4_K_M.gguf',
    },
  ],
};

export const fixtureSettings: SettingsPayload = {
  cache_path: 'C:\\uocr\\cache',
  database_path: 'C:\\uocr\\data\\uocr.duckdb',
  default_profile: 'experimental-exact-prefill-q4',
  ocr_concurrency: 1,
  pdf_dpi: 200,
  retry_profile: 'best-zero-empty-q4',
  runtime_variants: [
    {
      accelerator: 'cuda',
      backend: 'cuda',
      hardware_supported: true,
      installed: true,
      label: 'Windows x64 CUDA 13',
      platform: 'windows-x86_64-cuda13',
      runtime_id: 'windows-x86_64-cuda13',
      selectable: true,
      selected: true,
      support_detail: 'NVIDIA CUDA probe found',
    },
    {
      accelerator: 'rocm',
      backend: 'rocm',
      hardware_supported: false,
      installed: false,
      label: 'Windows x64 AMD ROCm/HIP',
      platform: 'windows-x86_64-rocm6',
      runtime_id: 'windows-x86_64-rocm6',
      selectable: false,
      selected: false,
      support_detail: 'rocminfo/hipinfo was not found',
    },
    {
      accelerator: 'cpu',
      backend: 'cpu',
      hardware_supported: true,
      installed: true,
      label: 'Windows x64 CPU',
      platform: 'windows-x86_64-cpu',
      runtime_id: 'windows-x86_64-cpu',
      selectable: true,
      selected: false,
      support_detail: 'CPU inference is available',
    },
  ],
  selected_accelerator: 'cuda',
  selected_model_id: 'unlimited-ocr-q4-k-m',
  selected_runtime_id: 'windows-x86_64-cuda13',
};
