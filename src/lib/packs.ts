export type Pack = { id: string; icon: string };

export const APP_PACKS: Pack[] = [
  { id: "base", icon: "package" },
  { id: "designer", icon: "sliders" },
  { id: "gamer", icon: "gamepad" },
  { id: "dev", icon: "terminal" },
  { id: "creator", icon: "rocket" },
  { id: "office", icon: "archive" },
  { id: "tools", icon: "shield" },
];

export type TweakPack = { id: string; icon: string; tweaks: string[] };

export const TWEAK_PACKS: TweakPack[] = [
  {
    id: "user",
    icon: "user",
    tweaks: [
      "show_file_extensions",
      "disable_bing_search",
      "disable_advertising_id",
      "disable_suggestions",
      "disable_activity_history",
      "classic_context_menu",
    ],
  },
  {
    id: "gamer",
    icon: "gamepad",
    tweaks: [
      "disable_game_dvr",
      "disable_telemetry",
      "disable_advertising_id",
      "disable_suggestions",
      "disable_bing_search",
      "disable_activity_history",
    ],
  },
  {
    id: "privacy",
    icon: "shield",
    tweaks: [
      "disable_telemetry",
      "disable_advertising_id",
      "disable_activity_history",
      "disable_bing_search",
      "disable_suggestions",
      "disable_copilot",
    ],
  },
  {
    id: "dev",
    icon: "terminal",
    tweaks: [
      "show_file_extensions",
      "show_hidden_files",
      "enable_long_paths",
      "classic_context_menu",
      "dark_mode",
    ],
  },
];

export const STRIPPED_PACKS: TweakPack[] = [
  {
    id: "game",
    icon: "gamepad",
    tweaks: [
      "fullscreen_optimizations",
      "network_throttling",
      "games_priority",
      "mouse_acceleration",
      "sticky_keys_prompts",
      "background_apps",
      "delivery_optimization",
    ],
  },
  {
    id: "light",
    icon: "rocket",
    tweaks: [
      "widgets_news",
      "telemetry_tasks",
      "background_apps",
      "ntfs_last_access",
      "svc_error_reporting",
      "svc_maps",
      "delivery_optimization",
    ],
  },
];
