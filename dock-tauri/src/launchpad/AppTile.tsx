interface Props {
  name: string;
  icon: string | null;
  disabledReason: string | null;
  pinned?: boolean;
  onPin?: () => void;
  onLaunch: () => void;
}

export function AppTile({
  name,
  icon,
  disabledReason,
  pinned = false,
  onPin,
  onLaunch,
}: Props) {
  return (
    <div className="launchpad-tile">
      <button
        type="button"
        className="launchpad-launch"
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
