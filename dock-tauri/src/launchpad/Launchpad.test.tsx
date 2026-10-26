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
});
