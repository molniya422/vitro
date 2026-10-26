export interface SettingsInput {
  iconSize: number;
  magnify: boolean;
  autohide: boolean;
  reducedMotion: boolean;
  profile: string;
  showMonitor: boolean;
}

export type ValidationResult =
  | { ok: true }
  | { ok: false; errors: string[] };

export function validateSettings(input: SettingsInput): ValidationResult {
  const errors: string[] = [];

  if (!Number.isInteger(input.iconSize) || input.iconSize < 16 || input.iconSize > 128) {
    errors.push("iconSize must be an integer between 16 and 128");
  }
  if (input.profile !== "macos" && input.profile !== "vitro") {
    errors.push(`profile must be "macos" or "vitro", got "${input.profile}"`);
  }

  return errors.length === 0 ? { ok: true } : { ok: false, errors };
}
