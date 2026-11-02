import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Launchpad } from "./Launchpad";
import type { DockItemView } from "../types";

const items: DockItemView[] = [
  { id: "a", kind: "app", name: "Calculator", icon: null, running: false, minimized: 0, disabledReason: null },
  { id: "b", kind: "app", name: "Paint", icon: null, running: false, minimized: 0, disabledReason: null },
];

describe("Launchpad", () => {
  it("shows the empty-results message when the query matches nothing", () => {
    render(
      <Launchpad
        items={items}
        loading={false}
        error={null}
        onLaunch={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByRole("searchbox"), {
      target: { value: "zzzz" },
    });

    expect(screen.getByText(/No applications match/)).toHaveTextContent("zzzz");
    expect(
      screen.queryByRole("button", { name: "Calculator" }),
    ).not.toBeInTheDocument();
  });

  it("drags an app onto the dock strip without launching it", () => {
    const onDropPin = vi.fn().mockResolvedValue(undefined);
    const onLaunch = vi.fn();
    render(
      <Launchpad
        items={items}
        loading={false}
        error={null}
        dockItems={[items[0]]}
        onDropPin={onDropPin}
        onLaunch={onLaunch}
        onClose={vi.fn()}
      />,
    );

    const tile = screen.getByRole("button", { name: "Paint" });
    fireEvent.pointerDown(tile, {
      button: 0,
      clientX: 100,
      clientY: 100,
      pointerId: 1,
    });
    // jsdom's window is 768 px tall; the drop band is the bottom 140 px.
    fireEvent.pointerMove(window, {
      clientX: 140,
      clientY: 740,
      pointerId: 1,
    });

    expect(
      screen.getByRole("group", { name: "Dock drop target" }),
    ).toBeInTheDocument();

    fireEvent.pointerUp(window, { clientX: 140, clientY: 740, pointerId: 1 });

    expect(onDropPin).toHaveBeenCalledTimes(1);
    expect(onDropPin.mock.calls[0][0]).toBe("b");

    // The release was a drop, so the tile click must not launch the app.
    fireEvent.click(tile);
    expect(onLaunch).not.toHaveBeenCalled();
  });

  it("cancels the drop outside the bottom band", () => {
    const onDropPin = vi.fn().mockResolvedValue(undefined);
    render(
      <Launchpad
        items={items}
        loading={false}
        error={null}
        dockItems={[items[0]]}
        onDropPin={onDropPin}
        onLaunch={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    const tile = screen.getByRole("button", { name: "Paint" });
    fireEvent.pointerDown(tile, {
      button: 0,
      clientX: 100,
      clientY: 100,
      pointerId: 1,
    });
    fireEvent.pointerMove(window, { clientX: 120, clientY: 300, pointerId: 1 });
    fireEvent.pointerUp(window, { clientX: 120, clientY: 300, pointerId: 1 });

    expect(onDropPin).not.toHaveBeenCalled();
  });
});
