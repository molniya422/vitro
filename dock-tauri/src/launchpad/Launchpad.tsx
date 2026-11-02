import { useEffect, useMemo, useRef, useState } from "react";
import type { DockItemView } from "../types";
import { placeAt } from "../dock/drag";
import { rankApps } from "./search";
import { AppTile } from "./AppTile";
import "./launchpad.css";

/** Pointer travel before a press becomes a drag instead of a launch. */
const DRAG_THRESHOLD_PX = 5;
/** Bottom band that accepts a drop; the dock strip renders inside it. */
const DROP_BAND_PX = 140;

interface Props {
  items: DockItemView[];
  loading: boolean;
  error: string | null;
  /** The current dock contents, previewed under the pointer while dragging. */
  dockItems?: DockItemView[];
  onPin?: (id: string) => Promise<void>;
  /** Drop at `index` in the dock. */
  onDropPin?: (id: string, index: number) => Promise<void>;
  onLaunch: (id: string) => void;
  onClose: () => void;
}

interface DragState {
  item: DockItemView;
  x: number;
  y: number;
  over: boolean;
  gap: number;
}

export function Launchpad({
  items,
  loading,
  error,
  dockItems = [],
  onPin,
  onDropPin,
  onLaunch,
  onClose,
}: Props) {
  const [query, setQuery] = useState("");
  const [pinError, setPinError] = useState<string | null>(null);
  const [drag, setDrag] = useState<DragState | null>(null);
  const press = useRef<{
    item: DockItemView;
    x: number;
    y: number;
    pointerId: number;
  } | null>(null);
  const dragRef = useRef<DragState | null>(null);
  const suppressClick = useRef(false);
  const stripRef = useRef<HTMLDivElement>(null);

  const pinnedIds = useMemo(
    () => new Set(dockItems.map((item) => item.id)),
    [dockItems],
  );
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

  const drop = async (item: DockItemView, index: number) => {
    if (!onDropPin) return;
    setPinError(null);
    try {
      await onDropPin(item.id, index);
    } catch (e) {
      setPinError(String(e));
    }
  };

  // The gesture listeners are installed once; the ref keeps them pointing at
  // the current callbacks.
  const handlers = useRef({ drop });
  handlers.current = { drop };

  useEffect(() => {
    const overBand = (y: number) => y >= window.innerHeight - DROP_BAND_PX;

    // Gap indices come from the strip's rendered icons. Measuring the preview
    // (which already includes the dragged app) keeps the indicator where the
    // pointer sees it, whatever the window size or icon count.
    const gapAt = (x: number) => {
      const strip = stripRef.current;
      if (!strip) return 0;
      const centers = Array.from(
        strip.querySelectorAll<HTMLElement>("[data-drop-id]"),
        (element) => {
          const rect = element.getBoundingClientRect();
          return rect.left + rect.width / 2;
        },
      );
      return centers.filter((center) => x > center).length;
    };

    const move = (event: PointerEvent) => {
      const start = press.current;
      if (!start || event.pointerId !== start.pointerId) return;
      if (!dragRef.current) {
        const distance = Math.hypot(
          event.clientX - start.x,
          event.clientY - start.y,
        );
        if (distance < DRAG_THRESHOLD_PX) return;
        // Releasing a drag must not launch the app.
        suppressClick.current = true;
      }
      const next: DragState = {
        item: start.item,
        x: event.clientX,
        y: event.clientY,
        over: overBand(event.clientY),
        gap: gapAt(event.clientX),
      };
      dragRef.current = next;
      setDrag(next);
    };

    const finish = (event: PointerEvent, commit: boolean) => {
      const start = press.current;
      const active = dragRef.current;
      press.current = null;
      dragRef.current = null;
      setDrag(null);
      if (!start || !active) return;
      if (!commit || !overBand(event.clientY)) return;
      void handlers.current.drop(start.item, gapAt(event.clientX));
    };

    const up = (event: PointerEvent) => {
      if (!press.current || event.pointerId !== press.current.pointerId) return;
      finish(event, true);
    };
    const cancel = (event: PointerEvent) => {
      if (!press.current || event.pointerId !== press.current.pointerId) return;
      finish(event, false);
    };
    const blur = () => {
      press.current = null;
      dragRef.current = null;
      setDrag(null);
    };

    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    window.addEventListener("pointercancel", cancel);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      window.removeEventListener("pointercancel", cancel);
      window.removeEventListener("blur", blur);
    };
  }, []);

  const beginPress = (
    item: DockItemView,
    event: React.PointerEvent<HTMLButtonElement>,
  ) => {
    suppressClick.current = false;
    if (
      event.button !== 0 ||
      !onDropPin ||
      item.disabledReason ||
      pinnedIds.has(item.id)
    ) {
      return;
    }
    press.current = {
      item,
      x: event.clientX,
      y: event.clientY,
      pointerId: event.pointerId,
    };
  };

  const launch = (item: DockItemView) => {
    if (suppressClick.current) {
      suppressClick.current = false;
      return;
    }
    onLaunch(item.id);
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

  const stripItems = drag
    ? placeAt(
        dockItems.map((item) => item.id),
        drag.item.id,
        drag.gap,
      )
    : [];
  const byId = new Map(dockItems.map((item) => [item.id, item]));
  if (drag) byId.set(drag.item.id, drag.item);

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
              pinned={pinnedIds.has(item.id)}
              draggable={
                Boolean(onDropPin) &&
                !item.disabledReason &&
                !pinnedIds.has(item.id)
              }
              dragging={drag?.item.id === item.id}
              onPointerDown={(event) => beginPress(item, event)}
              onPin={onPin ? () => void pin(item) : undefined}
              onLaunch={() => launch(item)}
            />
          ))}
        </div>
      )}
      {drag && (
        <>
          <div
            ref={stripRef}
            className={`launchpad-drop${drag.over ? " is-over" : ""}`}
            role="group"
            aria-label="Dock drop target"
          >
            <span className="launchpad-drop-hint">Drop to add to the Dock</span>
            {stripItems.map((id) => {
              const item = byId.get(id);
              if (!item) return null;
              return (
                <span
                  key={id}
                  className="launchpad-drop-icon"
                  data-drop-id={id}
                  title={item.name}
                >
                  {item.icon ? (
                    <img src={item.icon} alt="" draggable={false} />
                  ) : (
                    <span className="fallback-icon" aria-hidden="true">
                      {item.name.slice(0, 1).toUpperCase()}
                    </span>
                  )}
                </span>
              );
            })}
          </div>
          <div
            className="launchpad-ghost"
            style={{ left: drag.x, top: drag.y }}
            aria-hidden="true"
          >
            {drag.item.icon ? (
              <img src={drag.item.icon} alt="" draggable={false} />
            ) : (
              <span className="fallback-icon">
                {drag.item.name.slice(0, 1).toUpperCase()}
              </span>
            )}
          </div>
        </>
      )}
    </section>
  );
}
