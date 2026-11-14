const KEY = "setup-hub.favorites.v1";

export const favorites: { ids: string[] } = $state({ ids: [] });

export function loadFavorites(): void {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) favorites.ids = parsed.filter((id) => typeof id === "string");
  } catch {
    // Storage unavailable or corrupted: start empty.
  }
}

function save(): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(favorites.ids));
  } catch {
    // Favorites still work for this session.
  }
}

export function isFavorite(id: string): boolean {
  return favorites.ids.includes(id);
}

export function toggleFavorite(id: string): void {
  favorites.ids = isFavorite(id) ? favorites.ids.filter((item) => item !== id) : [...favorites.ids, id];
  save();
}

export function setFavorites(ids: string[]): void {
  favorites.ids = ids.filter((id) => typeof id === "string");
  save();
}
