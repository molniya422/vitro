import { useRef } from "react";
import type { DockAppearance, DockItemView } from "../types";
import { Glyph, glyphForSystem } from "./Glyph";
import { Tooltip } from "./Tooltip";

interface Props {
  item: DockItemView;
  appearance: DockAppearance;
  scale: number;
  magnifying: boolean;
  reducedMotion: boolean;
  showTooltip: boolean;
  /** Horizontal offset in px: the drag follow or the gap opening. */
  offsetX?: number;
  dragging?: boolean;
  /** A launch just started; play the bounce. */
  launching?: boolean;
  onPointerDown?: (event: React.PointerEvent<HTMLButtonElement>) => void;
  onHover: (id: string | null) => void;
  onActivate: (id: string) => void;
  onContextMenu: (id: string, x: number, y: number) => void;
}

export function DockItem({
  item,
  appearance,
  scale,
  magnifying,
  reducedMotion,
  showTooltip,
  offsetX = 0,
  dragging = false,
  launching = false,
  onPointerDown,
  onHover,
  onActivate,
  onContextMenu,
}: Props) {
  const ref = useRef<HTMLButtonElement>(null);

  const className = [
    "dock-item",
    magnifying ? "is-magnifying" : "",
    reducedMotion ? "is-reduced" : "",
    item.disabledReason ? "is-disabled" : "",
    dragging ? "is-dragging" : "",
    launching ? "is-launching" : "",
  ]
    .filter(Boolean)
    .join(" ");

  const style = {
    transform: `translateX(${offsetX}px) scale(${reducedMotion || dragging ? 1 : scale})`,
  };

  return (
    <button
      ref={ref}
      type="button"
      className={className}
      style={style}
      title={item.disabledReason ?? undefined}
      aria-label={`${item.name}${item.running ? ", running" : ""}`}
      onPointerDown={onPointerDown}
      onMouseEnter={() => onHover(item.id)}
      onMouseLeave={() => onHover(null)}
      onClick={() => onActivate(item.id)}
      onContextMenu={(event) => {
        event.preventDefault();
        onContextMenu(item.id, event.clientX, event.clientY);
      }}
      disabled={Boolean(item.disabledReason)}
    >
      {item.icon ? (
        <img src={item.icon} alt="" draggable={false} />
      ) : item.kind === "system" ? (
        <span className="app-icon system-icon">
          <Glyph name={glyphForSystem(item.name)} />
        </span>
      ) : (
        <span className="fallback-icon" aria-hidden="true">
          {item.name.slice(0, 1).toUpperCase()}
        </span>
      )}
      {item.icon && appearance.reflection && (
        <img className="reflection" src={item.icon} alt="" aria-hidden="true" draggable={false} />
      )}
      {item.running && appearance.showRunningDots && (
        <span className="running-dot" aria-hidden="true" />
      )}
      {item.minimized > 0 && appearance.showBadges && (
        <span className="badge" aria-label={`${item.minimized} minimized`}>
          {item.minimized}
        </span>
      )}
      {showTooltip && <Tooltip label={item.name} />}
    </button>
  );
}
