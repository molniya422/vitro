import { render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { describe, expect, it, vi } from "vitest";
import { Dock } from "./Dock";
import type { DockState } from "../types";

const state: DockState = {
  edge: "bottom",
  autohide: false,
  magnify: true,
  reducedMotion: false,
  profile: "vitro",
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
});
