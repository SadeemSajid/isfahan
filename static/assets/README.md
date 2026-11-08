# Pixel Art Assets Directory (`static/assets/`)

Drop your `.png` sprite sheets and tilesets downloaded from **itch.io** (or exported from Aseprite / Tiled) into this directory:

- `character.png` — Companion sprite sheet (default 16×16 frames, configurable in [`../data/sprite-manifest.json`](../data/sprite-manifest.json))
- `tileset.png` — Environment tileset (default 16×16 tiles, configurable in [`../data/sprite-manifest.json`](../data/sprite-manifest.json))

## Automatic Procedural Fallback

Until `character.png` and `tileset.png` are placed in this folder, [`WorldCanvas.svelte`](../../src/lib/components/WorldCanvas.svelte) automatically renders built-in crisp pixel-art tiles, a turquoise fountain, cypress trees, flowers, and an animated companion on the `<canvas>`. Once you drop in `character.png` or `tileset.png`, the renderer automatically detects and draws your sprite sheets!
