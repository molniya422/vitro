import { act, fireEvent, render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { describe, expect, it, vi } from "vitest";
import { api } from "../api";
import { Dock } from "./Dock";
import type { DockAppearance, DockState } from "../types";

vi.mock("../api", () => ({
  api: {
    setHitRects: vi.fn().mockResolvedValue(undefined),
    unpinApp: vi.fn().mockResolvedValue(undefined),
    reorderPins: vi.fn().mockResolvedValue(undefined),
    setDragActive: vi.fn().mockResolvedValue(undefined),
  },
}));

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

const state: DockState = {
  edge: "bottom",
  appearance,
  items: [
    { id: "a", kind: "app", name: "Idle", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "b", kind: "app", name: "Running", icon: null, running: true, minimized: 0, disabledReason: null },
    { id: "c", kind: "app", name: "Minimized", icon: null, running: true, minimized: 2, disabledReason: null },
    { id: "d", kind: "app", name: "Broken", icon: null, running: false, minimized: 0, disabledReason: "no target" },
  ],
};

function renderDock(overrides: Partial<ComponentProps<typeof Dock>> = {}) {
  return render(
    <Dock
      state={state}
      collapsed={false}
      onToggleCollapsed={vi.fn()}
      onOpenSettings={vi.fn()}
      onOpenLaunchpad={vi.fn()}
      onOpenMonitor={vi.fn()}
      onOpenShelf={vi.fn()}
      onActivate={vi.fn()}
      {...overrides}
    />,
  );
}

describe("Dock", () => {
  it("renders every tile state", () => {
    renderDock();
    expect(screen.getByLabelText("Idle")).toBeInTheDocument();
    expect(screen.getByLabelText("Running, running")).toBeInTheDocument();
    expect(screen.getByLabelText("2 minimized")).toBeInTheDocument();
    const broken = screen.getByLabelText("Broken");
    expect(broken).toBeDisabled();
  });

  it("applies appearance variables and hides disabled route entries", () => {
    const configured: DockState = {
      ...state,
      appearance: {
        ...appearance,
        iconSize: 64,
        showLaunchpadEntry: false,
        showMonitorEntry: false,
        showShelfEntry: false,
        leadingTiles: 1,
      },
    };
    const { container } = renderDock({ state: configured });

    const stage = container.querySelector<HTMLElement>(".dock-stage");
    expect(stage?.style.getPropertyValue("--icon-size")).toBe("64px");
    expect(screen.queryByRole("button", { name: "Open Launchpad" })).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Open system monitor" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Open Shelf" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Dock settings" })).toBeInTheDocument();
  });

  it("opens the shelf from its route tile", () => {
    const onOpenShelf = vi.fn();
    renderDock({ onOpenShelf });
    fireEvent.click(screen.getByRole("button", { name: "Open Shelf" }));
    expect(onOpenShelf).toHaveBeenCalledTimes(1);
  });

  it("opens the item menu on right click and removes a non-system app", async () => {
    renderDock();
    fireEvent.contextMenu(screen.getByLabelText("Idle"), {
      clientX: 120,
      clientY: 320,
    });

    const menu = await screen.findByRole("menu");
    expect(menu).toBeInTheDocument();
    fireEvent.click(screen.getByRole("menuitem", { name: "Remove from Dock" }));

    expect(api.unpinApp).toHaveBeenCalledWith("a");
  });

  it("bounces a tile on launch and clears the pulse", () => {
    vi.useFakeTimers();
    const onActivate = vi.fn();
    renderDock({ onActivate });
    const idle = screen.getByLabelText("Idle");

    fireEvent.click(idle);
    expect(onActivate).toHaveBeenCalledWith("a");
    expect(idle.className).toContain("is-launching");

    act(() => {
      vi.advanceTimersByTime(800);
    });
    expect(idle.className).not.toContain("is-launching");
    vi.useRealTimers();
  });

  it("reorders the dock when a tile is dragged, without launching it", () => {
    const onActivate = vi.fn();
    renderDock({ onActivate });
    const idle = screen.getByLabelText("Idle");

    // The pill starts at x=0 in jsdom, so clientX is pill-local. The centres
    // for four tiles with the default metrics are 264/320/376/432; the grab
    // point is left of the first centre, so the drag offset cancels out.
    fireEvent.pointerDown(idle, { button: 0, clientX: 208, pointerId: 1 });
    fireEvent.pointerMove(window, { clientX: 320, pointerId: 1 });
    fireEvent.pointerUp(window, { clientX: 320, pointerId: 1 });

    expect(api.setDragActive).toHaveBeenNthCalledWith(1, true);
    expect(api.setDragActive).toHaveBeenLastCalledWith(false);
    expect(api.reorderPins).toHaveBeenCalledWith(["b", "c", "a", "d"]);

    // The release is a drop, not a launch.
    fireEvent.click(idle);
    expect(onActivate).not.toHaveBeenCalled();
  });

  it("does not reorder when the drag ends in the same slot", () => {
    renderDock();
    const idle = screen.getByLabelText("Idle");

    fireEvent.pointerDown(idle, { button: 0, clientX: 208, pointerId: 1 });
    fireEvent.pointerMove(window, { clientX: 214, pointerId: 1 });
    fireEvent.pointerUp(window, { clientX: 214, pointerId: 1 });

    expect(api.reorderPins).not.toHaveBeenCalled();
  });

  it("does not offer to remove a system tile", async () => {
    const withSystem: DockState = {
      ...state,
      items: [
        ...state.items,
        {
          id: "shell:explorer",
          kind: "system",
          name: "Explorer",
          icon: null,
          running: false,
          minimized: 0,
          disabledReason: null,
        },
      ],
    };
    renderDock({ state: withSystem });

    fireEvent.contextMenu(screen.getByLabelText("Explorer"), {
      clientX: 120,
      clientY: 320,
    });
    expect(
      await screen.findByRole("menuitem", { name: "Remove from Dock" }),
    ).toBeDisabled();
  });
});
