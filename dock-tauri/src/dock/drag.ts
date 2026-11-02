/**
 * Pure math for drag-to-reorder.
 *
 * The DOM order never changes during a drag: the lifted tile follows the
 * pointer with a translate, and the tiles between the old and new slot shift by
 * one pitch so the gap opens up. Keeping this out of the component makes the
 * index math testable without a layout.
 */

/** Move the id at `from` to `to`, returning a new array. */
export function reorderIds(ids: string[], from: number, to: number): string[] {
  if (from === to || from < 0 || from >= ids.length) return ids.slice();
  const next = ids.slice();
  const [moved] = next.splice(from, 1);
  next.splice(Math.min(Math.max(to, 0), next.length), 0, moved);
  return next;
}

/**
 * Insert `id` at `index`, or move it there when it is already present.
 *
 * Drops from Launchpad use a gap index, so the dragged app may not be in the
 * list yet. Clamping keeps a drop past the last tile at the end.
 */
export function placeAt(ids: string[], id: string, index: number): string[] {
  const rest = ids.filter((existing) => existing !== id);
  const at = Math.min(Math.max(index, 0), rest.length);
  return [...rest.slice(0, at), id, ...rest.slice(at)];
}

/** Slot the dragged tile would occupy with the pointer at `pointerX`. */
export function targetIndex(pointerX: number, centers: number[]): number {
  if (centers.length === 0) return 0;
  if (centers.length === 1) return 0;
  const pitch = centers[1] - centers[0];
  if (!(pitch > 0)) return 0;
  const raw = Math.round((pointerX - centers[0]) / pitch);
  return Math.min(centers.length - 1, Math.max(0, raw));
}

/**
 * Horizontal offset for a tile that is not being dragged.
 *
 * Tiles between the source and the target slide one pitch toward the source so
 * the dragged tile's landing slot is visually empty.
 */
export function shiftFor(
  index: number,
  from: number,
  to: number,
  pitch: number,
): number {
  if (index === from) return 0;
  if (from < to && index > from && index <= to) return -pitch;
  if (to < from && index >= to && index < from) return pitch;
  return 0;
}
