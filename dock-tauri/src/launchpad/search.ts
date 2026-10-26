export interface SearchableApp {
  id: string;
  name: string;
}

/**
 * Filter and rank apps. Empty query returns everything alphabetically.
 * Prefix matches rank above substring matches; ties break alphabetically.
 */
export function rankApps<T extends SearchableApp>(apps: T[], query: string): T[] {
  const needle = query.trim().toLowerCase();
  if (needle.length === 0) {
    return [...apps].sort((a, b) =>
      a.name.toLowerCase().localeCompare(b.name.toLowerCase()),
    );
  }

  const scored = apps
    .map((app) => {
      const name = app.name.toLowerCase();
      if (name.startsWith(needle)) return { app, score: 0 };
      if (name.includes(needle)) return { app, score: 1 };
      return null;
    })
    .filter((entry): entry is { app: T; score: number } => entry !== null);

  scored.sort((a, b) => {
    if (a.score !== b.score) return a.score - b.score;
    return a.app.name.toLowerCase().localeCompare(b.app.name.toLowerCase());
  });
  return scored.map((entry) => entry.app);
}
