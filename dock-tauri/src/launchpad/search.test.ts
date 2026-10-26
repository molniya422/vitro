import { describe, expect, it } from "vitest";
import { rankApps } from "./search";

const apps = [
  { id: "a", name: "Calculator" },
  { id: "b", name: "Calendar" },
  { id: "c", name: "Paint" },
  { id: "d", name: "Notepad" },
];

describe("rankApps", () => {
  it("returns everything for an empty query, alphabetically", () => {
    const result = rankApps(apps, "");
    expect(result.map((a) => a.name)).toEqual([
      "Calculator",
      "Calendar",
      "Notepad",
      "Paint",
    ]);
  });

  it("prefix matches rank above substring matches", () => {
    const result = rankApps(apps, "cal");
    expect(result[0].name).toBe("Calculator");
    expect(result[1].name).toBe("Calendar");
  });

  it("is case insensitive", () => {
    expect(rankApps(apps, "PAI")[0].name).toBe("Paint");
  });

  it("returns an empty list when nothing matches", () => {
    expect(rankApps(apps, "zzzz")).toEqual([]);
  });

  it("tolerates a leading and trailing space", () => {
    expect(rankApps(apps, "  paint  ")[0].name).toBe("Paint");
  });

  it("does not mutate its input", () => {
    const copy = [...apps];
    rankApps(apps, "cal");
    expect(apps).toEqual(copy);
  });
});
