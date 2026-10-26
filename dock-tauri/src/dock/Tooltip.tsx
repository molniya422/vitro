export function Tooltip({ label }: { label: string }) {
  return (
    <div className="dock-tooltip" role="tooltip">
      {label}
    </div>
  );
}
