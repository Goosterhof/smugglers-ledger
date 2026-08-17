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
      // IRON & ICHOR (#00013, hybrid — investor-ruled 2026-08-16) — one cold
      // near-black ground, light ash text EVERYWHERE (no polarity flip), the
      // game's loot colours as the rarity words, one blood-red accent. These
      // MUST track the :root custom properties below verbatim; UnoCSS resolves
      // bg-sl-*/text-sl-* through this map, inline var(--sl-*) reads :root.
      // Every ratio is computed (WCAG 2.x); do not "tidy" a value without
      // re-deriving it. Cold near-black grounds:
      "sl-soot": "#05070A",
      "sl-cellar": "#080B10",
      "sl-iron": "#1E232B",
      "sl-folio": "#0A0D12",
      "sl-folio-shade": "#10141B",
      // Cooled bone-grey ash (light on dark); ink == chalk, polarity is uniform:
      "sl-chalk": "#BCC1C2",
      "sl-chalk-soft": "#7E8790",
      "sl-ink": "#BCC1C2",
      "sl-ink-soft": "#949A9C",
      "sl-rule": "rgba(188,193,194,0.13)",
      "sl-rule-strong": "rgba(188,193,194,0.28)",
      // The one accent (ichor — rule/edge/glyph only), the ember secondary,
      // the correction reds:
      "sl-lamp": "#D95C4C",
      // v0.7.0 THE CLOTTED CHROME — ichor's two new bodies. The wet ink the
      // room's controls are written in, and the dried ground the chosen one
      // sits on. Both measured; see #00013 §5. Mirror in :root below.
      "sl-ichor-ink": "#E6614F",
      "sl-clot": "#5A1815",
      "sl-clot-deep": "#230A0B",
      "sl-ember": "#C98F3E",
      "sl-wax": "#E0705E",
      "sl-wax-lit": "#EC8574",
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
    // THE CHIT — the v0.7.0 button language. A control at rest is written in
    // wet ichor; the one you have taken inverts to ash on dried blood behind
    // the system's own 2px lamp inset rule. Commands wear the taken dress at
    // rest, because a command is a choice already made.
    "sl-chit": "text-sl-ichor-ink hover:bg-sl-clot-deep hover:text-sl-chalk",
    "sl-chit-taken":
      "bg-sl-clot text-sl-chalk shadow-[inset_2px_0_0_0_var(--sl-lamp)]",
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
          /* IRON & ICHOR (#00013, hybrid — investor-ruled 2026-08-16): one cold
             near-black ground, light ash text EVERYWHERE (no polarity flip),
             the game's own loot colours as the rarity WORDS, one blood-red
             ichor accent (rule/edge/glyph only). The dark ground is FORCED, not
             stylistic — rarity=game-colour makes Rare green, so a green ground
             would fight the green word; red is the only accent touching none of
             the six loot hues, and a dark ground is what makes those hues
             legible as text. Every value carries a computed WCAG ratio (2.x);
             do not "tidy" one without re-deriving it. THIS map mirrors
             theme.colors above verbatim. Cold near-black grounds: */
          --sl-soot: #05070A;          /* the room / app background */
          --sl-cellar: #080B10;        /* rails, tab spine, wings, panels */
          --sl-iron: #1E232B;          /* borders on the room + the plate's lit top/left catch-light */
          --sl-folio: #0A0D12;         /* THE SHEET / plate — one value-step up from the room */
          --sl-folio-shade: #10141B;   /* the counting band + column-head field */
          /* Cooled bone-grey ash — light on dark, uniform polarity (ink == chalk): */
          --sl-chalk: #BCC1C2;         /* primary text on the room — 11.09:1 soot */
          --sl-chalk-soft: #7E8790;    /* muted labels on the room — 5.53:1 soot */
          --sl-ink: #BCC1C2;           /* primary text on the sheet — 10.70:1 folio (== chalk, the flip is gone) */
          --sl-ink-soft: #949A9C;      /* secondary on the sheet — 6.82:1 folio (SOLID ash, NOT an alpha) */
          --sl-rule: rgba(188,193,194,0.13);       /* hairline ruling — light-alpha so it reads on dark */
          --sl-rule-strong: rgba(188,193,194,0.28); /* separators, sheet edge, margin rule */
          /* The one accent, the ember secondary, the correction reds: */
          --sl-lamp: #D95C4C;          /* ichor — the rule/edge/focus colour (v0.7.0: see THE ICHOR LAW in #00013 — room ink + chosen ground now live in the two tokens below) */
          /* v0.7.0 THE CLOTTED CHROME — ichor's two new bodies (mirrors
             theme.colors above verbatim). Wet: the ink every resting control
             in the ROOM is written in. Dried: the ground the chosen control
             and every command sit on. NO loot hue may ever land on --sl-clot
             (Epic blue measures 4.04 there); --sl-wax may not either (4.22). */
          --sl-ichor-ink: #E6614F;   /* control ink — 5.94 soot / 5.81 cellar / 5.73 folio / 5.44 band */
          --sl-clot: #5A1815;        /* the taken chit's dried-blood ground — chalk on it 7.32 */
          --sl-clot-deep: #230A0B;   /* the hover wash — chalk on it 10.31, ichor-ink 5.53 */
          --sl-ember: #C98F3E;         /* warm brass — stat magnitudes, footed totals, the sort caret — 6.93:1 folio */
          --sl-wax: #E0705E;           /* the correction red as small text (struck word, Strike/Lift verbs) — 6.16:1 folio */
          --sl-wax-lit: #EC8574;       /* the Refused/flagged alarm on the room — 7.85:1 soot */
          /* Rarity words — the game's loot ladder, now TEXT on the sheet (ratio on --sl-folio): */
          --sl-loot-0: #E9E6DE;        /* Common — 15.60:1 */
          --sl-loot-1: #E3C53C;        /* Magical — 11.41:1 */
          --sl-loot-2: #54B65C;        /* Rare — 7.62:1 */
          --sl-loot-3: #5E8DE0;        /* Epic — 5.90:1 (tightest; the dark ground lifts it above the light-page 5.00) */
          --sl-loot-4: #B07EE6;        /* Legendary — 6.46:1 (the one lift, from game #A468E0) */
          --sl-loot-5: #D6923A;        /* Quest — 7.44:1 */
          /* -- rhythm -- */
          --sl-pitch: 26px;
          --sl-margin-rule: 56px;
          --sl-gutter: 24px;
          /* -- the one shadow, the light source -- */
          --sl-shadow-folio: 6px 6px 0 rgba(0,0,0,0.70);
          --sl-lamp-wash:
            radial-gradient(120% 80% at 42% -10%, rgba(217,92,76,0.07) 0%, rgba(217,92,76,0.025) 40%, transparent 70%),
            radial-gradient(130% 100% at 50% 38%, transparent 34%, rgba(0,0,0,0.62) 100%);
          /* -- motion -- */
          --sl-dur-ink: 1500ms;
          --sl-dur-swap: 160ms;
          --sl-dur-nudge: 90ms;
          --sl-ease-settle: cubic-bezier(0.2, 0, 0.1, 1);
          /* -- semantic aliases — one dark ground, no polarity flip, so these
                are globally correct and the .sl-folio remap is RETIRED
                (Iron & Ichor dissolves #00012's var()-indirection trap by
                construction: there is no second ground to re-declare for) -- */
          --sl-surface: var(--sl-soot);
          --sl-text: var(--sl-chalk);
          --sl-text-muted: var(--sl-chalk-soft);
          --sl-line: var(--sl-iron);
          --sl-accent: var(--sl-lamp);
          --sl-input-bg: var(--sl-surface);
          --sl-input-line: var(--sl-iron);
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
