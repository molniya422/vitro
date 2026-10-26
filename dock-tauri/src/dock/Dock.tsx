import { useCallback, useMemo, useRef, useState } from "react";
import type { DockState } from "../types";
import { profileFor, scalesFor } from "./magnify";
import { DockItem } from "./DockItem";
import "./dock.css";

interface Props {
  state: DockState;
  onActivate: (id: string) => void;
}

const ICON_PITCH = 56; // icon width + gap; must match the CSS
const TOOLTIP_DELAY_MS = 500;

export function Dock({ state, onActivate }: Props) {
  const [pointerX, setPointerX] = useState<number | null>(null);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const tooltipTimer = useRef<number | null>(null);
  const profile = profileFor(state.profile, state.reducedMotion);

  const centers = useMemo(
    () => state.items.map((_, index) => index * ICON_PITCH + ICON_PITCH / 2),
    [state.items],
  );

  const scales = useMemo(() => {
    if (pointerX === null || !state.magnify || state.reducedMotion) {
      return state.items.map(() => 1);
    }
    return scalesFor(centers, pointerX, profile.peak, profile.falloff, profile.radius);
  }, [centers, pointerX, state.magnify, state.reducedMotion, profile, state.items]);

  const handlePointerMove = useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    // Read layout, not React state: this runs on every pointer move.
    const bounds = event.currentTarget.getBoundingClientRect();
    setPointerX(event.clientX - bounds.left);
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
      <div className="dock-pill" role="toolbar" aria-label="Dock">
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
