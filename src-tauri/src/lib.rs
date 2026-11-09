use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, PhysicalPosition, State, Window};
use tauri_plugin_autostart::MacosLauncher;

/// Minimum interval (ms) between emitted `keystroke_tick` IPC events to prevent
/// key-repeat storms from flooding WebView IPC while preserving full typing responsiveness.
const MIN_TICK_INTERVAL_MS: u64 = 40;

#[inline]
fn should_emit_tick(last_tick_ms: &AtomicU64) -> bool {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let prev = last_tick_ms.load(Ordering::Relaxed);
    if now_ms.saturating_sub(prev) < MIN_TICK_INTERVAL_MS {
        return false;
    }
    last_tick_ms.store(now_ms, Ordering::Relaxed);
    true
}

struct GhostSignal {
    enabled: Mutex<bool>,
    cvar: Condvar,
}

struct OverlayState {
    ghost: Arc<GhostSignal>,
}

/// Toggle Ghost (click-through) mode on the transparent overlay window.
/// When Ghost mode is active, clicks pass through the world and sky,
/// but hovering the bottom HUD strip automatically re-enables cursor events
/// so the user can always click the Ghost button again to turn it off.
#[tauri::command]
fn set_click_through(
    window: Window,
    state: State<'_, OverlayState>,
    ignore: bool,
) -> Result<(), String> {
    if let Ok(mut enabled) = state.ghost.enabled.lock() {
        *enabled = ignore;
        state.ghost.cvar.notify_all();
    }
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| e.to_string())
}

/// Position the overlay neatly in the bottom-right corner of the primary monitor on startup,
/// with enough bottom clearance to stay above the macOS Dock and Windows Taskbar.
fn position_bottom_right(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen_size = monitor.size();
        let scale = monitor.scale_factor();
        let win_width = (384.0 * scale) as i32;
        let win_height = (288.0 * scale) as i32;
        let right_margin = (24.0 * scale) as i32;
        let bottom_margin = (96.0 * scale) as i32;

        let x = (screen_size.width as i32 - win_width - right_margin).max(0);
        let y = (screen_size.height as i32 - win_height - bottom_margin).max(0);

        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
}

/// Background watcher that allows the bottom HUD bar to remain clickable when Ghost mode is on.
/// Sleeps on a Condvar with 0% CPU usage whenever Ghost mode is disabled.
fn start_ghost_hud_watcher(window: tauri::WebviewWindow, ghost: Arc<GhostSignal>) {
    thread::spawn(move || {
        let mut currently_ignoring = false;
        let scale = window.scale_factor().unwrap_or(1.0);
        let hud_height_phys = 36.0 * scale;

        loop {
            // Park thread with zero CPU wakeups while Ghost mode is off
            {
                let mut enabled = ghost.enabled.lock().unwrap();
                while !*enabled {
                    if currently_ignoring {
                        let _ = window.set_ignore_cursor_events(false);
                        currently_ignoring = false;
                    }
                    enabled = ghost.cvar.wait(enabled).unwrap();
                }
            }

            thread::sleep(Duration::from_millis(100));

            let is_enabled = ghost.enabled.lock().map(|g| *g).unwrap_or(false);
            if !is_enabled {
                continue;
            }

            let Ok(cursor) = window.cursor_position() else {
                continue;
            };
            let Ok(win_pos) = window.outer_position() else {
                continue;
            };
            let Ok(win_size) = window.outer_size() else {
                continue;
            };

            let win_x = win_pos.x as f64;
            let win_y = win_pos.y as f64;
            let win_w = win_size.width as f64;
            let win_h = win_size.height as f64;

            let in_hud_strip = cursor.x >= win_x
                && cursor.x <= win_x + win_w
                && cursor.y >= (win_y + win_h - hud_height_phys)
                && cursor.y <= win_y + win_h;

            let should_ignore = !in_hud_strip;
            if should_ignore != currently_ignoring {
                let _ = window.set_ignore_cursor_events(should_ignore);
                currently_ignoring = should_ignore;
            }
        }
    });
}

/// macOS-native privacy-safe keystroke listener using CoreGraphics `CGEventTapCreate`.
#[cfg(target_os = "macos")]
mod macos_listener {
    use std::ffi::c_void;
    use std::ptr;
    use std::sync::atomic::AtomicU64;
    use tauri::{AppHandle, Emitter};

    type CFMachPortRef = *mut c_void;
    type CFAllocatorRef = *const c_void;
    type CFRunLoopSourceRef = *mut c_void;
    type CFRunLoopRef = *mut c_void;
    type CFStringRef = *const c_void;
    type CGEventTapProxy = *mut c_void;
    type CGEventRef = *mut c_void;
    type CGEventMask = u64;

    const K_CG_HID_EVENT_TAP: u32 = 0;
    const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
    const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;
    const K_CG_EVENT_KEY_DOWN: u32 = 10;
    const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;

    struct TapContext {
        app_handle: AppHandle,
        tap: CFMachPortRef,
        last_tick_ms: AtomicU64,
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFRunLoopCommonModes: CFStringRef;

        fn CGEventTapCreate(
            tap: u32,
            place: u32,
            options: u32,
            events_of_interest: CGEventMask,
            callback: unsafe extern "C" fn(
                proxy: CGEventTapProxy,
                event_type: u32,
                event: CGEventRef,
                user_info: *mut c_void,
            ) -> CGEventRef,
            user_info: *mut c_void,
        ) -> CFMachPortRef;

        fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

        fn CFMachPortCreateRunLoopSource(
            allocator: CFAllocatorRef,
            port: CFMachPortRef,
            order: isize,
        ) -> CFRunLoopSourceRef;

        fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
        fn CFRunLoopRun();
    }

    unsafe extern "C" fn event_tap_callback(
        _proxy: CGEventTapProxy,
        event_type: u32,
        event: CGEventRef,
        user_info: *mut c_void,
    ) -> CGEventRef {
        if user_info.is_null() {
            return event;
        }
        let ctx = &*(user_info as *const TapContext);

        if event_type == K_CG_EVENT_KEY_DOWN {
            // Privacy by design: we never read the keycode or character from `event`.
            if super::should_emit_tick(&ctx.last_tick_ms) {
                let _ = ctx.app_handle.emit("keystroke_tick", ());
            }
        } else if event_type == K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT && !ctx.tap.is_null() {
            CGEventTapEnable(ctx.tap, true);
        }

        event
    }

    pub fn start(app_handle: AppHandle) {
        unsafe {
            let ctx_ptr = Box::into_raw(Box::new(TapContext {
                app_handle,
                tap: ptr::null_mut(),
                last_tick_ms: AtomicU64::new(0),
            }));

            let event_mask: CGEventMask = 1 << K_CG_EVENT_KEY_DOWN;
            let tap = CGEventTapCreate(
                K_CG_HID_EVENT_TAP,
                K_CG_HEAD_INSERT_EVENT_TAP,
                K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
                event_mask,
                event_tap_callback,
                ctx_ptr as *mut c_void,
            );

            if tap.is_null() {
                eprintln!(
                    "Isfahan: Could not create macOS CGEventTap. Grant Accessibility permission in System Settings -> Privacy & Security -> Accessibility."
                );
                let _ = Box::from_raw(ctx_ptr);
                return;
            }

            (*ctx_ptr).tap = tap;

            let loop_source = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
            if loop_source.is_null() {
                let _ = Box::from_raw(ctx_ptr);
                return;
            }

            let current_loop = CFRunLoopGetCurrent();
            CFRunLoopAddSource(current_loop, loop_source, kCFRunLoopCommonModes);
            CGEventTapEnable(tap, true);
            CFRunLoopRun();
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn start_keystroke_listener(app_handle: AppHandle) {
    use rdev::{listen, EventType};
    use tauri::Emitter;

    let last_tick_ms = AtomicU64::new(0);
    if let Err(error) = listen(move |event| {
        if let EventType::KeyPress(_) = event.event_type {
            if should_emit_tick(&last_tick_ms) {
                let _ = app_handle.emit("keystroke_tick", ());
            }
        }
    }) {
        eprintln!("Global keyboard listener error: {:?}", error);
    }
}

#[cfg(target_os = "macos")]
fn start_keystroke_listener(app_handle: AppHandle) {
    macos_listener::start(app_handle);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ghost = Arc::new(GhostSignal {
        enabled: Mutex::new(false),
        cvar: Condvar::new(),
    });

    tauri::Builder::default()
        .manage(OverlayState {
            ghost: ghost.clone(),
        })
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .invoke_handler(tauri::generate_handler![set_click_through])
        .setup(move |app| {
            if let Some(main_window) = app.get_webview_window("main") {
                position_bottom_right(&main_window);
                start_ghost_hud_watcher(main_window, ghost.clone());
            }

            let app_handle = app.handle().clone();

            // Spawn privacy-safe global keystroke cadence listener
            thread::spawn(move || {
                start_keystroke_listener(app_handle);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
