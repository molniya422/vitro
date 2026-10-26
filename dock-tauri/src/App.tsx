import { useEffect, useState } from "react";
import { api, onDockState } from "./api";
import { Dock } from "./dock/Dock";
import { Launchpad } from "./launchpad/Launchpad";
import { MonitorPanel } from "./monitor/MonitorPanel";
import { SettingsPanel } from "./settings/SettingsPanel";
import type { DockState } from "./types";

type Route = "dock" | "launchpad" | "monitor" | "settings";

export function App() {
  const [state, setState] = useState<DockState | null>(null);
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

  // Rust owns the auto-hide state machine. React only reports pointer presence
  // and ticks while a transition is armed, so idle cost is zero.
  useEffect(() => {
    if (route !== "dock") return;

    let timer: number | null = null;
    let cancelled = false;

    function clearTimer() {
      if (timer !== null) {
        window.clearTimeout(timer);
        timer = null;
      }
    }

    function scheduleTick() {
      clearTimer();
      timer = window.setTimeout(() => void tick(), 100);
    }

    async function tick() {
      timer = null;
      if (cancelled) return;
      try {
        const next = await api.autohideTick(Date.now());
        if (!cancelled && next.pending) scheduleTick();
      } catch {
        // Stay in the current visibility and stop polling on error.
      }
    }

    function report(inside: boolean) {
      void api
        .setPointerInside(inside, Date.now())
        .then((next) => {
          if (!cancelled && next.pending) scheduleTick();
        })
        .catch(() => {});
      scheduleTick();
    }

    const enter = () => report(true);
    const leave = () => report(false);

    document.addEventListener("mouseenter", enter);
    document.addEventListener("mouseleave", leave);

    return () => {
      cancelled = true;
      document.removeEventListener("mouseenter", enter);
      document.removeEventListener("mouseleave", leave);
      clearTimer();
    };
  }, [route]);

  const go = (next: Route) => {
    setRoute(next);
    void api.openRoute(next);
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
          onActivate={(id) => void api.launchItem(id)}
        />
      )}
      {route === "launchpad" && (
        <Launchpad
          items={state.items}
          loading={false}
          error={null}
          onLaunch={(id) => void api.launchItem(id)}
          onClose={() => go("dock")}
        />
      )}
      {route === "monitor" && <MonitorPanel onClose={() => go("dock")} />}
      {route === "settings" && <SettingsPanel onClose={() => go("dock")} />}
    </main>
  );
}
