# Isfahan

A lightweight, 100% offline desktop writing companion that rewards your productivity.

As you type, your pixel-art companion grows happier, their miniature world blossoms with life, and they share pre-programmed words of wisdom from classic books and texts.

## Key Features

- **Privacy by Design**: Uses a global keystroke cadence listener (`rdev`) that **never** inspects or logs which keys are pressed—only that a keystroke occurred.
- **100% Offline & Lightweight**: Built with **Tauri v2 (Rust)**, **Svelte 5**, and **HTML5 `<canvas>`**. Zero network permissions, tiny memory footprint, and <5 MB installer size.
- **Transparent Click-Through Overlay**: Floats unobtrusively in the bottom-right corner of your screen (`alwaysOnTop`, frameless, with toggleable mouse click-through).
- **AI-Assisted Dev Ready**: Structured with hierarchical [`AGENTS.md`](./AGENTS.md) files, `.agents/rules/`, and declarative JSON data files under `static/data/` for effortless iteration.

## Quick Start (macOS / Windows / Linux)

```bash
# 1. Install frontend dependencies with Bun
bun install

# 2. Run web preview (great for rapid UI/Canvas tweaks)
bun run dev

# 3. Run full native desktop overlay (Tauri + Rust)
bun run tauri:dev
```

> **macOS Dev Note**: To allow the global keystroke listener (`rdev`) to detect keystrokes outside the window on macOS, grant your terminal/IDE permission in **System Settings → Privacy & Security → Accessibility**.

## Customizing Word Limits & Quotes

### 1. Changing Daily Word Limits & Quote Frequency

Word and keystroke progression constants live at the top of [`src/lib/stores/companion.svelte.ts`](./src/lib/stores/companion.svelte.ts):

```ts
export const KEYSTROKES_PER_WORD = 5;
export const DAILY_WORD_GOAL = 1000;
export const DAILY_KEYSTROKE_GOAL = DAILY_WORD_GOAL * KEYSTROKES_PER_WORD; // 5,000 keystrokes = 1,000 words = 100%
export const QUOTE_KEYSTROKE_INTERVAL = 150; // +150 keystrokes (~30 words) per quote
```

- **Daily Word Goal (`DAILY_WORD_GOAL`)**: Change `1000` to your desired daily word target (e.g., `500` or `2000`). Reaching `100%` of this goal brings the sanctuary to full bloom for the day. Progress automatically persists in `localStorage` for the current calendar day and resets at midnight.
- **Quote Interval (`QUOTE_KEYSTROKE_INTERVAL`)**: Change `150` (~30 words) to control how many keystrokes trigger the next quote bubble.
- **Level Thresholds ([`static/data/world-stages.json`](./static/data/world-stages.json))**: Each stage defines a `minHappiness` percentage (`0` to `100`) of your `DAILY_WORD_GOAL`:
  - `0%` — *Quiet Earth* (`0 words`)
  - `20%` — *Morning Sprouts* (`200 words` at a 1,000-word goal)
  - `45%` — *Woodland Glade* (`450 words`)
  - `70%` — *Blooming Sanctuary* (`700 words`)
  - `90%` — *Isfahan Oasis* (`900 words`)

### 2. Adding or Editing Quotes

All quotes are stored locally in [`static/data/quotes.json`](./static/data/quotes.json). Add new entries to the JSON array without touching any application code:

```json
{
  "id": "golestan-patience",
  "text": "Have patience. All things are difficult before they become easy.",
  "author": "Saadi Shirazi",
  "source": "Gulistan (1258)",
  "minHappiness": 20
}
```

- `id`: Unique string identifier for the quote.
- `text`: The quote displayed inside the pixel-art speech bubble.
- `author`: Author name.
- `source`: Book or text title.
- `minHappiness`: Minimum daily progress percentage (`0`–`100`) required before this quote enters the random selection pool.

## Project Layout

- [`AGENTS.md`](./AGENTS.md) — AI coding instructions, invariants, and repository map
- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) — Architecture, cross-platform workflow, and asset pipeline
- [`static/data/`](./static/data/) — Edit `quotes.json`, `world-stages.json`, and `sprite-manifest.json`
- [`static/assets/`](./static/assets/README.md) — Drop in custom `.png` sprite sheets & tilesets from itch.io
- [`src/`](./src/) — Svelte 5 frontend, HTML5 `<canvas>` pixel-art renderer, and happiness store
- [`src-tauri/`](./src-tauri/) — Rust backend, privacy-safe keystroke hook, and transparent window controls
