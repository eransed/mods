export type UserInterfaceColors = {
  background_color: string
  foreground_color: string
  accent_color: string
}

export function applyUserInterfaceColors(colors: UserInterfaceColors): void {
  const root = document.documentElement
  root.style.setProperty('--bg', colors.background_color)
  root.style.setProperty('--fg', colors.foreground_color)
  root.style.setProperty('--accent', colors.accent_color)
}