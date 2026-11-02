import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DockItem } from "./DockItem";
import type { DockAppearance, DockItemView } from "../types";

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

const item: DockItemView = {
  id: "a",
  kind: "app",
  name: "Alpha",
  icon: "data:image/png;base64,AA==",
  running: true,
  minimized: 2,
  disabledReason: null,
};

function renderItem(configured: DockAppearance) {
  return render(
    <DockItem
      item={item}
      appearance={configured}
      scale={1}
      magnifying={false}
      reducedMotion={false}
      showTooltip={false}
      onHover={vi.fn()}
      onActivate={vi.fn()}
      onContextMenu={vi.fn()}
    />,
  );
}

describe("DockItem", () => {
  it("renders the reflection, running dot, and badge when enabled", () => {
    const { container } = renderItem(appearance);
    expect(container.querySelector(".reflection")).not.toBeNull();
    expect(container.querySelector(".running-dot")).not.toBeNull();
    expect(container.querySelector(".badge")).not.toBeNull();
  });

  it("marks a tile as launching for the bounce", () => {
    const { container } = render(
      <DockItem
        item={item}
        appearance={appearance}
        scale={1}
        magnifying={false}
        reducedMotion={false}
        showTooltip={false}
        launching
        onHover={vi.fn()}
        onActivate={vi.fn()}
        onContextMenu={vi.fn()}
      />,
    );
    expect(container.querySelector(".dock-item.is-launching")).not.toBeNull();
  });

  it("honours the appearance toggles", () => {
    const { container } = renderItem({
      ...appearance,
      reflection: false,
      showRunningDots: false,
      showBadges: false,
    });
    expect(container.querySelector(".reflection")).toBeNull();
    expect(container.querySelector(".running-dot")).toBeNull();
    expect(container.querySelector(".badge")).toBeNull();
  });
});
