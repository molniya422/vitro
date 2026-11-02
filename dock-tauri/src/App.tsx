import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { api, onDockState } from "./api";
import { Dock } from "./dock/Dock";
import { placeAt } from "./dock/drag";
import { Launchpad } from "./launchpad/Launchpad";
import { MonitorPanel } from "./monitor/MonitorPanel";
import { SettingsPanel } from "./settings/SettingsPanel";
import { Shelf } from "./shelf/Shelf";
import type { DockItemView, DockState, ShelfItemView } from "./types";

type Route = "dock" | "launchpad" | "monitor" | "settings" | "shelf";

export function App() {
  const [state, setState] = useState<DockState | null>(null);
  const [catalog, setCatalog] = useState<DockItemView[] | null>(null);
  const [catalogError, setCatalogError] = useState<string | null>(null);
  const [shelf, setShelf] = useState<ShelfItemView[] | null>(null);
  const [shelfError, setShelfError] = useState<string | null>(null);
  const [route, setRoute] = useState<Route>("dock");
  const [collapsed, setCollapsed] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    api
      .getDockState()
      .then(setState)
      .catch((e) => setError(String(e)));
    void onDockState(setState).then((unlisten) => {
      dispose = unlisten;
    });
    return () => dispose?.();
  }, []);

  // Files dropped anywhere on the window (the pill, or an open panel) land on
  // the shelf. The window only receives drops where it is interactive, which is
  // exactly the visible pill / panel.
  useEffect(() => {
    let dispose: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type !== "drop" || event.payload.paths.length === 0) {
          return;
        }
        void api
          .shelfAdd(event.payload.paths)
          .then((items) => {
            setShelf(items);
            setShelfError(null);
          })
          .catch((e) => setShelfError(String(e)))
          .finally(() => {
            setRoute("shelf");
            void api.openRoute("shelf");
          });
      })
      .then((unlisten) => {
        dispose = unlisten;
      })
      .catch(() => {});
    return () => dispose?.();
  }, []);

  // The catalog is expensive (one icon per installed app). Fetch it only when
  // Launchpad actually opens; the dock's own state arrives separately.
  useEffect(() => {
    if (route !== "launchpad" || catalog !== null || catalogError !== null) return;
    let cancelled = false;
    api
      .getCatalog()
      .then((items) => {
        if (!cancelled) setCatalog(items);
      })
      .catch((e) => {
        if (!cancelled) setCatalogError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [catalog, catalogError, route]);

  // Opening the shelf from the dock tile loads the current list once.
  useEffect(() => {
    if (route !== "shelf" || shelf !== null) return;
    let cancelled = false;
    api
      .getShelf()
      .then((items) => {
        if (!cancelled) setShelf(items);
      })
      .catch((e) => {
        if (!cancelled) setShelfError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [route, shelf]);

  const go = (next: Route) => {
    setRoute(next);
    void api.openRoute(next);
  };

  const addToDock = async (id: string) => {
    await api.pinApp(id);
    setState(await api.getDockState());
  };

  // Dropping from Launchpad inserts at the gap the pointer was over. An app
  // that is already pinned is moved instead of appended and then moved.
  const placeInDock = async (id: string, index: number) => {
    if (!state) return;
    const ids = state.items.map((item) => item.id);
    if (ids.includes(id)) {
      await api.reorderPins(placeAt(ids, id, index));
    } else {
      await api.pinApp(id);
      await api.reorderPins(placeAt(ids, id, index));
    }
    setState(await api.getDockState());
  };

  const openShelfItem = async (path: string) => {
    try {
      await api.openShelfItem(path);
    } catch (e) {
      setShelfError(String(e));
    }
  };

  const removeShelfItem = async (path: string) => {
    try {
      setShelf(await api.shelfRemove(path));
    } catch (e) {
      setShelfError(String(e));
    }
  };

  const clearShelf = async () => {
    try {
      setShelf(await api.shelfClear());
    } catch (e) {
      setShelfError(String(e));
    }
  };

  if (error) {
    return (
      <main className="dock-stage" role="alert">
        Dock unavailable: {error}
      </main>
    );
  }
  if (!state) {
    return <main className="dock-stage" aria-busy="true" />;
  }

  return (
    <main>
      {route === "dock" && (
        <Dock
          state={state}
          collapsed={collapsed}
          onToggleCollapsed={() => setCollapsed((value) => !value)}
          onOpenSettings={() => go("settings")}
          onOpenLaunchpad={() => go("launchpad")}
          onOpenMonitor={() => go("monitor")}
          onOpenShelf={() => go("shelf")}
          onActivate={(id) => void api.launchItem(id)}
        />
      )}
      {route === "launchpad" && (
        <Launchpad
          items={catalog ?? []}
          loading={catalog === null && catalogError === null}
          error={catalogError}
          dockItems={state.items}
          onPin={addToDock}
          onDropPin={placeInDock}
          onLaunch={(id) => void api.launchItem(id)}
          onClose={() => go("dock")}
        />
      )}
      {route === "monitor" && <MonitorPanel onClose={() => go("dock")} />}
      {route === "settings" && <SettingsPanel onClose={() => go("dock")} />}
      {route === "shelf" && (
        <Shelf
          items={shelf ?? []}
          loading={shelf === null && shelfError === null}
          error={shelfError}
          onOpen={(path) => void openShelfItem(path)}
          onRemove={(path) => void removeShelfItem(path)}
          onClear={() => void clearShelf()}
          onClose={() => go("dock")}
        />
      )}
    </main>
  );
}
