import { useCallback, useMemo, useRef, useState } from "react";
import type { DockState } from "../types";
import { profileFor, scalesFor } from "./magnify";
import { ControlTile } from "./ControlTile";
import { DockItem } from "./DockItem";
import { Glyph } from "./Glyph";
import "./dock.css";

interface Props {
  state: DockState;
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onOpenSettings: () => void;
  onOpenLaunchpad: () => void;
  onOpenMonitor: () => void;
  onActivate: (id: string) => void;
}

const ICON_SIZE = 48; // must match --icon-size in dock.css
const ICON_GAP = 8; // must match --icon-gap
const ICON_PITCH = ICON_SIZE + ICON_GAP;
const PILL_PADDING_H = 16; // must match --pad-h
const TOOLTIP_DELAY_MS = 500;
// The control tile and the two route entry points sit before the app tiles.
const LEADING_TILES = 3;

export function Dock({
  state,
  collapsed,
  onToggleCollapsed,
  onOpenSettings,
  onOpenLaunchpad,
  onOpenMonitor,
  onActivate,
}: Props) {
  const [pointerX, setPointerX] = useState<number | null>(null);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const tooltipTimer = useRef<number | null>(null);
  const pillRef = useRef<HTMLDivElement>(null);
  const profile = profileFor(state.profile, state.reducedMotion);

  // Centers are pill-local: the leading padding plus whole icon pitches. Using
  // pitch/2 as the first center was 12 px short and shifted the magnify peak.
  const centers = useMemo(
    () =>
      state.items.map(
        (_, index) =>
          PILL_PADDING_H +
          (index + LEADING_TILES) * ICON_PITCH +
          ICON_SIZE / 2,
      ),
    [state.items],
  );

  const scales = useMemo(() => {
    if (pointerX === null || !state.magnify || state.reducedMotion) {
      return state.items.map(() => 1);
    }
    return scalesFor(centers, pointerX, profile.peak, profile.falloff, profile.radius);
  }, [centers, pointerX, state.magnify, state.reducedMotion, profile, state.items]);

  const handlePointerMove = useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    // Measure against the pill, not the stage: the icon centers are pill-local,
    // and the stage is wider than the pill whenever the window is not exactly
    // pill-sized (a browser, or a window mid-route-change).
    const pill = pillRef.current;
    if (!pill) return;
    setPointerX(event.clientX - pill.getBoundingClientRect().left);
  }, []);

  const handleHover = useCallback((id: string | null) => {
    if (tooltipTimer.current !== null) {
      window.clearTimeout(tooltipTimer.current);
      tooltipTimer.current = null;
    }
    if (id === null) {
      setHoveredId(null);
      return;
    }
    tooltipTimer.current = window.setTimeout(
      () => setHoveredId(id),
      TOOLTIP_DELAY_MS,
    );
  }, []);

  return (
    <div className="dock-stage" onPointerMove={handlePointerMove} onPointerLeave={() => setPointerX(null)}>
      <div
        ref={pillRef}
        className={`dock-pill${collapsed ? " is-collapsed" : ""}`}
        role="toolbar"
        aria-label="Dock"
      >
        <ControlTile
          collapsed={collapsed}
          onSettings={onOpenSettings}
          onToggleCollapsed={onToggleCollapsed}
        />
        <button
          type="button"
          className="dock-item route-entry"
          aria-label="Open Launchpad"
          onClick={onOpenLaunchpad}
        >
          <span className="app-icon system-icon">
            <Glyph name="grid" />
          </span>
        </button>
        <button
          type="button"
          className="dock-item route-entry"
          aria-label="Open system monitor"
          onClick={onOpenMonitor}
        >
          <span className="app-icon system-icon">
            <Glyph name="chart" />
          </span>
        </button>
        {state.items.map((item, index) => (
          <DockItem
            key={item.id}
            item={item}
            scale={scales[index]}
            magnifying={pointerX !== null && scales[index] > 1.001}
            reducedMotion={state.reducedMotion}
            showTooltip={hoveredId === item.id}
            onHover={handleHover}
            onActivate={onActivate}
          />
        ))}
      </div>
    </div>
  );
}
