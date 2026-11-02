import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsPanel } from "./SettingsPanel";
import { api } from "../api";
import { DEFAULT_SETTINGS } from "./validate";
import { previewAppearance } from "./preview";
import type { DockItemView, DockState, SettingsView } from "../types";

vi.mock("../api", () => ({
  api: {
    getSettings: vi.fn(),
    saveSettings: vi.fn(),
    getDockState: vi.fn(),
    getCatalog: vi.fn(),
    pinApp: vi.fn(),
    unpinApp: vi.fn(),
    reorderPins: vi.fn(),
  },
}));

const view: SettingsView = {
  ...DEFAULT_SETTINGS,
  iconSize: 44,
  fillOpacity: 0.72,
  tint: "#101828",
  showDelayMs: 250,
};

const dockItems: DockItemView[] = [
  {
    id: "idle-app",
    kind: "app",
    name: "Idle App",
    icon: null,
    running: false,
    minimized: 0,
    disabledReason: null,
  },
];

const dockState: DockState = {
  items: dockItems,
  edge: "bottom",
  appearance: previewAppearance(view),
};

const catalog: DockItemView[] = [
  {
    id: "cat-alpha",
    kind: "app",
    name: "Alpha Player",
    icon: null,
    running: false,
    minimized: 0,
    disabledReason: null,
  },
  {
    id: "cat-bravo",
    kind: "app",
    name: "Bravo Editor",
    icon: null,
    running: false,
    minimized: 0,
    disabledReason: null,
  },
];

describe("SettingsPanel", () => {
  beforeEach(() => {
    vi.mocked(api.getSettings).mockResolvedValue(view);
    vi.mocked(api.getDockState).mockResolvedValue(dockState);
    vi.mocked(api.getCatalog).mockResolvedValue(catalog);
    vi.mocked(api.saveSettings).mockResolvedValue(undefined);
    vi.mocked(api.pinApp).mockResolvedValue(undefined);
    vi.mocked(api.unpinApp).mockResolvedValue(undefined);
    vi.mocked(api.reorderPins).mockResolvedValue(undefined);
  });

  it("shows the validation error list when icon size is invalid", async () => {
    render(<SettingsPanel onClose={vi.fn()} />);
    const slider = (await screen.findByLabelText(/icon size/i)) as HTMLInputElement;
    await waitFor(() => expect(slider.value).toBe("44"));

    // jsdom clamps range values to the element's min, so widen it first to
    // drive an out-of-range value through the UI.
    slider.min = "0";
    fireEvent.change(slider, { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("iconSize");
    expect(api.saveSettings).not.toHaveBeenCalled();
  });

  it("saves the complete view, including fields edited on other tabs", async () => {
    render(<SettingsPanel onClose={vi.fn()} />);
    const slider = (await screen.findByLabelText(/icon size/i)) as HTMLInputElement;
    await waitFor(() => expect(slider.value).toBe("44"));

    fireEvent.change(slider, { target: { value: "46" } });
    await screen.findByText("46 px");
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(api.saveSettings).toHaveBeenCalled());
    const payload = vi.mocked(api.saveSettings).mock.calls[0][0];
    expect(payload.iconSize).toBe(46);
    expect(payload.fillOpacity).toBe(0.72);
    expect(payload.tint).toBe("#101828");
    expect(payload.showDelayMs).toBe(250);
    expect(payload.yieldFullscreen).toBe(true);
    expect(payload.showMonitorEntry).toBe(true);
  });

  it("switches panels through the tabs", async () => {
    render(<SettingsPanel onClose={vi.fn()} />);
    await screen.findByLabelText(/icon size/i);

    fireEvent.click(screen.getByRole("tab", { name: "Behaviour" }));
    expect(screen.getByRole("slider", { name: /magnify peak/i })).toBeInTheDocument();
    expect(screen.queryByRole("slider", { name: /icon size/i })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("tab", { name: "Apps" }));
    expect(await screen.findByRole("heading", { name: "Add an app" })).toBeInTheDocument();
  });

  it("adds a catalog app from the Apps tab", async () => {
    render(<SettingsPanel onClose={vi.fn()} />);
    await screen.findByLabelText(/icon size/i);

    fireEvent.click(screen.getByRole("tab", { name: "Apps" }));
    const add = await screen.findByRole("button", { name: "Add Alpha Player" });
    fireEvent.click(add);

    await waitFor(() => expect(api.pinApp).toHaveBeenCalledWith("cat-alpha"));
  });
});
