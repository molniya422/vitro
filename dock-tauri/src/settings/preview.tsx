import { cssVars } from "../dock/layout";
import type { DockAppearance, SettingsView } from "../types";

/**
 * Derive the same appearance Rust sends with `dock://state` so the preview
 * shows real size, spacing, tint, opacity, radius, and reflection.
 */
export function previewAppearance(settings: SettingsView): DockAppearance {
  const leadingTiles =
    1 +
    Number(settings.showLaunchpadEntry) +
    Number(settings.showMonitorEntry) +
    Number(settings.showShelfEntry);
  return {
    iconSize: settings.iconSize,
    iconGap: settings.iconGap,
    paddingH: settings.paddingH,
    paddingV: settings.paddingV,
    pillRadius: settings.pillRadius,
    tint: settings.tint,
    fillOpacity: settings.fillOpacity,
    iconRadius: settings.iconRadius,
    reflection: settings.reflection,
    reflectionStrength: settings.reflectionStrength,
    reflectionSpace: settings.reflection
      ? Math.round((settings.iconSize * 5) / 6)
      : 0,
    magnify: settings.magnify,
    magnifyPeak: settings.magnifyPeak,
    magnifyRadius:
      Math.round(
        settings.magnifyRadiusIcons * (settings.iconSize + settings.iconGap) * 100,
      ) / 100,
    reducedMotion: settings.reducedMotion,
    profile: settings.profile,
    tooltips: settings.tooltips,
    tooltipDelayMs: settings.tooltipDelayMs,
    showRunningDots: settings.showRunningDots,
    showBadges: settings.showBadges,
    showLaunchpadEntry: settings.showLaunchpadEntry,
    showMonitorEntry: settings.showMonitorEntry,
    showShelfEntry: settings.showShelfEntry,
    leadingTiles,
  };
}

const TILES = ["A", "B", "C"];

export function DockPreview({ settings }: { settings: SettingsView }) {
  const appearance = previewAppearance(settings);
  return (
    <div className="dock-preview" role="img" aria-label="Dock preview">
      <div className="dock-preview-pill" style={cssVars(appearance)}>
        <span className="dock-preview-tile is-control" aria-hidden="true">
          ≡
        </span>
        {settings.showLaunchpadEntry && (
          <span className="dock-preview-tile is-control" aria-hidden="true">
            ⊞
          </span>
        )}
        {settings.showMonitorEntry && (
          <span className="dock-preview-tile is-control" aria-hidden="true">
            ∿
          </span>
        )}
        {settings.showShelfEntry && (
          <span className="dock-preview-tile is-control" aria-hidden="true">
            ⌄
          </span>
        )}
        {TILES.map((letter, index) => (
          <span className="dock-preview-tile" key={letter} aria-hidden="true">
            {letter}
            {index === 1 && settings.showRunningDots && (
              <span className="running-dot" />
            )}
          </span>
        ))}
      </div>
      {settings.reflection && (
        <div
          className="dock-preview-reflection"
          style={{ opacity: settings.reflectionStrength }}
          aria-hidden="true"
        />
      )}
    </div>
  );
}
