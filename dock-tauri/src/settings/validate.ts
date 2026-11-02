import type { SettingsView } from "../types";

export type SettingsInput = SettingsView;

/** Mirrors `Settings::default()` in Rust. */
export const DEFAULT_SETTINGS: SettingsView = {
  iconSize: 48,
  iconGap: 8,
  paddingH: 16,
  paddingV: 10,
  pillRadius: 34,
  tint: "#161a26",
  fillOpacity: 0.62,
  iconRadius: 22,
  reflection: true,
  reflectionStrength: 0.25,
  magnify: true,
  magnifyPeak: 1.6,
  magnifyRadiusIcons: 2.2,
  profile: "vitro",
  reducedMotion: false,
  tooltips: true,
  tooltipDelayMs: 500,
  showRunningDots: true,
  showBadges: true,
  autohide: true,
  showDelayMs: 300,
  hideDelayMs: 500,
  yieldFullscreen: true,
  showLaunchpadEntry: true,
  showMonitorEntry: true,
  showShelfEntry: true,
};

export type ValidationResult = { ok: true } | { ok: false; errors: string[] };

const INTEGER_RANGES: Array<[keyof SettingsView, number, number]> = [
  ["iconSize", 32, 80],
  ["iconGap", 0, 24],
  ["paddingH", 4, 32],
  ["paddingV", 2, 24],
  ["pillRadius", 0, 64],
  ["iconRadius", 0, 50],
  ["tooltipDelayMs", 0, 2_000],
  ["showDelayMs", 0, 1_000],
  ["hideDelayMs", 100, 3_000],
];

const FLOAT_RANGES: Array<[keyof SettingsView, number, number]> = [
  ["fillOpacity", 0.15, 1],
  ["reflectionStrength", 0, 1],
  ["magnifyPeak", 1, 2],
  ["magnifyRadiusIcons", 1, 5],
];

export function validateSettings(input: SettingsInput): ValidationResult {
  const errors: string[] = [];

  for (const [key, min, max] of INTEGER_RANGES) {
    const value = input[key] as number;
    if (!Number.isInteger(value) || value < min || value > max) {
      errors.push(`${key} must be an integer between ${min} and ${max}`);
    }
  }
  for (const [key, min, max] of FLOAT_RANGES) {
    const value = input[key] as number;
    if (!Number.isFinite(value) || value < min || value > max) {
      errors.push(`${key} must be between ${min} and ${max}`);
    }
  }
  if (!/^#[0-9a-f]{6}$/i.test(input.tint)) {
    errors.push("tint must be a #rrggbb colour");
  }
  if (input.profile !== "macos" && input.profile !== "vitro") {
    errors.push(`profile must be "macos" or "vitro", got "${input.profile}"`);
  }

  return errors.length === 0 ? { ok: true } : { ok: false, errors };
}
