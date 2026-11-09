import { invoke } from "@tauri-apps/api/core";

export interface UsbDisk {
  number: number;
  name: string;
  sizeGb: number;
  style: string;
  letters: string;
}

export interface DiagCheck {
  id: string;
  title: string;
  status: "ok" | "warn" | "fail";
  detail: string;
  code: string;
  fixable: boolean;
}

export interface AppFolder {
  root: "roaming" | "local";
  name: string;
  sizeMb: number;
  files: number;
  modified: number;
  known: string | null;
  running: boolean;
}

export interface BackupEntry {
  root: "roaming" | "local";
  name: string;
  files: number;
  bytes: number;
}

export interface BackupManifest {
  version: number;
  created: number;
  computer: string;
  entries: BackupEntry[];
}

export interface BackupRequest {
  root: string;
  name: string;
}

export interface Package {
  id: string;
  name: string;
  version: string;
  available: string;
  source: string;
}

export interface ServiceInfo {
  name: string;
  display: string;
  state: string;
  mode: string;
  delayed: boolean;
  path: string;
  thirdParty: boolean;
  protected: boolean;
}

export interface StartupItem {
  id: string;
  name: string;
  command: string;
  location: string;
  enabled: boolean;
}

export interface JunkCategory {
  id: string;
  title: string;
  hint: string;
  sizeMb: number;
  files: number;
  requiresAdmin: boolean;
  recommended: boolean;
}

export interface CleanReport {
  freedMb: number;
  skippedFiles: number;
  notes: string[];
}

export interface MenuItem {
  scope: "files" | "folders" | "desktop";
  key: string;
  label: string;
  command: string;
  icon: string;
  disabled: boolean;
  extended: boolean;
  origin: "user" | "machine";
  ours: boolean;
}

export type AppCategory = "system" | "browsers" | "gaming" | "production";
export type InstallKind = "winget" | "download" | "custom";

export interface CatalogApp {
  id: string;
  name: string;
  summary: string;
  category: AppCategory;
  kind: InstallKind;
  wingetId: string | null;
  downloadUrl: string | null;
  installerArgs: string[];
  customConfig: Record<string, string>;
  presets: string[];
  dependsOn: string[];
  placeholder: boolean;
  sha256: string | null;
}

export interface Catalog {
  version: number;
  apps: CatalogApp[];
}

export interface TweakStatus {
  id: string;
  enabled: boolean;
  requiresAdmin: boolean;
}

export interface InstallProgress {
  appId: string;
  percent: number | null;
  phase: string;
  message: string | null;
}

export type JobStatus = "running" | "done" | "error";

export interface JobState {
  status: JobStatus;
  percent: number | null;
  phase: string;
  message?: string;
}

export type ToastTone = "success" | "error" | "info";

export interface ToastItem {
  id: number;
  tone: ToastTone;
  title: string;
  message?: string;
}

export const DISCLAIMER_KEY = "setup-hub.disclaimer.accepted.v1";

export const api = {
  catalog: () => invoke<Catalog>("bundled_catalog"),
  installWinget: (wingetId: string) =>
    invoke<string>("install_winget", { wingetId }),
  installEntry: (appId: string) => invoke<string>("install_entry", { appId }),
  tweakStates: () => invoke<TweakStatus[]>("tweak_states"),
  applyTweak: (id: string, enabled: boolean) =>
    invoke<string>("apply_tweak", { id, enabled }),
  strippedStates: () => invoke<TweakStatus[]>("stripped_states"),
  applyStripped: (id: string, enabled: boolean) =>
    invoke<string>("apply_stripped", { id, enabled }),
  createRestorePoint: () => invoke<string>("create_restore_point"),
  listUsbDisks: () => invoke<UsbDisk[]>("list_usb_disks"),
  createUsbInstaller: (
    isoPath: string,
    diskNumber: number,
    variant: string,
    scheme: string,
  ) =>
    invoke<string>("create_usb_installer", { isoPath, diskNumber, variant, scheme }),
  listServices: () => invoke<ServiceInfo[]>("list_services"),
  setServiceMode: (name: string, mode: string) =>
    invoke<string>("set_service_mode", { name, mode }),
  listStartup: () => invoke<StartupItem[]>("list_startup"),
  setStartupEnabled: (location: string, name: string, enabled: boolean) =>
    invoke<string>("set_startup_enabled", { location, name, enabled }),
  scanJunk: () => invoke<JunkCategory[]>("scan_junk"),
  cleanJunk: (ids: string[]) => invoke<CleanReport>("clean_junk", { ids }),
  emptyRecycleBin: () => invoke<string>("empty_recycle_bin"),
  listMenu: () => invoke<MenuItem[]>("list_menu"),
  addMenuItem: (scope: string, key: string, label: string, command: string, icon: string, extended: boolean) =>
    invoke<string>("add_menu_item", { scope, key, label, command, icon, extended }),
  setMenuDisabled: (scope: string, key: string, origin: string, disabled: boolean) =>
    invoke<string>("set_menu_disabled", { scope, key, origin, disabled }),
  removeMenuItem: (scope: string, key: string) =>
    invoke<string>("remove_menu_item", { scope, key }),
  listInstalled: () => invoke<Package[]>("list_installed"),
  listUpdates: () => invoke<Package[]>("list_updates"),
  upgradePackage: (id: string) => invoke<string>("upgrade_package", { id }),
  uninstallPackage: (id: string) => invoke<string>("uninstall_package", { id }),
  writePreset: (path: string, content: string) =>
    invoke<void>("write_preset", { path, content }),
  readPreset: (path: string) => invoke<string>("read_preset", { path }),
  scanAppdata: () => invoke<AppFolder[]>("scan_appdata"),
  createBackup: (items: BackupRequest[], destination: string, includeCache: boolean) =>
    invoke<string>("create_backup", { items, destination, includeCache }),
  inspectBackup: (path: string) => invoke<BackupManifest>("inspect_backup", { path }),
  restoreArchive: (path: string, selected: BackupRequest[]) =>
    invoke<string>("restore_archive", { path, selected }),
  runDiagnostics: () => invoke<DiagCheck[]>("run_diagnostics"),
  fixDiagnostic: (id: string) => invoke<string>("fix_diagnostic", { id }),
  exportDiagnostics: () => invoke<string>("export_diagnostics"),
  logFrontendError: (message: string) =>
    invoke<void>("log_frontend_error", { message }),
  restartExplorer: () => invoke<string>("restart_explorer"),
  adminStatus: () => invoke<boolean>("admin_status"),
  restartElevated: () => invoke<void>("restart_elevated"),
  customHookPath: () => invoke<string>("custom_hook_path"),
  restoreBackup: (source: string, scope: string, folderName: string) =>
    invoke<string>("restore_backup", { source, scope, folderName }),
  exit: () => invoke<void>("exit_app"),
};

export function humanizeError(error: unknown): string {
  const message = String(error);
  if (message.includes("admin_required")) {
    return "Требуются права администратора. Перезапустите приложение с повышенными правами.";
  }
  if (message.includes("not_found")) {
    return "Запись не найдена: возможно, изменение уже применено.";
  }
  if (message.includes("configuration template")) {
    return "Это шаблон. Укажите реальный URL и отключите placeholder в каталоге.";
  }
  return message;
}
