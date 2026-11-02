import { describe, expect, it } from "vitest";
import type { DockAppearance } from "../types";
import { cssVars, hexToRgba, iconCenters } from "./layout";

const appearance: DockAppearance = {
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
  reflectionSpace: 40,
  magnify: true,
  magnifyPeak: 1.6,
  magnifyRadius: 123.2,
  reducedMotion: false,
  profile: "vitro",
  tooltips: true,
  tooltipDelayMs: 500,
  showRunningDots: true,
  showBadges: true,
  showLaunchpadEntry: true,
  showMonitorEntry: true,
  showShelfEntry: true,
  leadingTiles: 4,
};

describe("cssVars", () => {
  it("maps appearance to the dock's CSS variables", () => {
    const vars = cssVars(appearance);
    expect(vars["--icon-size"]).toBe("48px");
    expect(vars["--icon-gap"]).toBe("8px");
    expect(vars["--pad-h"]).toBe("16px");
    expect(vars["--pad-v"]).toBe("10px");
    expect(vars["--pill-radius"]).toBe("34px");
    expect(vars["--icon-radius"]).toBe("22%");
    expect(vars["--reflection-space"]).toBe("40px");
    expect(vars["--reflection-opacity"]).toBe("0.25");
    expect(vars["--glass-fill"]).toBe("rgba(22, 26, 38, 0.62)");
  });
});

describe("iconCenters", () => {
  it("starts after the leading tiles and follows the pitch", () => {
    expect(iconCenters(3, appearance)).toEqual([264, 320, 376]);
  });

  it("tracks a different size, gap, padding, and leading count", () => {
    const larger: DockAppearance = {
      ...appearance,
      iconSize: 64,
      iconGap: 12,
      paddingH: 10,
      leadingTiles: 2,
    };
    expect(iconCenters(2, larger)).toEqual([194, 270]);
  });

  it("returns no centers for an empty dock", () => {
    expect(iconCenters(0, appearance)).toEqual([]);
  });
});

describe("hexToRgba", () => {
  it("converts a six-digit colour", () => {
    expect(hexToRgba("#7fa4ff", 0.5)).toBe("rgba(127, 164, 255, 0.5)");
  });

  it("falls back to the default tint for invalid input", () => {
    expect(hexToRgba("blue", 0.62)).toBe("rgba(22, 26, 38, 0.62)");
  });

  it("clamps the alpha", () => {
    expect(hexToRgba("#000000", 5)).toBe("rgba(0, 0, 0, 1)");
    expect(hexToRgba("#000000", -1)).toBe("rgba(0, 0, 0, 0)");
  });
});
