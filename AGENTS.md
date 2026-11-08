# Isfahan — AI Agent Instructions & Project Constitution

**Isfahan** is a lightweight, 100% offline desktop writing companion and pixel-art overlay widget built with **Tauri v2 (Rust)**, **Svelte 5 (SvelteKit Static SPA)**, and **HTML5 `<canvas>`**.

As the user types anywhere on their system, the companion's happiness grows, its pixel-art world flourishes with new tiles and props, and it periodically shares pre-programmed words of wisdom from classic books and texts.

---

## 1. Non-Negotiable Security & Privacy Invariants

> **CRITICAL FOR ALL AI AGENTS**: This application registers an OS-level keyboard hook (`rdev::listen`). To protect user privacy and prevent Windows Defender / antivirus false-positives, you **MUST NEVER** violate these rules:

1. **Zero Key Payload Inspection**:
   - In Rust (`src-tauri/src/lib.rs`), **ONLY** match `rdev::EventType::KeyPress(_)` using a wildcard `_`.
   - **NEVER** read, match, format, serialize, or log the key code, scan code, or `event.name`.
   - The event emitted to the frontend (`"keystroke_tick"`) **MUST** carry an empty unit payload `()`.
2. **100% Offline By Design**:
   - **NEVER** add HTTP clients (`reqwest`, `ureq`, `tauri-plugin-http`), WebSockets, telemetry, analytics, or external URL fetches in either Rust or Svelte.
   - All quotes, fonts, sprites, and tilesets must be bundled locally inside `static/`.
3. **Zero Disk I/O Inside Hook Callback**:
   - The `rdev::listen` thread must **never** perform file I/O, database writes, or heavy computation. It must only emit the lightweight `"keystroke_tick"` event.

---

## 2. Tech Stack & Tooling

- **Package Manager**: `bun` (use `bun install`, `bun run dev`, `bun run check`, `bun run tauri:dev`, `bun run tauri:build`)
- **Frontend**: SvelteKit 2 + Svelte 5 Runes (`$state`, `$derived`, `$effect`) + TypeScript (`strict: true`)
- **Rendering**: HTML5 `<canvas>` (`ctx.imageSmoothingEnabled = false` and CSS `image-rendering: pixelated`)
- **Desktop Shell**: Tauri v2 (`src-tauri/`), transparent frameless `alwaysOnTop` overlay window
- **Global Keystroke Listener**: `rdev` crate (cross-platform macOS Quartz & Windows `WH_KEYBOARD_LL`)
- **Autostart on Login**: `tauri-plugin-autostart`
- **CI/CD**: GitHub Actions (`.github/workflows/release.yml`) via `tauri-apps/tauri-action` to build Windows `.msi` / `.exe` and macOS bundles automatically.

---

## 3. Repository Structure (AI-Friendly Modularity)

```text
isfahan/
├── AGENTS.md                        # Root instructions & invariants for AI coding agents
├── .agents/
│   └── rules/
│       ├── privacy-and-security.md  # Enforced security & antivirus-safe rules
│       └── pixel-art-canvas.md      # Crisp pixel-art rendering & asset rules
├── docs/
│   └── ARCHITECTURE.md              # Deep dive into overlay, macOS permissions, & Windows builds
├── static/
│   ├── assets/                      # Drop itch.io PNG sprite sheets & tilesets here
│   │   └── README.md                # Asset drop-in specifications
│   └── data/                        # Declarative JSON data (edit without touching code)
│       ├── quotes.json              # Words of wisdom, book sources, & happiness thresholds
│       ├── world-stages.json        # Happiness stages & world object unlocks
│       └── sprite-manifest.json     # Frame sizes, animations, & tile coordinates for PNGs
├── src/
│   ├── AGENTS.md                    # Frontend-specific Svelte 5 + Canvas rules
│   ├── app.css                      # Transparent window styling & pixelated rendering rules
│   ├── app.html                     # Root HTML template
│   ├── lib/
│   │   ├── components/
│   │   │   ├── WorldCanvas.svelte   # HTML5 Canvas tile + character renderer
│   │   │   ├── SpeechBubble.svelte  # Pixel-art quote bubble
│   │   │   └── HudControls.svelte   # Unobtrusive hover controls (click-through, drag, autostart)
│   │   ├── stores/
│   │   │   └── companion.svelte.ts  # Reactive happiness engine, decay loop, & quote selector
│   │   └── types/
│   │       └── world.ts             # TypeScript interfaces for JSON schemas & world state
│   └── routes/
│       ├── +layout.ts               # SSR disabled (`export const ssr = false`)
│       └── +page.svelte             # Main overlay composition & click-through management
└── src-tauri/
    ├── AGENTS.md                    # Rust/Tauri-specific rules
    ├── Cargo.toml                   # Rust dependencies (tauri, rdev, tauri-plugin-autostart)
    ├── tauri.conf.json              # Transparent, undecorated, always-on-top overlay config
    ├── capabilities/
    │   └── default.json             # Minimal offline-only Tauri v2 permissions
    └── src/
        ├── lib.rs                   # Privacy-safe rdev listener & window commands
        └── main.rs                  # Binary entrypoint
```

---

## 4. Development & Verification Commands

- **Install dependencies**: `bun install`
- **Typecheck Svelte / TS**: `bun run check`
- **Check Rust compilation**: `cargo check --manifest-path src-tauri/Cargo.toml`
- **Run web-only preview (with simulated keystrokes)**: `bun run dev`
- **Run desktop overlay (Tauri dev)**: `bun run tauri:dev`
- **Build production installer**: `bun run tauri:build`
