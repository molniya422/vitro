import { useRef } from "react";
import type { DockItemView } from "../types";
import { Tooltip } from "./Tooltip";

interface Props {
  item: DockItemView;
  scale: number;
  magnifying: boolean;
  reducedMotion: boolean;
  showTooltip: boolean;
  onHover: (id: string | null) => void;
  onActivate: (id: string) => void;
}

export function DockItem({
  item,
  scale,
  magnifying,
  reducedMotion,
  showTooltip,
  onHover,
  onActivate,
}: Props) {
  const ref = useRef<HTMLButtonElement>(null);

  const className = [
    "dock-item",
    magnifying ? "is-magnifying" : "",
    reducedMotion ? "is-reduced" : "",
    item.disabledReason ? "is-disabled" : "",
  ]
    .filter(Boolean)
    .join(" ");

  const style = reducedMotion
    ? { opacity: 1 }
    : { transform: `scale(${scale})` };

  return (
    <button
      ref={ref}
      type="button"
      className={className}
      style={style}
      title={item.disabledReason ?? undefined}
      aria-label={`${item.name}${item.running ? ", running" : ""}`}
      onMouseEnter={() => onHover(item.id)}
      onMouseLeave={() => onHover(null)}
      onClick={() => onActivate(item.id)}
      disabled={Boolean(item.disabledReason)}
    >
      {item.icon ? (
        <img src={item.icon} alt="" draggable={false} />
      ) : (
        <span className="fallback-icon" aria-hidden="true">
          {item.name.slice(0, 1).toUpperCase()}
        </span>
      )}
      {item.icon && (
        <img className="reflection" src={item.icon} alt="" aria-hidden="true" draggable={false} />
      )}
      {item.running && <span className="running-dot" aria-hidden="true" />}
      {item.minimized > 0 && (
        <span className="badge" aria-label={`${item.minimized} minimized`}>
          {item.minimized}
        </span>
      )}
      {showTooltip && <Tooltip label={item.name} />}
    </button>
  );
}
