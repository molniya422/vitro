import { useMemo, useState } from "react";
import type { DockItemView } from "../types";
import { rankApps } from "./search";
import { AppTile } from "./AppTile";
import "./launchpad.css";

interface Props {
  items: DockItemView[];
  loading: boolean;
  error: string | null;
  pinnedIds?: string[];
  onPin?: (id: string) => Promise<void>;
  onLaunch: (id: string) => void;
  onClose: () => void;
}

export function Launchpad({
  items,
  loading,
  error,
  pinnedIds = [],
  onPin,
  onLaunch,
  onClose,
}: Props) {
  const [query, setQuery] = useState("");
  const [pinError, setPinError] = useState<string | null>(null);

  const searchable = useMemo(
    () => items.map((item) => ({ id: item.id, name: item.name, item })),
    [items],
  );
  const results = useMemo(() => rankApps(searchable, query), [searchable, query]);

  const pin = async (item: DockItemView) => {
    if (!onPin) return;
    setPinError(null);
    try {
      await onPin(item.id);
    } catch (e) {
      setPinError(String(e));
    }
  };

  if (loading) {
    return (
      <section className="launchpad" aria-busy="true" aria-label="Launchpad">
        <p className="launchpad-empty">Scanning installed applications…</p>
      </section>
    );
  }
  if (error) {
    return (
      <section className="launchpad" role="alert">
        <p className="launchpad-empty">Could not load applications: {error}</p>
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
        <button type="button" className="launchpad-close" onClick={onClose}>
          Close
        </button>
      </header>
      {pinError && (
        <p className="launchpad-error" role="alert">
          Could not add to the dock: {pinError}
        </p>
      )}
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
              pinned={pinnedIds.includes(item.id)}
              onPin={onPin ? () => void pin(item) : undefined}
              onLaunch={() => onLaunch(item.id)}
            />
          ))}
        </div>
      )}
    </section>
  );
}
