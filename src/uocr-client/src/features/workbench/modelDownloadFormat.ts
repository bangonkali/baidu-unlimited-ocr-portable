export function formatBytes(bytes?: number | null) {
  if (!bytes || bytes <= 0) {
    return '0 B';
  }
  const units = ['B', 'KiB', 'MiB', 'GiB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

export function formatRate(bytesPerSecond?: number | null) {
  if (!bytesPerSecond || bytesPerSecond <= 0) {
    return '0 MiB/s';
  }
  return `${(bytesPerSecond / 1024 / 1024).toFixed(2)} MiB/s`;
}

export function formatEta(seconds?: number | null) {
  if (seconds === null || seconds === undefined || seconds < 0) {
    return 'Unknown';
  }
  if (seconds < 60) {
    return `${Math.ceil(seconds)}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remaining = Math.ceil(seconds % 60);
  return `${minutes}m ${remaining}s`;
}

export function formatPercent(value?: number | null) {
  return `${Math.max(0, Math.min(100, value ?? 0)).toFixed(1)}%`;
}
