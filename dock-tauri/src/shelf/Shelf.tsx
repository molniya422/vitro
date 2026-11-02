import type { ShelfItemView } from "../types";
import "./shelf.css";

interface Props {
  items: ShelfItemView[];
  loading: boolean;
  error: string | null;
  onOpen: (path: string) => void;
  onRemove: (path: string) => void;
  onClear: () => void;
  onClose: () => void;
}

/**
 * Session scratch space for files dropped on the dock. The panel is fully
 * interactive, so files can also be dropped straight onto it.
 */
export function Shelf({
  items,
  loading,
  error,
  onOpen,
  onRemove,
  onClear,
  onClose,
}: Props) {
  return (
    <section className="shelf" aria-label="Shelf">
      <header>
        <div>
          <h2>Shelf</h2>
          <p className="shelf-hint">
            Drag files onto the dock to park them here. They last until the dock
            restarts.
          </p>
        </div>
        <div className="shelf-actions">
          <button type="button" onClick={onClear} disabled={items.length === 0}>
            Clear
          </button>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </div>
      </header>

      {loading ? (
        <p className="shelf-empty" aria-busy="true">
          Loading…
        </p>
      ) : error ? (
        <p className="shelf-error" role="alert">
          {error}
        </p>
      ) : items.length === 0 ? (
        <p className="shelf-empty">
          Drag files here to keep them handy.
        </p>
      ) : (
        <ul className="shelf-list">
          {items.map((item) => (
            <li
              key={item.id}
              className={`shelf-item${item.missing ? " is-missing" : ""}`}
            >
              <button
                type="button"
                className="shelf-open"
                aria-label={`Open ${item.name}`}
                onClick={() => onOpen(item.path)}
                disabled={item.missing}
                title={item.path}
              >
                <span className="shelf-icon">
                  {item.icon ? (
                    <img src={item.icon} alt="" draggable={false} />
                  ) : (
                    <span className="fallback-icon" aria-hidden="true">
                      {item.name.slice(0, 1).toUpperCase()}
                    </span>
                  )}
                </span>
                <span className="shelf-meta">
                  <span className="shelf-name">{item.name}</span>
                  <span className="shelf-path">
                    {item.missing ? "Missing — " : ""}
                    {item.path}
                  </span>
                </span>
              </button>
              <button
                type="button"
                className="shelf-remove"
                aria-label={`Remove ${item.name}`}
                onClick={() => onRemove(item.path)}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
