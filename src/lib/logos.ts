const logos: Record<string, string> = {
  "7zip": "/logos/7zip.svg",
  "adobe-creative-cloud": "/logos/creative-cloud.svg",
  "creative-cloud": "/logos/creative-cloud.svg",
  audacity: "/logos/audacity.svg",
  "battle-net": "/logos/battle-net.svg",
  brave: "/logos/brave.svg",
  chrome: "/logos/google-chrome.svg",
  cursor: "/logos/cursor.svg",
  "davinci-resolve": "/logos/davinci-resolve.svg",
  discord: "/logos/discord.svg",
  "ea-app": "/logos/ea-app.svg",
  "epic-games": "/logos/epic-games.svg",
  everything: "/logos/everything.svg",
  firefox: "/logos/firefox.svg",
  git: "/logos/git.svg",
  "gog-galaxy": "/logos/gog-galaxy.svg",
  happ: "/logos/happ.svg",
  "notepad-plus-plus": "/logos/notepad-plus-plus.svg",
  "obs-studio": "/logos/obs-studio.svg",
  powertoys: "/logos/powertoys.svg",
  prism: "/logos/prism.svg",
  spotify: "/logos/spotify.svg",
  steam: "/logos/steam.svg",
  telegram: "/logos/telegram.svg",
  vlc: "/logos/vlc.svg",
  vscode: "/logos/vscode.svg",
  winrar: "/logos/winrar.svg",
  zoom: "/logos/zoom.svg",
};

export function logoFor(appId: string): string | null {
  return logos[appId] ?? null;
}
