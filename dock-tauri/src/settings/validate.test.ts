import { describe, expect, it } from "vitest";
import { DEFAULT_SETTINGS, validateSettings, type SettingsInput } from "./validate";

function withChange(change: Partial<SettingsInput>): SettingsInput {
  return { ...DEFAULT_SETTINGS, ...change };
}

describe("validateSettings", () => {
  it("accepts the defaults", () => {
    expect(validateSettings(DEFAULT_SETTINGS).ok).toBe(true);
  });

  it("accepts every documented boundary", () => {
    const edge = withChange({
      iconSize: 32,
      iconGap: 24,
      paddingH: 4,
      paddingV: 24,
      pillRadius: 64,
      iconRadius: 50,
      tooltipDelayMs: 2_000,
      showDelayMs: 1_000,
      hideDelayMs: 100,
      fillOpacity: 0.15,
      reflectionStrength: 1,
      magnifyPeak: 2,
      magnifyRadiusIcons: 5,
    });
    expect(validateSettings(edge).ok).toBe(true);
  });

  it("rejects an icon size below 32", () => {
    const result = validateSettings(withChange({ iconSize: 8 }));
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errors.join(" ")).toContain("iconSize");
  });

  it("rejects an icon size above 80", () => {
    expect(validateSettings(withChange({ iconSize: 200 })).ok).toBe(false);
  });

  it("rejects every integer field outside its range", () => {
    const cases: Array<[keyof SettingsInput, number]> = [
      ["iconGap", 25],
      ["paddingH", 3],
      ["paddingV", 1],
      ["pillRadius", 65],
      ["iconRadius", 51],
      ["tooltipDelayMs", 2_001],
      ["showDelayMs", 1_001],
      ["hideDelayMs", 99],
    ];
    for (const [key, value] of cases) {
      const result = validateSettings(withChange({ [key]: value }));
      expect(result.ok, `${key}=${value}`).toBe(false);
    }
  });

  it("rejects every float field outside its range", () => {
    const cases: Array<[keyof SettingsInput, number]> = [
      ["fillOpacity", 0.1],
      ["reflectionStrength", 1.1],
      ["magnifyPeak", 0.9],
      ["magnifyRadiusIcons", 5.1],
    ];
    for (const [key, value] of cases) {
      const result = validateSettings(withChange({ [key]: value }));
      expect(result.ok, `${key}=${value}`).toBe(false);
    }
  });

  it("rejects a non-integer integer field", () => {
    expect(validateSettings(withChange({ iconSize: 48.5 })).ok).toBe(false);
  });

  it("rejects an invalid tint", () => {
    const result = validateSettings(withChange({ tint: "blue" }));
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errors.join(" ")).toContain("tint");
  });

  it("rejects an unknown profile", () => {
    const result = validateSettings(
      withChange({ profile: "nonsense" as SettingsInput["profile"] }),
    );
    expect(result.ok).toBe(false);
  });
});
