import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MonitorPanel } from "./MonitorPanel";
import { api } from "../api";

vi.mock("../api", () => ({
  api: {
    sampleMonitor: vi.fn(),
  },
}));

describe("MonitorPanel", () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("renders unavailable when the sidecar rejects", async () => {
    vi.mocked(api.sampleMonitor).mockRejectedValue(
      new Error("sidecar unavailable"),
    );
    render(<MonitorPanel onClose={vi.fn()} />);

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("sidecar unavailable");
  });
});
