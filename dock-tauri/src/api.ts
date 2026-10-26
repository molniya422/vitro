import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AutoHideState, DockState, MonitorSample } from "./types";

export const api = {
  getDockState: () => invoke<DockState>("get_dock_state"),
  launchItem: (id: string) => invoke<void>("launch_item", { id }),
  openRoute: (route: string) => invoke<void>("set_route", { route }),
  getSettings: () => invoke<Record<string, unknown>>("get_settings"),
  saveSettings: (settings: Record<string, unknown>) =>
    invoke<void>("save_settings", { settings }),
  sampleMonitor: () => invoke<MonitorSample>("sample_monitor"),
  setPointerInside: (inside: boolean, nowMs: number) =>
    invoke<AutoHideState>("set_pointer_inside", { inside, nowMs }),
  autohideTick: (nowMs: number) =>
    invoke<AutoHideState>("autohide_tick", { nowMs }),
  setAutohideEnabled: (enabled: boolean) =>
    invoke<AutoHideState>("set_autohide_enabled", { enabled }),
};

export function onDockState(handler: (state: DockState) => void): Promise<UnlistenFn> {
  return listen<DockState>("dock://state", (event) => handler(event.payload));
}

export function onMonitorStatus(
  handler: (status: { available: boolean; detail: string | null }) => void,
): Promise<UnlistenFn> {
  return listen("monitor://status", (event) =>
    handler(event.payload as { available: boolean; detail: string | null }),
  );
}
