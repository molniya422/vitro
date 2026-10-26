import { describe, expect, it } from "vitest";
import { formatBytes, formatPercent } from "./format";

describe("formatBytes", () => {
  it("uses GB for large values", () => {
    expect(formatBytes(8 * 1024 ** 3)).toBe("8.0 GB");
  });

  it("uses MB for small values", () => {
    expect(formatBytes(512 * 1024 ** 2)).toBe("512 MB");
  });

  it("handles zero", () => {
    expect(formatBytes(0)).toBe("0 MB");
  });
});

describe("formatPercent", () => {
  it("rounds to one decimal", () => {
    expect(formatPercent(12.34)).toBe("12.3%");
  });

  it("clamps above 100", () => {
    expect(formatPercent(140)).toBe("100.0%");
  });

  it("clamps below 0", () => {
    expect(formatPercent(-5)).toBe("0.0%");
  });
});
