# Frontend Svelte 5 & HTML5 Canvas Rules (`src/`)

When modifying files inside `src/`:

1. **Svelte 5 Runes**:
   - Use Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`) instead of legacy Svelte 4 `writable` stores or `export let`.
   - Keep SSR disabled in `src/routes/+layout.ts` (`export const ssr = false; export const prerender = true;`).

2. **Crisp Pixel Art on `<canvas>`**:
   - Every `<canvas>` element must have CSS `image-rendering: pixelated;` and `ctx.imageSmoothingEnabled = false;` set before drawing.
   - Use integer coordinates (`Math.floor(x)`, `Math.floor(y)`) when calling `ctx.drawImage` or `ctx.fillRect` to prevent sub-pixel blurring.
   - Render at native pixel-art resolution (e.g., `180x140` logical pixels) and scale up cleanly by an integer factor (`2x` -> `360x280`) via CSS or canvas scaling.

3. **Data-Driven Design**:
   - Quotes live in `static/data/quotes.json`.
   - World progression stages live in `static/data/world-stages.json`.
   - Sprite sheet & tileset mappings live in `static/data/sprite-manifest.json`.
   - Always validate changes against the TypeScript interfaces in `src/lib/types/world.ts`.

4. **Browser Fallback for Rapid AI / Web Testing**:
   - Always guard Tauri API calls (`@tauri-apps/api/*`) with a runtime check (`'__TAURI_INTERNALS__' in window`).
   - When running in a standard browser (`bun run dev`), fall back to a `window.addEventListener('keydown', () => registerKeystrokeTick())` listener (ignoring the `KeyboardEvent` key details) so UI and canvas changes can be tested both in Tauri and in any browser.
