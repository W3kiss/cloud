// Global app settings: persisted in localStorage and applied to <html>.

export type ThemeId =
  | "mono"
  | "cyan"
  | "violet"
  | "emerald"
  | "rose"
  | "amber"
  | "graphite"
  | "midnight"
  | "forest"
  | "wine"
  | "amoled"
  | "light";

/** A theme is a complete token palette, not just an accent colour. */
export interface ThemeDef {
  id: ThemeId;
  label: string;
  hint: string;
  accent: string;
  accent2: string;
  violet: string;
  /** Page background and soft (inset) background. */
  bg: string;
  bgSoft: string;
  /** Panel/surface base colour as an "r g b" triplet. */
  panel: string;
  /** Foreground-tint colour ("r g b") used for borders and translucent fills. */
  fg: string;
  text: string;
  muted: string;
  mutedStrong: string;
  scheme: "dark" | "light";
  /** Window tint (RGBA) behind the web content. */
  tint: [number, number, number, number];
}

const DARK = {
  fg: "255 255 255",
  text: "#f5f5f6",
  muted: "#97979f",
  mutedStrong: "#c8c8ce",
  scheme: "dark" as const,
};

function dark(
  id: ThemeId,
  label: string,
  hint: string,
  accent: string,
  accent2: string,
  violet: string,
  bg: string,
  bgSoft: string,
  panel: string,
): ThemeDef {
  const [r, g, b] = panel.split(" ").map(Number);
  return {
    id, label, hint, accent, accent2, violet, bg, bgSoft, panel, ...DARK,
    tint: [r, g, b, 230],
  };
}

export const THEMES: ThemeDef[] = [
  dark("mono", "Моно", "Классическая чёрно-белая", "#f5f5f6", "#a1a1aa", "#d4d4d8", "#08080a", "#0c0c0f", "10 10 13"),
  dark("cyan", "Бирюза", "Холодный неон", "#22d3ee", "#38bdf8", "#a5f3fc", "#060b0e", "#0a1216", "8 14 18"),
  dark("violet", "Фиолет", "Мягкий лавандовый", "#a78bfa", "#818cf8", "#ddd6fe", "#0a0813", "#100d1c", "13 10 24"),
  dark("emerald", "Изумруд", "Свежий зелёный", "#34d399", "#2dd4bf", "#a7f3d0", "#060d0a", "#0a1511", "8 17 13"),
  dark("rose", "Роза", "Тёплый розовый", "#fb7185", "#f472b6", "#fecdd3", "#12080b", "#1a0d11", "22 10 14"),
  dark("amber", "Янтарь", "Золотистый акцент", "#fbbf24", "#fb923c", "#fde68a", "#100c05", "#18120a", "20 15 8"),
  dark("graphite", "Графит", "Нейтральный серый фон", "#e4e4e7", "#a1a1aa", "#d4d4d8", "#17181b", "#1d1e22", "26 27 31"),
  dark("midnight", "Полночь", "Глубокий синий фон", "#60a5fa", "#818cf8", "#bfdbfe", "#070b17", "#0b1124", "9 14 30"),
  dark("forest", "Лес", "Тёмно-зелёный фон", "#4ade80", "#a3e635", "#bbf7d0", "#08110b", "#0d1a12", "10 22 15"),
  dark("wine", "Вино", "Бордовый фон", "#f87171", "#fb7185", "#fecaca", "#14070a", "#1d0b10", "26 10 15"),
  {
    ...dark("amoled", "AMOLED", "Чистый чёрный фон", "#d4d4d8", "#8a8a92", "#e4e4e7", "#000000", "#000000", "0 0 0"),
    tint: [0, 0, 0, 255],
  },
  {
    id: "light",
    label: "Светлая",
    hint: "Светлый фон (экспериментально)",
    accent: "#2563eb",
    accent2: "#4f46e5",
    violet: "#3b82f6",
    bg: "#f4f5f8",
    bgSoft: "#eceef3",
    panel: "246 247 250",
    fg: "20 24 36",
    text: "#14161f",
    muted: "#5b6070",
    mutedStrong: "#2f3342",
    scheme: "light",
    tint: [244, 245, 248, 235],
  },
];

export interface Settings {
  theme: ThemeId;
  animations: boolean;
  blur: boolean;
  grid: boolean;
  compact: boolean;
}

const DEFAULTS: Settings = {
  theme: "mono",
  animations: true,
  blur: true,
  grid: true,
  compact: false,
};

const STORAGE_KEY = "setup-hub.settings.v1";

export const settings: Settings = $state({ ...DEFAULTS });

export function loadSettings(): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as Partial<Settings>;
    if (parsed.theme && THEMES.some((theme) => theme.id === parsed.theme)) {
      settings.theme = parsed.theme;
    }
    for (const key of ["animations", "blur", "grid", "compact"] as const) {
      if (typeof parsed[key] === "boolean") settings[key] = parsed[key];
    }
  } catch {
    // Corrupted or unavailable storage: keep defaults.
  }
}

export function saveSettings(): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // Storage may be unavailable; the settings still apply for this session.
  }
}

export function resetSettings(): void {
  Object.assign(settings, DEFAULTS);
  saveSettings();
}

export function applySettings(): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  const theme = THEMES.find((item) => item.id === settings.theme) ?? THEMES[0];
  const vars: Record<string, string> = {
    "--accent": theme.accent,
    "--accent-2": theme.accent2,
    "--violet": theme.violet,
    "--accent-soft": `color-mix(in srgb, ${theme.accent} 16%, transparent)`,
    "--bg": theme.bg,
    "--bg-soft": theme.bgSoft,
    "--panel": theme.panel,
    "--fg": theme.fg,
    "--text": theme.text,
    "--muted": theme.muted,
    "--muted-strong": theme.mutedStrong,
    "--surface": `rgb(${theme.fg} / 0.032)`,
    "--surface-strong": `rgb(${theme.fg} / 0.065)`,
    "--border": `rgb(${theme.fg} / 0.085)`,
    "--border-strong": `rgb(${theme.fg} / 0.17)`,
  };
  for (const [name, value] of Object.entries(vars)) root.style.setProperty(name, value);
  root.style.setProperty("color-scheme", theme.scheme);
  root.dataset.theme = theme.id;
  root.dataset.scheme = theme.scheme;

  root.classList.toggle("s-no-motion", !settings.animations);
  root.classList.toggle("s-no-blur", !settings.blur);
  root.classList.toggle("s-hide-grid", !settings.grid);
  root.classList.toggle("s-compact", settings.compact);

  // Native window: Mica tint + caption-bar theme follow the palette.
  void import("@tauri-apps/api/core")
    .then(({ invoke }) =>
      invoke("set_window_tint", { r: theme.tint[0], g: theme.tint[1], b: theme.tint[2], a: theme.tint[3], dark: theme.scheme === "dark" }),
    )
    .catch(() => {});
}
