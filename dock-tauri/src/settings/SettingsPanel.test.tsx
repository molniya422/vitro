import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsPanel } from "./SettingsPanel";
import { api } from "../api";

vi.mock("../api", () => ({
  api: {
    getSettings: vi.fn(),
    saveSettings: vi.fn(),
  },
}));

describe("SettingsPanel", () => {
  beforeEach(() => {
    vi.mocked(api.getSettings).mockResolvedValue({
      schema_version: 1,
      icon_size: 44,
      magnify: true,
      autohide: false,
      reduced_motion: false,
      profile: "vitro",
      show_monitor: false,
      pinned: [],
    });
    vi.mocked(api.saveSettings).mockResolvedValue(undefined);
  });

  it("shows the validation error list when icon size is invalid", async () => {
    render(<SettingsPanel onClose={vi.fn()} />);
    const slider = screen.getByLabelText(/icon size/i) as HTMLInputElement;
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
});
