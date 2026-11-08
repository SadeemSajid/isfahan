# Rule: Pixel-Art & Overlay Rendering Conventions

## Visual & Performance Standards
- **Integer Scaling Only**: Base logical resolution is `180x140` (11x8 tile grid of `16x16` tiles, or `2x` integer scale to `360x280` window).
- **No Anti-Aliasing**: Always enforce `ctx.imageSmoothingEnabled = false` on `<canvas>` 2D contexts and `image-rendering: pixelated` in CSS.
- **Low CPU/GPU Footprint**: Since Isfahan runs continuously in the background while the user works, keep the `<canvas>` animation loop throttled (e.g., 12–24 FPS for pixel-art sprite frames, or redraw on state/frame tick) rather than spinning busy calculations at 120Hz.
- **Graceful Asset Fallback**: In `WorldCanvas.svelte`, if an itch.io sprite sheet PNG is not yet present in `static/assets/`, render crisp procedural pixel-art sprites and tiles so the app always looks complete out of the box.
