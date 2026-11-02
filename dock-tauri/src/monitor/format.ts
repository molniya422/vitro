const MB = 1024 ** 2;
const GB = 1024 ** 3;

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 MB";
  if (bytes >= GB) return `${(bytes / GB).toFixed(1)} GB`;
  return `${Math.round(bytes / MB)} MB`;
}

export function formatPercent(value: number): string {
  if (!Number.isFinite(value)) return "0.0%";
  const clamped = Math.min(100, Math.max(0, value));
  return `${clamped.toFixed(1)}%`;
}
