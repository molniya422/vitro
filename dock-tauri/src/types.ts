export type Edge = "bottom";

export type MotionProfile = "macos" | "vitro";

export type ItemKind = "app" | "separator" | "system";

export interface DockItemView {
  id: string;
  kind: ItemKind;
  name: string;
  icon: string | null;
  running: boolean;
  minimized: number;
  disabledReason: string | null;
}

/** Everything React needs to style and lay out the dock. */
export interface DockAppearance {
  iconSize: number;
  iconGap: number;
  paddingH: number;
  paddingV: number;
  pillRadius: number;
  tint: string;
  fillOpacity: number;
  iconRadius: number;
  reflection: boolean;
  reflectionStrength: number;
  reflectionSpace: number;
  magnify: boolean;
  magnifyPeak: number;
  /** Magnify reach in CSS pixels, derived by Rust from the icons-by-reach setting. */
  magnifyRadius: number;
  reducedMotion: boolean;
  profile: MotionProfile;
  tooltips: boolean;
  tooltipDelayMs: number;
  showRunningDots: boolean;
  showBadges: boolean;
  showLaunchpadEntry: boolean;
  showMonitorEntry: boolean;
  showShelfEntry: boolean;
  /** Control tile + enabled route entries before the app tiles. */
  leadingTiles: number;
}

export interface DockState {
  items: DockItemView[];
  edge: Edge;
  appearance: DockAppearance;
}

/**
 * The camelCase settings payload. `schemaVersion` and `pinned` are owned by
 * Rust: the form can never overwrite either.
 */
export interface SettingsView {
  iconSize: number;
  iconGap: number;
  paddingH: number;
  paddingV: number;
  pillRadius: number;
  tint: string;
  fillOpacity: number;
  iconRadius: number;
  reflection: boolean;
  reflectionStrength: number;
  magnify: boolean;
  magnifyPeak: number;
  magnifyRadiusIcons: number;
  profile: MotionProfile;
  reducedMotion: boolean;
  tooltips: boolean;
  tooltipDelayMs: number;
  showRunningDots: boolean;
  showBadges: boolean;
  autohide: boolean;
  showDelayMs: number;
  hideDelayMs: number;
  yieldFullscreen: boolean;
  showLaunchpadEntry: boolean;
  showMonitorEntry: boolean;
  showShelfEntry: boolean;
}

/** A file or folder parked on the session shelf. */
export interface ShelfItemView {
  id: string;
  name: string;
  path: string;
  isDir: boolean;
  missing: boolean;
  icon: string | null;
}

/** An interactive rect in CSS pixels, relative to the window's client origin. */
export interface HitRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface MonitorSample {
  cpu: number;
  memUsed: number;
  memTotal: number;
  diskUsed: number;
  diskTotal: number;
  netRx: number;
  netTx: number;
}

