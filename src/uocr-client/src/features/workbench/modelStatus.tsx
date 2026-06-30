import { CheckCircle2, CircleDot } from 'lucide-react';

export function statusIcon(status: string) {
  return status === 'downloaded' ? <CheckCircle2 size={12} /> : <CircleDot size={12} />;
}

export function statusText(status: string) {
  if (status === 'downloaded') {
    return 'Model files are present. Scans can start.';
  }
  if (status === 'downloading') {
    return 'Downloading model assets from Hugging Face.';
  }
  if (status === 'error') {
    return 'Download failed. Check Diagnostics for the detailed error and retry.';
  }
  if (status === 'cancelled') {
    return 'Download was cancelled. Retry will resume partial files when possible.';
  }
  return 'Download the model files before starting OCR.';
}
