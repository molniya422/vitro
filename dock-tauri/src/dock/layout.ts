import type { CSSProperties } from "react";
import type { DockAppearance } from "../types";

/** CSS custom properties consumed by dock.css. */
export type DockStyle = CSSProperties & { [key: `--${string}`]: string };

/**
 * Map appearance to the variables the dock CSS reads.
 *
 * Keeping this pure makes the size/tint/opacity contract testable without a
 * browser; Rust and React both derive the same numbers from Settings.
 */
export function cssVars(appearance: DockAppearance): DockStyle {
  return {
    "--icon-size": `${appearance.iconSize}px`,
    "--icon-gap": `${appearance.iconGap}px`,
    "--pad-h": `${appearance.paddingH}px`,
    "--pad-v": `${appearance.paddingV}px`,
    "--pill-radius": `${appearance.pillRadius}px`,
    "--glass-fill": hexToRgba(appearance.tint, appearance.fillOpacity),
    "--icon-radius": `${appearance.iconRadius}%`,
    "--reflection-space": `${appearance.reflectionSpace}px`,
    "--reflection-opacity": `${appearance.reflectionStrength}`,
  };
}

/**
 * Pill-local centers used by the magnification curve.
 *
 * The first app tile follows the control tile and the enabled route entries,
 * each `iconSize` wide with one gap between every pair.
 */
export function iconCenters(count: number, appearance: DockAppearance): number[] {
  const pitch = appearance.iconSize + appearance.iconGap;
  return Array.from(
    { length: count },
    (_, index) =>
      appearance.paddingH +
      (appearance.leadingTiles + index) * pitch +
      appearance.iconSize / 2,
  );
}

/** `#rrggbb` to a CSS rgba() string; invalid input falls back to the default tint. */
export function hexToRgba(hex: string, alpha: number): string {
  const match = /^#([0-9a-f]{6})$/i.exec(hex.trim());
  const value = match ? Number.parseInt(match[1], 16) : 0x161a26;
  const r = (value >> 16) & 0xff;
  const g = (value >> 8) & 0xff;
  const b = value & 0xff;
  const a = Math.min(1, Math.max(0, Number.isFinite(alpha) ? alpha : 0));
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}
