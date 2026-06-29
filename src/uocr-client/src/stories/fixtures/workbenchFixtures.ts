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
    page_count: 1,
    regions: 2,
    relative_path: 'finance/invoice-014.png',
    status: 'completed',
  },
  {
    display_name: 'Sample 0001.pdf',
    file_hash: 'hash-shipping-form',
    page_count: 3,
    regions: 8,
    relative_path: 'dataset/Sample 0001.pdf',
    status: 'running',
  },
];

export const fixtureBoxes: OverlayBox[] = [
  {
    height_percent: 8,
    label: 'Invoice total',
    left_percent: 18,
    page_no: 1,
    region_id: 'reg-total',
    top_percent: 34,
    width_percent: 26,
  },
  {
    height_percent: 7,
    label: 'Supplier',
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
  models: [
    {
      display_name: 'Unlimited-OCR Q4_K_M',
      downloaded_bytes: 100,
      model_id: 'unlimited-ocr-q4-k-m',
      status: 'downloaded',
      total_bytes: 100,
    },
  ],
  profiles: [
    {
      default_max_tokens: 8192,
      description: 'Current R-SWA Q4 demo default.',
      engine_name: 'llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full',
      key: 'best-zero-empty-q4',
      label: 'Practical zero-empty Q4',
    },
  ],
};

export const fixtureSettings: SettingsPayload = {
  default_profile: 'best-zero-empty-q4',
  ocr_concurrency: 1,
  pdf_dpi: 200,
  retry_profile: 'experimental-exact-prefill-q4',
};
