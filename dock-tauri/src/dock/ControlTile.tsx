import { useEffect, useRef } from "react";

interface Props {
  collapsed: boolean;
  onSettings: () => void;
  onToggleCollapsed: () => void;
}

/** A click that lands inside this window is a single click, not a double. */
const SINGLE_CLICK_DELAY_MS = 250;

/**
 * Left-edge control tile. A single click opens settings; a second click within
 * the delay cancels it and collapses/expands instead, so the first click of a
 * double click never leaks into settings.
 */
export function ControlTile({ collapsed, onSettings, onToggleCollapsed }: Props) {
  const pending = useRef<number | null>(null);

  useEffect(
    () => () => {
      if (pending.current !== null) {
        window.clearTimeout(pending.current);
      }
    },
    [],
  );

  const handleClick = () => {
    if (pending.current !== null) {
      window.clearTimeout(pending.current);
      pending.current = null;
      onToggleCollapsed();
      return;
    }
    pending.current = window.setTimeout(() => {
      pending.current = null;
      onSettings();
    }, SINGLE_CLICK_DELAY_MS);
  };

  return (
    <button
      type="button"
      className="dock-item control-tile"
      aria-label="Dock settings"
      aria-expanded={!collapsed}
      onClick={handleClick}
    >
      <span className="fallback-icon" aria-hidden="true">
        ⚙
      </span>
    </button>
  );
}
