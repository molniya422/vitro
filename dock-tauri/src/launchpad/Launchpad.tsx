import { useMemo, useState } from "react";
import type { DockItemView } from "../types";
import { rankApps } from "./search";
import { AppTile } from "./AppTile";

interface Props {
  items: DockItemView[];
  loading: boolean;
  error: string | null;
  onLaunch: (id: string) => void;
  onClose: () => void;
}

export function Launchpad({ items, loading, error, onLaunch, onClose }: Props) {
  const [query, setQuery] = useState("");

  const searchable = useMemo(
    () => items.map((item) => ({ id: item.id, name: item.name, item })),
    [items],
  );
  const results = useMemo(() => rankApps(searchable, query), [searchable, query]);

  if (loading) {
    return (
      <section className="launchpad" aria-busy="true" aria-label="Launchpad">
        <p>Scanning installed applications…</p>
      </section>
    );
  }
  if (error) {
    return (
      <section className="launchpad" role="alert">
        <p>Could not load applications: {error}</p>
      </section>
    );
  }

  return (
    <section className="launchpad" aria-label="Launchpad">
      <header>
        <input
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search applications"
          aria-label="Search applications"
          autoFocus
        />
        <button type="button" onClick={onClose}>
          Close
        </button>
      </header>
      {items.length === 0 ? (
        <p className="launchpad-empty">No applications available.</p>
      ) : results.length === 0 ? (
        <p className="launchpad-empty">No applications match “{query}”.</p>
      ) : (
        <div className="launchpad-grid">
          {results.map(({ item }) => (
            <AppTile
              key={item.id}
              name={item.name}
              icon={item.icon}
              disabledReason={item.disabledReason}
              onLaunch={() => onLaunch(item.id)}
            />
          ))}
        </div>
      )}
    </section>
  );
}
