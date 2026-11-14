import { APP_VERSION, REPO } from "./version";

export interface UpdateInfo {
  latest: string;
  url: string;
  newer: boolean;
}

function parts(version: string): number[] {
  return version.replace(/^v/i, "").split(".").map((n) => Number.parseInt(n, 10) || 0);
}

export function isNewer(latest: string, current: string): boolean {
  const a = parts(latest);
  const b = parts(current);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const x = a[i] ?? 0;
    const y = b[i] ?? 0;
    if (x !== y) return x > y;
  }
  return false;
}

export async function checkForUpdate(): Promise<UpdateInfo> {
  const response = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers: { Accept: "application/vnd.github+json" },
  });
  if (!response.ok) throw new Error(`GitHub ${response.status}`);
  const data = (await response.json()) as { tag_name?: string; html_url?: string };
  const latest = String(data.tag_name ?? "");
  if (!latest) throw new Error("no release");
  return {
    latest: latest.replace(/^v/i, ""),
    url: data.html_url ?? `https://github.com/${REPO}/releases`,
    newer: isNewer(latest, APP_VERSION),
  };
}
