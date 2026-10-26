export interface MagnifyProfile {
  peak: number;
  falloff: number;
  radius: number;
}

const MACOS: MagnifyProfile = { peak: 1.2, falloff: 0.45, radius: 120 };
const VITRO: MagnifyProfile = { peak: 1.6, falloff: 0.45, radius: 120 };

/**
 * Scale for an icon `distance` pixels from the pointer.
 *
 * Falls from `peak` at distance 0 to exactly 1.0 at `radius` and beyond.
 * Negative distances are treated as 0; a non-positive radius returns 1.0.
 *
 * The envelope is a generalized Gaussian: `falloff` is the influence width as
 * a fraction of `radius`, and the quartic shape keeps the influence local so
 * icons beyond an immediate neighbor sit at rest.
 */
export function magnifiedScale(
  distance: number,
  peak: number,
  falloff: number,
  radius: number,
): number {
  if (!(radius > 0)) return 1;
  const d = Math.max(0, distance);
  if (d >= radius) return 1;
  const normalized = d / radius;
  const width = Math.max(falloff, 1e-6);
  const weight = Math.exp(-((normalized / width) ** 4));
  return 1 + (peak - 1) * weight;
}

/** Scale for every icon given the pointer x and the icon centers. */
export function scalesFor(
  centers: number[],
  pointerX: number,
  peak: number,
  falloff: number,
  radius: number,
): number[] {
  return centers.map((center) =>
    magnifiedScale(Math.abs(center - pointerX), peak, falloff, radius),
  );
}

/** Resolve the profile for a settings combination. */
export function profileFor(
  profile: "macos" | "vitro",
  reducedMotion: boolean,
): MagnifyProfile {
  if (reducedMotion) return { peak: 1.0, falloff: 0.45, radius: 120 };
  return profile === "macos" ? MACOS : VITRO;
}
