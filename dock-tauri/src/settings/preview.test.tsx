import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { DockPreview, previewAppearance } from "./preview";
import { DEFAULT_SETTINGS } from "./validate";
import type { SettingsView } from "../types";

function settings(overrides: Partial<SettingsView> = {}): SettingsView {
  return { ...DEFAULT_SETTINGS, ...overrides };
}

describe("previewAppearance", () => {
  it("derives the values the dock receives from Rust", () => {
    const appearance = previewAppearance(settings());
    expect(appearance.leadingTiles).toBe(4);
    expect(appearance.reflectionSpace).toBe(40);
    expect(appearance.magnifyRadius).toBe(123.2);
  });

  it("drops reflection space and route entries when disabled", () => {
    const appearance = previewAppearance(
      settings({
        reflection: false,
        showLaunchpadEntry: false,
        showMonitorEntry: false,
        showShelfEntry: false,
      }),
    );
    expect(appearance.reflectionSpace).toBe(0);
    expect(appearance.leadingTiles).toBe(1);
  });
});

describe("DockPreview", () => {
  it("applies size, colour, and reflection live", () => {
    const { container } = render(
      <DockPreview
        settings={settings({ iconSize: 64, tint: "#ff0000", fillOpacity: 0.4 })}
      />,
    );
    const preview = screen.getByRole("img", { name: "Dock preview" });
    expect(preview).toBeInTheDocument();

    const pill = container.querySelector<HTMLElement>(".dock-preview-pill");
    expect(pill?.style.getPropertyValue("--icon-size")).toBe("64px");
    expect(pill?.style.getPropertyValue("--glass-fill")).toBe(
      "rgba(255, 0, 0, 0.4)",
    );
    expect(container.querySelector(".dock-preview-reflection")).not.toBeNull();
  });

  it("hides disabled route entries and the reflection", () => {
    const { container } = render(
      <DockPreview
        settings={settings({
          reflection: false,
          showLaunchpadEntry: false,
          showMonitorEntry: false,
          showShelfEntry: false,
        })}
      />,
    );
    expect(container.querySelectorAll(".dock-preview-tile.is-control")).toHaveLength(1);
    expect(container.querySelector(".dock-preview-reflection")).toBeNull();
  });
});
