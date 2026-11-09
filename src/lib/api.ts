import { invoke } from "@tauri-apps/api/core";

export interface UsbDisk {
  number: number;
  name: string;
  sizeGb: number;
  style: string;
  letters: string;
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
