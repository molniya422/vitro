import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ShelfItemView } from "../types";
import { Shelf } from "./Shelf";

const items: ShelfItemView[] = [
  {
    id: "C:\\Users\\me\\report.pdf",
    name: "report.pdf",
    path: "C:\\Users\\me\\report.pdf",
    isDir: false,
    missing: false,
    icon: null,
  },
  {
    id: "C:\\Users\\me\\Projects",
    name: "Projects",
    path: "C:\\Users\\me\\Projects",
    isDir: true,
    missing: false,
    icon: null,
  },
];

function renderShelf(overrides: Partial<Parameters<typeof Shelf>[0]> = {}) {
  return render(
    <Shelf
      items={items}
      loading={false}
      error={null}
      onOpen={vi.fn()}
      onRemove={vi.fn()}
      onClear={vi.fn()}
      onClose={vi.fn()}
      {...overrides}
    />,
  );
}

describe("Shelf", () => {
  it("shows the drop hint when empty", () => {
    renderShelf({ items: [] });
    expect(
      screen.getByText("Drag files here to keep them handy."),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clear" })).toBeDisabled();
  });

  it("lists items and opens the clicked one", () => {
    const onOpen = vi.fn();
    renderShelf({ onOpen });

    fireEvent.click(screen.getByRole("button", { name: "Open report.pdf" }));
    expect(onOpen).toHaveBeenCalledWith("C:\\Users\\me\\report.pdf");
  });

  it("removes and clears through the native callbacks", () => {
    const onRemove = vi.fn();
    const onClear = vi.fn();
    renderShelf({ onRemove, onClear });

    fireEvent.click(screen.getByRole("button", { name: "Remove report.pdf" }));
    expect(onRemove).toHaveBeenCalledWith("C:\\Users\\me\\report.pdf");

    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    expect(onClear).toHaveBeenCalled();
  });

  it("disables an item whose file disappeared", () => {
    renderShelf({
      items: [{ ...items[0], missing: true }],
    });
    expect(
      screen.getByRole("button", { name: "Open report.pdf" }),
    ).toBeDisabled();
    expect(screen.getByText(/Missing —/)).toBeInTheDocument();
  });

  it("shows a load error instead of the list", () => {
    renderShelf({ items: [], error: "boom" });
    expect(screen.getByRole("alert")).toHaveTextContent("boom");
  });
});
