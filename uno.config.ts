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
      // BLOOD & PARCHMENT (Lot C) — these MUST track the :root custom
      // properties below; UnoCSS resolves bg-sl-*/text-sl-* utilities through
      // this map, while inline var(--sl-*) styles read :root. Two surfaces,
      // one palette — edit both or the page ships half-brown.
      "sl-soot": "#171210",
      "sl-cellar": "#201815",
      "sl-iron": "#3D2E26",
      "sl-chalk": "#CBB89A",
      "sl-chalk-soft": "#A0917A",
      "sl-lamp": "#E0705E",
      "sl-oxblood": "#A32E24",
      "sl-wax-lit": "#EC8574",
      // Folio ground — the page you read
      "sl-folio": "#E5D9BC",
      "sl-folio-shade": "#D9CCAA",
      "sl-ink": "#241D16",
      "sl-ink-soft": "rgba(36,29,22,0.68)",
      "sl-rule": "rgba(36,29,22,0.18)",
      "sl-rule-strong": "rgba(36,29,22,0.42)",
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
          /* BLOOD & PARCHMENT (Lot C, investor-ruled 2026-08-16): Grim Dawn's
             own chrome — the blood-red of the logo and health globe over
             darkened leather, page as true aged parchment. Every value below
             carries a measured WCAG ratio; the lamp SPLIT is load-bearing —
             deep oxblood (#A32E24) is legible-irrelevant (it only tints the
             non-text wash), while the interactive lamp is a LIT blood-red so
             chrome labels clear 4.5:1 on the dark room (oxblood-as-text was
             2.6:1 and failed). */
          --sl-soot: #171210;          /* darkened leather — the room */
          --sl-cellar: #201815;        /* strip / panel ground */
          --sl-iron: #3D2E26;          /* dark borders */
          --sl-chalk: #CBB89A;         /* dark-room text — 9.6:1 on soot */
          --sl-chalk-soft: #A0917A;    /* dimmed secondary — 6.0:1 on soot */
          --sl-lamp: #E0705E;          /* LIT blood-red — interactive accent, 5.9:1 on soot / 5.5:1 on strip */
          --sl-oxblood: #A32E24;       /* deep blood — wash + non-text borders ONLY, never a text colour */
          --sl-wax-lit: #EC8574;       /* the Refused alarm red — 7.2:1 on soot */
          /* -- folio ground (aged parchment) -- */
          --sl-folio: #E5D9BC;
          --sl-folio-shade: #D9CCAA;
          --sl-ink: #241D16;           /* 11.9:1 on parchment */
          --sl-ink-soft: rgba(36,29,22,0.68); /* 4.9:1 on folio / 4.6:1 on shade — verified, do not lower the alpha */
          --sl-rule: rgba(36,29,22,0.18);
          --sl-rule-strong: rgba(36,29,22,0.42);
          --sl-wax: #8E2B22;           /* the strike red on parchment — 6.0:1, a bookkeeper's correction */
          /* -- rarity ink ramp (six ordinals; stamp glyph first, colour second) -- */
          --sl-tier-0: #3A342A;
          --sl-tier-1: #6B5810;
          --sl-tier-2: #2F5E3A;
          --sl-tier-3: #2A4E78;
          --sl-tier-4: #5A3A78;
          --sl-tier-5: #8A3A12;
          /* -- loot chips: the game's own rarity hues, vivid, carried by a
                stamped swatch (never by text — the tier inks above keep the
                words WCAG-legible on cream). White/Yellow/Green/Blue/Purple
                per the game's loot labels; Quest keeps the amber family. -- */
          --sl-loot-0: #E9E6DE;
          --sl-loot-1: #E3C53C;
          --sl-loot-2: #54B65C;
          --sl-loot-3: #5E8DE0;
          --sl-loot-4: #A468E0;
          --sl-loot-5: #D6923A;
          /* -- rhythm -- */
          --sl-pitch: 26px;
          --sl-margin-rule: 56px;
          --sl-gutter: 24px;
          /* -- the one shadow, the one lamp -- */
          --sl-shadow-folio: 6px 6px 0 rgba(0,0,0,0.55);
          --sl-lamp-wash: radial-gradient(120% 80% at 50% -10%, rgba(163,46,36,0.15) 0%, rgba(163,46,36,0.05) 38%, transparent 68%);
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
