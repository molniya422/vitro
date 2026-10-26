import { describe, expect, it } from "vitest";
import { validateSettings } from "./validate";

describe("validateSettings", () => {
  it("accepts the defaults", () => {
    const result = validateSettings({
      iconSize: 48,
      magnify: true,
      autohide: false,
      reducedMotion: false,
      profile: "vitro",
      showMonitor: false,
    });
    expect(result.ok).toBe(true);
  });

  it("rejects an icon size below 16", () => {
    const result = validateSettings({
      iconSize: 8,
      magnify: true,
      autohide: false,
      reducedMotion: false,
      profile: "vitro",
      showMonitor: false,
    });
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errors.join(" ")).toContain("iconSize");
  });

  it("rejects an icon size above 128", () => {
    const result = validateSettings({
      iconSize: 200,
      magnify: true,
      autohide: false,
      reducedMotion: false,
      profile: "vitro",
      showMonitor: false,
    });
    expect(result.ok).toBe(false);
  });

  it("rejects an unknown profile", () => {
    const result = validateSettings({
      iconSize: 48,
      magnify: true,
      autohide: false,
      reducedMotion: false,
      profile: "nonsense",
      showMonitor: false,
    });
    expect(result.ok).toBe(false);
  });
});
