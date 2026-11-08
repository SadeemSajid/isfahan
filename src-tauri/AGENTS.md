# Rust & Tauri Backend Rules (`src-tauri/`)

When modifying files inside `src-tauri/`:

1. **Keyboard Hook Privacy (`src/lib.rs`)**:
   - On **macOS** (`#[cfg(target_os = "macos")]`), use the direct `CGEventTapCreate` listener (`macos_listener`) with `event_mask = 1 << K_CG_EVENT_KEY_DOWN`. Do **not** use `rdev` on macOS because `rdev` calls `TSMGetInputSourceProperty` on a background thread (which crashes with `_dispatch_assert_queue_fail` on macOS) and hooks all mouse events.
   - On **Windows/Linux** (`#[cfg(not(target_os = "macos"))]`), use `rdev::listen` inside a dedicated `std::thread::spawn` background thread and match **only** `EventType::KeyPress(_)`.
   - Emit `"keystroke_tick"` with `()` payload via `app_handle.emit("keystroke_tick", ())`.
   - Never inspect keycodes/characters, write to disk, allocate large buffers, or block inside the hook callback.

2. **Transparent Click-Through Overlay**:
   - Keep `"transparent": true`, `"decorations": false`, and `"alwaysOnTop": true` in `tauri.conf.json`.
   - Expose the `set_click_through(window: tauri::Window, ignore: bool)` command using `window.set_ignore_cursor_events(ignore)` so the frontend can pass mouse clicks through transparent canvas regions.

3. **Zero Network Footprint**:
   - Do not add network-related crates (`reqwest`, `hyper`, `tokio-tungstenite`, `tauri-plugin-http`).
   - Keep `capabilities/default.json` restricted to local window management, events, and `autostart`.

4. **Cross-Platform Parity (macOS Dev -> Windows Target)**:
   - Avoid platform-only APIs unless gated with `#[cfg(target_os = "windows")]` and paired with a macOS equivalent so `cargo check` and `bun run tauri:dev` work seamlessly on macOS.
