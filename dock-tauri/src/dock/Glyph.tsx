/**
 * Inline SVG glyphs for system tiles and control buttons.
 *
 * These are deliberately vector rather than extracted bitmaps: Explorer and
 * Settings are shell targets with no executable path, so the icon extractor
 * cannot produce an image for them, and a letter placeholder reads as broken.
 */

const STROKE = {
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.7,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
};

export type GlyphName = "folder" | "gear" | "grid" | "chart" | "sliders" | "unknown";

export function glyphForSystem(name: string): GlyphName {
  const lower = name.toLowerCase();
  if (lower.includes("explorer") || lower.includes("file")) return "folder";
  if (lower.includes("setting")) return "gear";
  return "unknown";
}

export function Glyph({ name }: { name: GlyphName }) {
  return (
    <span className="dock-glyph" aria-hidden="true">
      <svg viewBox="0 0 24 24" width="100%" height="100%">
        {name === "folder" && (
          <path
            {...STROKE}
            d="M3 7.5A2.5 2.5 0 0 1 5.5 5h3.2c.5 0 1 .2 1.4.6l1.1 1.1c.3.3.8.5 1.2.5h5.9A2.5 2.5 0 0 1 21 9.7v7.8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5z"
          />
        )}
        {name === "gear" && (
          <>
            <circle {...STROKE} cx="12" cy="12" r="3.2" />
            <path
              {...STROKE}
              d="M12 3.5v2.1M12 18.4v2.1M4.9 7.9l1.8 1M17.3 15.1l1.8 1M4.9 16.1l1.8-1M17.3 8.9l1.8-1"
            />
          </>
        )}
        {name === "grid" && (
          <>
            <rect {...STROKE} x="4" y="4" width="6.5" height="6.5" rx="1.6" />
            <rect {...STROKE} x="13.5" y="4" width="6.5" height="6.5" rx="1.6" />
            <rect {...STROKE} x="4" y="13.5" width="6.5" height="6.5" rx="1.6" />
            <rect {...STROKE} x="13.5" y="13.5" width="6.5" height="6.5" rx="1.6" />
          </>
        )}
        {name === "chart" && (
          <>
            <path {...STROKE} d="M4 19h16" />
            <path {...STROKE} d="M7 19v-5M11.5 19V8M16 19v-8" />
          </>
        )}
        {name === "sliders" && (
          <>
            <path {...STROKE} d="M4 8h9M17 8h3M4 16h3M11 16h9" />
            <circle {...STROKE} cx="15" cy="8" r="1.8" />
            <circle {...STROKE} cx="9" cy="16" r="1.8" />
          </>
        )}
        {name === "unknown" && (
          <rect {...STROKE} x="5" y="5" width="14" height="14" rx="3" />
        )}
      </svg>
    </span>
  );
}
