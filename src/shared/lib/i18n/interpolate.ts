/** Подстановка `{name}` в строке. */
export function interpolate(
  template: string,
  params?: Record<string, string | number>,
): string {
  if (!params) {
    return template;
  }
  return template.replace(/\{(\w+)\}/g, (_, key: string) => {
    const v = params[key];
    return v !== undefined ? String(v) : `{${key}}`;
  });
}
