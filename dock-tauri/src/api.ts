import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DockItemView,
  DockState,
  HitRect,
  MonitorSample,
  SettingsView,
  ShelfItemView,
} from "./types";

export const api = {
  getDockState: () => invoke<DockState>("get_dock_state"),
  getCatalog: () => invoke<DockItemView[]>("get_catalog"),
  pinApp: (id: string) => invoke<void>("pin_app", { id }),
  unpinApp: (id: string) => invoke<void>("unpin_app", { id }),
  reorderPins: (ids: string[]) => invoke<void>("reorder_pins", { ids }),
  launchItem: (id: string) => invoke<void>("launch_item", { id }),
  openRoute: (route: string) => invoke<void>("set_route", { route }),
  getSettings: () => invoke<SettingsView>("get_settings"),
  saveSettings: (settings: SettingsView) =>
    invoke<void>("save_settings", { settings }),
  sampleMonitor: () => invoke<MonitorSample>("sample_monitor"),
  setHitRects: (pillWidth: number, rects: HitRect[], scale: number) =>
    invoke<void>("set_hit_rects", { pillWidth, rects, scale }),
  /** A tile drag owns the pointer: keep the window interactive and visible. */
  setDragActive: (active: boolean) =>
    invoke<void>("set_drag_active", { active }),
  getShelf: () => invoke<ShelfItemView[]>("get_shelf"),
  shelfAdd: (paths: string[]) =>
    invoke<ShelfItemView[]>("shelf_add", { paths }),
  shelfRemove: (path: string) =>
    invoke<ShelfItemView[]>("shelf_remove", { path }),
  shelfClear: () => invoke<ShelfItemView[]>("shelf_clear"),
  openShelfItem: (path: string) =>
    invoke<void>("open_shelf_item", { path }),
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
