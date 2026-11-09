// Global app settings: persisted in localStorage and applied to <html>.

export type ThemeId = "mono" | "cyan" | "violet" | "emerald" | "rose" | "amber" | "amoled";

export interface ThemeDef {
  id: ThemeId;
  label: string;
  hint: string;
  accent: string;
  accent2: string;
  violet: string;
}

export const THEMES: ThemeDef[] = [
  { id: "mono", label: "Моно", hint: "Классическая чёрно-белая", accent: "#f5f5f6", accent2: "#a1a1aa", violet: "#d4d4d8" },
  { id: "cyan", label: "Бирюза", hint: "Холодный неон", accent: "#22d3ee", accent2: "#38bdf8", violet: "#a5f3fc" },
  { id: "violet", label: "Фиолет", hint: "Мягкий лавандовый", accent: "#a78bfa", accent2: "#818cf8", violet: "#ddd6fe" },
  { id: "emerald", label: "Изумруд", hint: "Свежий зелёный", accent: "#34d399", accent2: "#2dd4bf", violet: "#a7f3d0" },
  { id: "rose", label: "Роза", hint: "Тёплый розовый", accent: "#fb7185", accent2: "#f472b6", violet: "#fecdd3" },
  { id: "amber", label: "Янтарь", hint: "Золотистый акцент", accent: "#fbbf24", accent2: "#fb923c", violet: "#fde68a" },
  { id: "amoled", label: "AMOLED", hint: "Чистый чёрный фон", accent: "#d4d4d8", accent2: "#8a8a92", violet: "#e4e4e7" },
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

  root.style.setProperty("--accent", theme.accent);
  root.style.setProperty("--accent-2", theme.accent2);
  root.style.setProperty("--violet", theme.violet);
  root.style.setProperty("--accent-soft", `color-mix(in srgb, ${theme.accent} 16%, transparent)`);
  root.dataset.theme = theme.id;

  root.classList.toggle("s-no-motion", !settings.animations);
  root.classList.toggle("s-no-blur", !settings.blur);
  root.classList.toggle("s-hide-grid", !settings.grid);
  root.classList.toggle("s-compact", settings.compact);
}
