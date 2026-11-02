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

export type GlyphName =
  | "folder"
  | "gear"
  | "grid"
  | "chart"
  | "sliders"
  | "tray"
  | "unknown";

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
            <circle {...STROKE} cx="12" cy="12" r="4.6" />
            <circle {...STROKE} cx="12" cy="12" r="1.9" />
            <path
              {...STROKE}
              d="M12 2.8v2.4M12 18.8v2.4M2.8 12h2.4M18.8 12h2.4M5.5 5.5l1.7 1.7M16.8 16.8l1.7 1.7M5.5 18.5l1.7-1.7M16.8 7.2l1.7-1.7"
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
            <path {...STROKE} d="M4 19.2h16" />
            <path {...STROKE} d="M7.2 19.2v-5.4M11.6 19.2V8.2M16 19.2v-7.6" />
          </>
        )}
        {name === "sliders" && (
          <>
            <path {...STROKE} d="M4 8.4h8.4M17.2 8.4H20" />
            <circle {...STROKE} cx="14.6" cy="8.4" r="2.2" />
            <path {...STROKE} d="M4 15.6h2.8M11.6 15.6H20" />
            <circle {...STROKE} cx="9.4" cy="15.6" r="2.2" />
          </>
        )}
        {/* From design-art/art/dock-glyphs.svg#g-tray: a tray with a drop arrow. */}
        {name === "tray" && (
          <>
            <path
              {...STROKE}
              d="M4 12.8v4.2A2.4 2.4 0 0 0 6.4 19.4h11.2a2.4 2.4 0 0 0 2.4-2.4v-4.2"
            />
            <path {...STROKE} d="M3.2 12.8h4.9l1.3 2.2h5.2l1.3-2.2h4.9" />
            <path {...STROKE} d="M12 4.4v6.2M12 10.6 9.6 8.2M12 10.6l2.4-2.4" />
          </>
        )}
        {name === "unknown" && (
          <rect {...STROKE} x="5" y="5" width="14" height="14" rx="3" />
        )}
      </svg>
    </span>
  );
}
