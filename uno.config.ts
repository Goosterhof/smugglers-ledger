import {
  defineConfig,
  presetAttributify,
  presetIcons,
  presetWind3,
  transformerVariantGroup,
} from "unocss";

// The Smuggler's Ledger — GASLAMP DOUBLE-ENTRY (Design System #00012)
//
// Two grounds, one surface: the shell (title wings, tab spine, margin rail)
// is soot-black under a single lamp wash; the ledger sheet itself is warm
// cream with real ruling. The folio is NOT a second theme — it is a local
// alias remap under `.sl-folio` (see the preflight below), exactly the
// ink-on-paper page-scoping shape. Every contrast pair below is computed in
// #00012, not asserted.
//
// The litmus for every addition: would a gaslamp double-entry ledger do this?
// No pills, no glow, no spinner, no blur, radius 0, ONE shadow.

export default defineConfig({
  presets: [
    presetWind3({ dark: "class" }),
    presetAttributify(),
    presetIcons({
      scale: 1.2,
    }),
  ],
  transformers: [transformerVariantGroup()],
  theme: {
    colors: {
      // Soot ground — the room you stand in
      "sl-soot": "#14110E",
      "sl-cellar": "#1E1A15",
      "sl-iron": "#332C24",
      "sl-chalk": "#D9CFB6",
      "sl-chalk-soft": "#A89C82",
      "sl-lamp": "#D89B32",
      "sl-wax-lit": "#CE6A52",
      // Folio ground — the page you read
      "sl-folio": "#EDE4CE",
      "sl-folio-shade": "#E2D7BC",
      "sl-ink": "#24201A",
      "sl-ink-soft": "rgba(36,32,26,0.68)",
      "sl-rule": "rgba(36,32,26,0.18)",
      "sl-rule-strong": "rgba(36,32,26,0.42)",
      "sl-wax": "#8E2B22",
    },
    fontFamily: {
      folio: "'IM Fell English', 'Iowan Old Style', serif",
      figure: "'IBM Plex Mono', 'Consolas', monospace",
      chrome: "'Epilogue', system-ui, sans-serif",
    },
    boxShadow: {
      // The only shadow in the system: the sheet is a physical object.
      folio: "6px 6px 0 rgba(0, 0, 0, 0.55)",
    },
  },
  shortcuts: {
    // Chrome labels — the room's voice
    "sl-chrome-label":
      "font-chrome text-[11px] leading-[16px] font-700 tracking-[1.5px] uppercase",
    // Column heads and stamp lines — the figures' smallest register
    "sl-column-head":
      "font-figure text-[11px] leading-[26px] font-600 tracking-[1.6px] uppercase",
    // The location column — the loudest thing on the row by treatment
    "sl-entry-where":
      "font-figure text-[13px] leading-[26px] font-500 tracking-[0.6px] uppercase",
    "sl-entry-figure": "font-figure text-[13px] leading-[26px] font-600 tabular-nums",
    "sl-entry-sub": "font-figure text-[11px] leading-[13px] font-400",
    "sl-foot": "font-figure text-[12px] leading-[26px] font-600 uppercase",
    "sl-state-voice": "font-folio italic text-[18px] leading-[26px]",
  },
  preflights: [
    {
      getCSS: () => PREFLIGHT_CSS,
    },
  ],
});

const PREFLIGHT_CSS = `
        :root {
          color-scheme: dark;
          /* -- soot ground -- */
          --sl-soot: #14110E;
          --sl-cellar: #1E1A15;
          --sl-iron: #332C24;
          --sl-chalk: #D9CFB6;
          --sl-chalk-soft: #A89C82;
          --sl-lamp: #D89B32;
          --sl-wax-lit: #CE6A52;
          /* -- folio ground -- */
          --sl-folio: #EDE4CE;
          --sl-folio-shade: #E2D7BC;
          --sl-ink: #24201A;
          --sl-ink-soft: rgba(36,32,26,0.68); /* 0.68, NOT 0.62 — 0.62 measured 4.24:1 and failed */
          --sl-rule: rgba(36,32,26,0.18);
          --sl-rule-strong: rgba(36,32,26,0.42);
          --sl-wax: #8E2B22;
          /* -- rarity ink ramp (six ordinals; stamp glyph first, colour second) -- */
          --sl-tier-0: #3A342A;
          --sl-tier-1: #6B5810;
          --sl-tier-2: #2F5E3A;
          --sl-tier-3: #2A4E78;
          --sl-tier-4: #5A3A78;
          --sl-tier-5: #8A3A12;
          /* -- rhythm -- */
          --sl-pitch: 26px;
          --sl-margin-rule: 56px;
          --sl-gutter: 24px;
          /* -- the one shadow, the one lamp -- */
          --sl-shadow-folio: 6px 6px 0 rgba(0,0,0,0.55);
          --sl-lamp-wash: radial-gradient(120% 80% at 50% -10%, rgba(216,155,50,0.13) 0%, rgba(216,155,50,0.04) 38%, transparent 68%);
          /* -- motion -- */
          --sl-dur-ink: 1500ms;
          --sl-dur-swap: 160ms;
          --sl-dur-nudge: 90ms;
          --sl-ease-settle: cubic-bezier(0.2, 0, 0.1, 1);
          /* -- semantic aliases (soot values; .sl-folio remaps them) -- */
          --sl-surface: var(--sl-soot);
          --sl-text: var(--sl-chalk);
          --sl-text-muted: var(--sl-chalk-soft);
          --sl-line: var(--sl-iron);
          --sl-accent: var(--sl-lamp);
          --sl-input-bg: var(--sl-surface);
          --sl-input-line: var(--sl-iron);
        }
        /* The folio scope: a local alias remap, not a second theme.
           THE TRAP, restated: var()-indirected maps bake at the declaring
           element, so every indirected alias is RE-DECLARED here from the
           folio primitives — never merely inherited (#00012, The Two Grounds). */
        .sl-folio {
          --sl-surface: var(--sl-folio);
          --sl-text: var(--sl-ink);
          --sl-text-muted: var(--sl-ink-soft);
          --sl-line: var(--sl-rule);
          --sl-accent: var(--sl-wax);
          --sl-input-bg: var(--sl-folio);
          --sl-input-line: var(--sl-rule-strong);
        }
        html, body, #app { height: 100%; }
        body {
          margin: 0;
          background: var(--sl-soot);
          color: var(--sl-chalk);
          font-family: 'Epilogue', system-ui, sans-serif;
        }
        ::selection { background: rgba(216, 155, 50, 0.28); }
        ::-webkit-scrollbar { width: 8px; height: 8px; }
        ::-webkit-scrollbar-track { background: transparent; }
        ::-webkit-scrollbar-thumb { background: var(--sl-iron); border-radius: 0; }
        ::-webkit-scrollbar-thumb:hover { background: var(--sl-chalk-soft); }
        /* The Gadget Containment Protocols' reduced-motion floor. Every state
           on this surface is a PLACE (a printed time, a dog-eared tab, a
           completed rule) — the terminal frame is legible with all animation
           deleted. No canvas/RAF surface exists here; the CSS floor covers it
           completely (wireframe #00038). */
        @media (prefers-reduced-motion: reduce) {
          *, *::before, *::after {
            animation-duration: 0.01ms !important;
            animation-iteration-count: 1 !important;
            transition-duration: 0.01ms !important;
            scroll-behavior: auto !important;
          }
        }
      `;
