import { describe, expect, it } from "vitest";
import { magnifiedScale, profileFor, scalesFor } from "./magnify";

describe("magnifiedScale", () => {
  it("returns the peak at zero distance", () => {
    expect(magnifiedScale(0, 1.6, 0.45, 120)).toBeCloseTo(1.6, 5);
  });

  it("returns 1.0 at and beyond the radius", () => {
    expect(magnifiedScale(120, 1.6, 0.45, 120)).toBeCloseTo(1.0, 5);
    expect(magnifiedScale(400, 1.6, 0.45, 120)).toBeCloseTo(1.0, 5);
  });

  it("decays monotonically with distance", () => {
    let previous = Number.POSITIVE_INFINITY;
    for (const distance of [0, 20, 40, 60, 80, 100, 120]) {
      const scale = magnifiedScale(distance, 1.6, 0.45, 120);
      expect(scale).toBeLessThanOrEqual(previous + 1e-9);
      previous = scale;
    }
  });

  it("never goes below 1.0", () => {
    for (let distance = 0; distance <= 300; distance += 7) {
      expect(magnifiedScale(distance, 1.6, 0.45, 120)).toBeGreaterThanOrEqual(1.0);
    }
  });

  it("treats negative distance as zero", () => {
    expect(magnifiedScale(-50, 1.6, 0.45, 120)).toBeCloseTo(1.6, 5);
  });

  it("tolerates a zero radius instead of dividing by zero", () => {
    expect(magnifiedScale(10, 1.6, 0.45, 0)).toBe(1.0);
  });
});

describe("scalesFor", () => {
  const centers = [40, 96, 152, 208];

  it("peaks at the hovered index and falls off to both sides", () => {
    const scales = scalesFor(centers, 96, 1.6, 0.45, 120);
    expect(scales[1]).toBeCloseTo(1.6, 5);
    expect(scales[0]).toBeGreaterThan(1.0);
    expect(scales[2]).toBeGreaterThan(1.0);
    expect(scales[2]).toBeLessThan(scales[1]);
    expect(scales[3]).toBeCloseTo(1.0, 5);
  });

  it("returns all ones when the pointer is far away", () => {
    const scales = scalesFor(centers, 5000, 1.6, 0.45, 120);
    expect(scales.every((s) => Math.abs(s - 1) < 1e-9)).toBe(true);
  });
});

describe("profileFor", () => {
  it("uses a restrained peak for the macOS profile", () => {
    const profile = profileFor("macos", false);
    expect(profile.peak).toBeCloseTo(1.2, 5);
    expect(profile.falloff).toBeGreaterThan(0);
  });

  it("uses the elastic peak for the vitro profile", () => {
    const profile = profileFor("vitro", false);
    expect(profile.peak).toBeCloseTo(1.6, 5);
    expect(profile.falloff).toBeCloseTo(0.45, 5);
  });

  it("disables magnification entirely for reduced motion", () => {
    const profile = profileFor("vitro", true);
    expect(profile.peak).toBeCloseTo(1.0, 5);
  });
});
