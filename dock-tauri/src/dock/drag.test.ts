import { describe, expect, it } from "vitest";
import { placeAt, reorderIds, shiftFor, targetIndex } from "./drag";

const ids = ["a", "b", "c", "d"];

describe("reorderIds", () => {
  it("moves an id forward and backward", () => {
    expect(reorderIds(ids, 0, 2)).toEqual(["b", "c", "a", "d"]);
    expect(reorderIds(ids, 3, 1)).toEqual(["a", "d", "b", "c"]);
  });

  it("returns a copy when the index does not change", () => {
    const next = reorderIds(ids, 1, 1);
    expect(next).toEqual(ids);
    expect(next).not.toBe(ids);
  });

  it("clamps a target beyond the list", () => {
    expect(reorderIds(ids, 0, 99)).toEqual(["b", "c", "d", "a"]);
  });
});

describe("placeAt", () => {
  it("inserts an id that is not in the list yet", () => {
    expect(placeAt(ids, "x", 0)).toEqual(["x", "a", "b", "c", "d"]);
    expect(placeAt(ids, "x", 2)).toEqual(["a", "b", "x", "c", "d"]);
    expect(placeAt(ids, "x", 4)).toEqual(["a", "b", "c", "d", "x"]);
  });

  it("moves an existing id and clamps the target", () => {
    expect(placeAt(ids, "a", 2)).toEqual(["b", "c", "a", "d"]);
    expect(placeAt(ids, "a", 99)).toEqual(["b", "c", "d", "a"]);
  });
});

describe("targetIndex", () => {
  const centers = [208, 264, 320, 376];

  it("snaps to the nearest icon centre", () => {
    expect(targetIndex(208, centers)).toBe(0);
    expect(targetIndex(240, centers)).toBe(1);
    expect(targetIndex(310, centers)).toBe(2);
  });

  it("clamps outside the dock", () => {
    expect(targetIndex(-100, centers)).toBe(0);
    expect(targetIndex(900, centers)).toBe(3);
  });

  it("handles an empty or single-tile dock", () => {
    expect(targetIndex(100, [])).toBe(0);
    expect(targetIndex(100, [208])).toBe(0);
  });
});

describe("shiftFor", () => {
  const pitch = 56;

  it("opens a gap when moving forward", () => {
    expect(shiftFor(1, 0, 2, pitch)).toBe(-pitch);
    expect(shiftFor(2, 0, 2, pitch)).toBe(-pitch);
    expect(shiftFor(3, 0, 2, pitch)).toBe(0);
    expect(shiftFor(0, 0, 2, pitch)).toBe(0);
  });

  it("opens a gap when moving backward", () => {
    expect(shiftFor(1, 3, 1, pitch)).toBe(pitch);
    expect(shiftFor(2, 3, 1, pitch)).toBe(pitch);
    expect(shiftFor(0, 3, 1, pitch)).toBe(0);
    expect(shiftFor(3, 3, 1, pitch)).toBe(0);
  });
});
