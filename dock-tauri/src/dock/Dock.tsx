import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { api } from "../api";
import type { DockState, HitRect } from "../types";
import { reorderIds, shiftFor, targetIndex } from "./drag";
import { cssVars, iconCenters } from "./layout";
import { profileFor, scalesFor } from "./magnify";
import { ContextMenu } from "./ContextMenu";
import { ControlTile } from "./ControlTile";
import { DockItem } from "./DockItem";
import { Glyph } from "./Glyph";
import "./dock.css";

/** Pointer travel before a press becomes a drag instead of a click. */
const DRAG_THRESHOLD_PX = 4;
/** Matches the launch bounce in dock.css. */
const LAUNCH_PULSE_MS = 700;

interface Props {
  state: DockState;
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onOpenSettings: () => void;
  onOpenLaunchpad: () => void;
  onOpenMonitor: () => void;
  onOpenShelf: () => void;
  onActivate: (id: string) => void;
}

interface DragState {
  id: string;
  from: number;
  /** Pointer position in pill-local CSS pixels. */
  x: number;
  /** Distance from the grab point to the tile centre, so it does not jump. */
  offset: number;
}

export function Dock({
  state,
  collapsed,
  onToggleCollapsed,
  onOpenSettings,
  onOpenLaunchpad,
  onOpenMonitor,
  onOpenShelf,
  onActivate,
}: Props) {
  const [pointerX, setPointerX] = useState<number | null>(null);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const [menu, setMenu] = useState<{ id: string; x: number; y: number } | null>(
    null,
  );
  const [drag, setDrag] = useState<DragState | null>(null);
  // The reorder is shown immediately; Rust confirms it with a state push.
  const [pendingOrder, setPendingOrder] = useState<string[] | null>(null);
  const [pulseId, setPulseId] = useState<string | null>(null);
  const pulseTimer = useRef<number | null>(null);
  const tooltipTimer = useRef<number | null>(null);
  const pillRef = useRef<HTMLDivElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const press = useRef<{
    id: string;
    index: number;
    startX: number;
    pointerId: number;
  } | null>(null);
  const dragRef = useRef<DragState | null>(null);
  const suppressClick = useRef(false);
  // Acquired on press, not on the 4 px threshold: the watcher polls every
  // 16 ms, and a fast flick into the headroom must not turn the window
  // click-through before the native side knows a gesture owns it.
  const lockHeld = useRef(false);
  const appearance = state.appearance;
  const profile = profileFor(appearance.profile, appearance.reducedMotion);
  const dragging = drag !== null;
  const menuItem = menu
    ? state.items.find((item) => item.id === menu.id) ?? null
    : null;

  // Apply a just-dropped order until the matching state push arrives, so the
  // tiles do not snap back for the duration of the disk write.
  const orderedItems = useMemo(() => {
    if (!pendingOrder) return state.items;
    const byId = new Map(state.items.map((item) => [item.id, item]));
    const next = pendingOrder.flatMap((id) => {
      const item = byId.get(id);
      return item ? [item] : [];
    });
    return next.length === state.items.length ? next : state.items;
  }, [pendingOrder, state.items]);

  useEffect(() => {
    if (!pendingOrder) return;
    const current = state.items.map((item) => item.id);
    const sameSet =
      current.length === pendingOrder.length &&
      current.every((id) => pendingOrder.includes(id));
    if (!sameSet) {
      setPendingOrder(null);
      return;
    }
    if (current.every((id, index) => id === pendingOrder[index])) {
      setPendingOrder(null);
    }
  }, [pendingOrder, state.items]);

  // Centers are pill-local: padding, then every leading tile and icon pitch.
  // Rust derives the same leading count for the window width.
  const centers = useMemo(
    () => iconCenters(orderedItems.length, appearance),
    [appearance, orderedItems.length],
  );

  // The window-level drag listeners are installed once; refs keep them current
  // when a state push lands mid-gesture.
  const centersRef = useRef(centers);
  centersRef.current = centers;
  const itemsRef = useRef(orderedItems);
  itemsRef.current = orderedItems;

  const scales = useMemo(() => {
    if (
      dragging ||
      pointerX === null ||
      !appearance.magnify ||
      appearance.reducedMotion
    ) {
      return orderedItems.map(() => 1);
    }
    return scalesFor(
      centers,
      pointerX,
      appearance.magnifyPeak,
      profile.falloff,
      appearance.magnifyRadius,
    );
  }, [
    appearance.magnify,
    appearance.magnifyPeak,
    appearance.magnifyRadius,
    appearance.reducedMotion,
    centers,
    dragging,
    orderedItems,
    pointerX,
    profile.falloff,
  ]);

  // The transparent window still receives clicks in CSS. Report the pill and
  // any open menu in CSS pixels so the native watcher can pass every other
  // pixel through and scope auto-hide to the pill instead of the whole window.
  const reportHitRects = useCallback(() => {
    const pill = pillRef.current;
    if (!pill) return;
    const pillRect = pill.getBoundingClientRect();
    const rects: HitRect[] = [];
    const menu = menuRef.current;
    if (menu) {
      const rect = menu.getBoundingClientRect();
      rects.push({
        x: rect.left,
        y: rect.top,
        width: rect.width,
        height: rect.height,
      });
    }
    void api
      .setHitRects(pillRect.width, rects, window.devicePixelRatio || 1)
      .catch(() => {});
  }, []);
  useLayoutEffect(() => {
    reportHitRects();
  }, [menu, reportHitRects]);

  useLayoutEffect(() => {
    reportHitRects();
    const pill = pillRef.current;
    const Observer = window.ResizeObserver;
    const observer = Observer ? new Observer(reportHitRects) : null;
    if (pill && observer) observer.observe(pill);
    window.addEventListener("resize", reportHitRects);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", reportHitRects);
    };
  }, [
    appearance.iconGap,
    appearance.iconSize,
    appearance.leadingTiles,
    appearance.paddingH,
    appearance.paddingV,
    appearance.showLaunchpadEntry,
    appearance.showMonitorEntry,
    appearance.showShelfEntry,
    collapsed,
    orderedItems.length,
    reportHitRects,
  ]);

  const handlePointerMove = useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    // Measure against the pill, not the stage: the icon centers are pill-local,
    // and the stage is wider than the pill whenever the window is not exactly
    // pill-sized (a browser, or a window mid-route-change).
    const pill = pillRef.current;
    if (!pill) return;
    setPointerX(event.clientX - pill.getBoundingClientRect().left);
  }, []);

  const handleHover = useCallback(
    (id: string | null) => {
      if (dragRef.current) return;
      if (tooltipTimer.current !== null) {
        window.clearTimeout(tooltipTimer.current);
        tooltipTimer.current = null;
      }
      if (id === null || !appearance.tooltips) {
        setHoveredId(null);
        return;
      }
      tooltipTimer.current = window.setTimeout(
        () => setHoveredId(id),
        appearance.tooltipDelayMs,
      );
    },
    [appearance.tooltipDelayMs, appearance.tooltips],
  );

  useEffect(
    () => () => {
      if (tooltipTimer.current !== null) {
        window.clearTimeout(tooltipTimer.current);
      }
      if (pulseTimer.current !== null) {
        window.clearTimeout(pulseTimer.current);
      }
    },
    [],
  );

  useEffect(() => {
    if (menu && !menuItem) setMenu(null);
  }, [menu, menuItem]);

  const openMenu = useCallback((id: string, x: number, y: number) => {
    if (dragRef.current) return;
    if (tooltipTimer.current !== null) {
      window.clearTimeout(tooltipTimer.current);
      tooltipTimer.current = null;
    }
    setHoveredId(null);
    setMenu({ id, x, y });
  }, []);

  const startPress = useCallback(
    (id: string, index: number, event: React.PointerEvent<HTMLButtonElement>) => {
      if (event.button !== 0) return;
      const pill = pillRef.current;
      if (!pill) return;
      // A fresh press always clears the stale suppression from a previous drag.
      suppressClick.current = false;
      press.current = {
        id,
        index,
        startX: event.clientX - pill.getBoundingClientRect().left,
        pointerId: event.pointerId,
      };
      if (!lockHeld.current) {
        lockHeld.current = true;
        void api.setDragActive(true).catch(() => {
          lockHeld.current = false;
        });
      }
    },
    [],
  );

  const activate = useCallback(
    (id: string) => {
      if (suppressClick.current) {
        suppressClick.current = false;
        return;
      }
      // A single bounce per click; reduced motion suppresses it in CSS.
      if (pulseTimer.current !== null) {
        window.clearTimeout(pulseTimer.current);
      }
      setPulseId(id);
      pulseTimer.current = window.setTimeout(() => {
        pulseTimer.current = null;
        setPulseId(null);
      }, LAUNCH_PULSE_MS);
      onActivate(id);
    },
    [onActivate],
  );

  // The gesture lives on the window so it survives leaving the tile. The native
  // side is told about it as well: auto-hide must not collapse the dock and the
  // window must stay interactive while a tile follows the pointer.
  useEffect(() => {
    const localX = (clientX: number) => {
      const pill = pillRef.current;
      return pill ? clientX - pill.getBoundingClientRect().left : clientX;
    };

    const clearTooltip = () => {
      if (tooltipTimer.current !== null) {
        window.clearTimeout(tooltipTimer.current);
        tooltipTimer.current = null;
      }
    };

    const move = (event: PointerEvent) => {
      const start = press.current;
      if (!start || event.pointerId !== start.pointerId) return;
      const x = localX(event.clientX);
      if (!dragRef.current) {
        if (Math.abs(x - start.startX) < DRAG_THRESHOLD_PX) return;
        clearTooltip();
        setHoveredId(null);
        const next: DragState = {
          id: start.id,
          from: start.index,
          x,
          offset: start.startX - centersRef.current[start.index],
        };
        dragRef.current = next;
        // Releasing a drag is a drop, never a launch.
        suppressClick.current = true;
        setDrag(next);
        return;
      }
      const next = { ...dragRef.current, x };
      dragRef.current = next;
      setDrag(next);
    };

    const releaseLock = () => {
      if (!lockHeld.current) return;
      lockHeld.current = false;
      void api.setDragActive(false).catch(() => {});
    };

    const finish = (commit: boolean) => {
      const current = dragRef.current;
      press.current = null;
      dragRef.current = null;
      setDrag(null);
      releaseLock();
      if (!current || !commit) return;
      const projected = current.x - current.offset;
      const to = targetIndex(projected, centersRef.current);
      if (to === current.from) return;
      const next = reorderIds(
        itemsRef.current.map((item) => item.id),
        current.from,
        to,
      );
      setPendingOrder(next);
      void api.reorderPins(next).catch(() => setPendingOrder(null));
    };

    const up = (event: PointerEvent) => {
      if (!press.current || event.pointerId !== press.current.pointerId) return;
      finish(true);
    };
    const cancel = (event: PointerEvent) => {
      if (!press.current || event.pointerId !== press.current.pointerId) return;
      finish(false);
    };
    // Losing focus (alt-tab, the pointer leaving the window mid-drag) must not
    // leave the dock locked or a tile stuck to the cursor.
    const blur = () => {
      if (press.current) finish(false);
    };

    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    window.addEventListener("pointercancel", cancel);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      window.removeEventListener("pointercancel", cancel);
      window.removeEventListener("blur", blur);
      if (lockHeld.current) {
        lockHeld.current = false;
        void api.setDragActive(false).catch(() => {});
      }
    };
  }, []);

  const pitch = appearance.iconSize + appearance.iconGap;
  const projected = drag ? drag.x - drag.offset : 0;
  const dropTo = drag ? targetIndex(projected, centers) : 0;

  return (
    <div
      className="dock-stage"
      style={cssVars(appearance)}
      onPointerMove={handlePointerMove}
      onPointerLeave={() => setPointerX(null)}
    >
      <div
        ref={pillRef}
        className={`dock-pill${collapsed ? " is-collapsed" : ""}`}
        role="toolbar"
        aria-label="Dock"
      >
        <ControlTile
          collapsed={collapsed}
          onSettings={onOpenSettings}
          onToggleCollapsed={onToggleCollapsed}
        />
        {appearance.showLaunchpadEntry && (
          <button
            type="button"
            className="dock-item route-entry"
            aria-label="Open Launchpad"
            onClick={onOpenLaunchpad}
          >
            <span className="app-icon system-icon">
              <Glyph name="grid" />
            </span>
          </button>
        )}
        {appearance.showMonitorEntry && (
          <button
            type="button"
            className="dock-item route-entry"
            aria-label="Open system monitor"
            onClick={onOpenMonitor}
          >
            <span className="app-icon system-icon">
              <Glyph name="chart" />
            </span>
          </button>
        )}
        {appearance.showShelfEntry && (
          <button
            type="button"
            className="dock-item route-entry"
            aria-label="Open Shelf"
            onClick={onOpenShelf}
          >
            <span className="app-icon system-icon">
              <Glyph name="tray" />
            </span>
          </button>
        )}
        {orderedItems.map((item, index) => {
          const offsetX = drag
            ? index === drag.from
              ? projected - centers[index]
              : shiftFor(index, drag.from, dropTo, pitch)
            : 0;
          return (
            <DockItem
              key={item.id}
              item={item}
              appearance={appearance}
              scale={scales[index]}
              magnifying={!dragging && pointerX !== null && scales[index] > 1.001}
              reducedMotion={appearance.reducedMotion}
              showTooltip={!dragging && hoveredId === item.id}
              offsetX={offsetX}
              dragging={drag?.id === item.id}
              launching={pulseId === item.id}
              onPointerDown={(event) => startPress(item.id, index, event)}
              onHover={handleHover}
              onActivate={activate}
              onContextMenu={openMenu}
            />
          );
        })}
      </div>
      {menu && menuItem && (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          menuRef={menuRef}
          onClose={() => setMenu(null)}
          actions={[
            {
              label: "Open",
              onSelect: () => activate(menuItem.id),
              disabled: Boolean(menuItem.disabledReason),
            },
            {
              label: "Remove from Dock",
              onSelect: () => {
                void api.unpinApp(menuItem.id).catch(() => {});
              },
              disabled: menuItem.kind === "system",
            },
          ]}
        />
      )}
    </div>
  );
}
