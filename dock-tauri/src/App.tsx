import { useEffect, useState } from "react";
import { api, onDockState } from "./api";
import { Dock } from "./dock/Dock";
import type { DockState } from "./types";

export function App() {
  const [state, setState] = useState<DockState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    api
      .getDockState()
      .then(setState)
      .catch((e) => setError(String(e)));
    onDockState(setState).then((unlisten) => {
      dispose = unlisten;
    });
    return () => dispose?.();
  }, []);

  if (error) {
    return <main className="dock-stage">Dock unavailable: {error}</main>;
  }
  if (!state) {
    return <main className="dock-stage" aria-busy="true" />;
  }
  return (
    <main>
      <Dock state={state} onActivate={(id) => void api.launchItem(id)} />
    </main>
  );
}
