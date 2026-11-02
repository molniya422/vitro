import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ControlTile } from "./ControlTile";

describe("ControlTile", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("opens settings on a single click", () => {
    const onSettings = vi.fn();
    const onToggle = vi.fn();
    render(
      <ControlTile
        collapsed={false}
        onSettings={onSettings}
        onToggleCollapsed={onToggle}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /dock settings/i }));
    expect(onSettings).not.toHaveBeenCalled();
    act(() => vi.advanceTimersByTime(250));
    expect(onSettings).toHaveBeenCalledTimes(1);
    expect(onToggle).not.toHaveBeenCalled();
  });

  it("collapses on a double click without opening settings", () => {
    const onSettings = vi.fn();
    const onToggle = vi.fn();
    render(
      <ControlTile
        collapsed={false}
        onSettings={onSettings}
        onToggleCollapsed={onToggle}
      />,
    );
    const tile = screen.getByRole("button", { name: /dock settings/i });
    fireEvent.click(tile);
    act(() => vi.advanceTimersByTime(100));
    fireEvent.click(tile);
    act(() => vi.advanceTimersByTime(500));
    expect(onToggle).toHaveBeenCalledTimes(1);
    expect(onSettings).not.toHaveBeenCalled();
  });

  it("exposes the collapsed state to assistive tech", () => {
    render(
      <ControlTile collapsed onSettings={() => {}} onToggleCollapsed={() => {}} />,
    );
    expect(
      screen.getByRole("button", { name: /dock settings/i }),
    ).toHaveAttribute("aria-expanded", "false");
  });
});
