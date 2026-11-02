interface Props {
  name: string;
  icon: string | null;
  disabledReason: string | null;
  pinned?: boolean;
  /** Unpinned tiles can be dragged onto the dock drop target. */
  draggable?: boolean;
  dragging?: boolean;
  onPointerDown?: (event: React.PointerEvent<HTMLButtonElement>) => void;
  onPin?: () => void;
  onLaunch: () => void;
}

export function AppTile({
  name,
  icon,
  disabledReason,
  pinned = false,
  draggable = false,
  dragging = false,
  onPointerDown,
  onPin,
  onLaunch,
}: Props) {
  const className = [
    "launchpad-launch",
    draggable ? "is-draggable" : "",
    dragging ? "is-dragging" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className="launchpad-tile">
      <button
        type="button"
        className={className}
        onPointerDown={draggable ? onPointerDown : undefined}
        onClick={onLaunch}
        disabled={Boolean(disabledReason)}
        aria-label={name}
        title={disabledReason ?? name}
      >
        {icon ? (
          <img src={icon} alt="" draggable={false} />
        ) : (
          <span className="fallback-icon" aria-hidden="true">
            {name.slice(0, 1).toUpperCase()}
          </span>
        )}
        <span className="launchpad-name">{name}</span>
      </button>
      {onPin && !pinned && (
        <button
          type="button"
          className="launchpad-pin"
          onClick={onPin}
          aria-label={`Add ${name} to dock`}
          title={`Add ${name} to dock`}
        >
          +
        </button>
      )}
    </div>
  );
}
