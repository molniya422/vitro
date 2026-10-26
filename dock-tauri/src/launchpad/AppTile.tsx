interface Props {
  name: string;
  icon: string | null;
  disabledReason: string | null;
  onLaunch: () => void;
}

export function AppTile({ name, icon, disabledReason, onLaunch }: Props) {
  return (
    <button
      type="button"
      className="launchpad-tile"
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
  );
}
