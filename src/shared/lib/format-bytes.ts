/** Форматирует размер в байтах (1024-based, как в проводнике / Vortex). */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  const v = bytes / k ** i;
  const decimals = i === 0 ? 0 : i === 1 ? 1 : 2;
  return `${v.toFixed(decimals)} ${sizes[i]}`;
}
