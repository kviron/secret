/**
 * Steam store / библиотека: стандартный горизонтальный header (≈460×215, соотношение 460:215).
 * @see https://partner.steamgames.com/doc/store/assets/standardassets
 */
export function steamHeaderImageUrl(steamAppId: number): string {
  return `https://cdn.akamai.steamstatic.com/steam/apps/${steamAppId}/header.jpg`;
}

/** Вертикальная капсула библиотеки 600×900 — для других экранов, не для широкой карточки. */
export function steamLibraryCapsuleUrl(steamAppId: number): string {
  return `https://cdn.akamai.steamstatic.com/steam/apps/${steamAppId}/library_600x900.jpg`;
}
