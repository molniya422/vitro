import { useEffect, useRef, type RefObject } from "react";

export interface MenuAction {
  label: string;
  onSelect: () => void;
  disabled?: boolean;
}

interface Props {
  x: number;
  y: number;
  actions: MenuAction[];
  onClose: () => void;
  menuRef?: RefObject<HTMLDivElement | null>;
}

/**
 * A right-click menu anchored above the pointer. It lives inside the dock
 * window and is included in the reported hit rects, so its clicks are not
 * passed through to the app underneath.
 */
export function ContextMenu({ x, y, actions, onClose, menuRef }: Props) {
  const localRef = useRef<HTMLDivElement>(null);
  const ref = menuRef ?? localRef;

  useEffect(() => {
    const first = ref.current?.querySelector<HTMLButtonElement>(
      "button:not(:disabled)",
    );
    first?.focus();

    const onPointerDown = (event: PointerEvent) => {
      if (ref.current && event.target instanceof Node && ref.current.contains(event.target)) {
        return;
      }
      onClose();
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    // Register after the opening contextmenu event so it cannot close itself.
    const timer = window.setTimeout(
      () => document.addEventListener("pointerdown", onPointerDown),
      0,
    );
    document.addEventListener("keydown", onKeyDown);
    return () => {
      window.clearTimeout(timer);
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [onClose, ref]);

  return (
    <div
      ref={ref}
      className="dock-menu"
      role="menu"
      aria-label="Dock item menu"
      style={{ left: x, top: y }}
      onContextMenu={(event) => event.preventDefault()}
    >
      {actions.map((action) => (
        <button
          key={action.label}
          type="button"
          role="menuitem"
          disabled={action.disabled}
          onClick={() => {
            action.onSelect();
            onClose();
          }}
        >
          {action.label}
        </button>
      ))}
    </div>
  );
}
