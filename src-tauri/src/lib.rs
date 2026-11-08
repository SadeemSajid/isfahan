use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition, State, Window};
use tauri_plugin_autostart::MacosLauncher;

struct OverlayState {
    ghost_mode: Arc<AtomicBool>,
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
    state.ghost_mode.store(ignore, Ordering::SeqCst);
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| e.to_string())
}

/// Position the overlay neatly in the bottom-right corner of the primary monitor on startup.
fn position_bottom_right(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen_size = monitor.size();
        let scale = monitor.scale_factor();
        let win_width = (384.0 * scale) as i32;
        let win_height = (288.0 * scale) as i32;
        let margin = (20.0 * scale) as i32;

        let x = (screen_size.width as i32 - win_width - margin).max(0);
        let y = (screen_size.height as i32 - win_height - (margin * 2)).max(0);

        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
}

/// Background watcher that allows the bottom HUD bar to remain clickable even when Ghost mode is on.
fn start_ghost_hud_watcher(window: tauri::WebviewWindow, ghost_mode: Arc<AtomicBool>) {
    thread::spawn(move || {
        let mut currently_ignoring = false;

        loop {
            thread::sleep(Duration::from_millis(60));

            if !ghost_mode.load(Ordering::Relaxed) {
                if currently_ignoring {
                    let _ = window.set_ignore_cursor_events(false);
                    currently_ignoring = false;
                }
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
            let scale = window.scale_factor().unwrap_or(1.0);

            // Bottom HUD bar occupies the bottom 36 logical pixels of the window
            let hud_height_phys = 36.0 * scale;
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
            let _ = ctx.app_handle.emit("keystroke_tick", ());
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

    if let Err(error) = listen(move |event| {
        if let EventType::KeyPress(_) = event.event_type {
            let _ = app_handle.emit("keystroke_tick", ());
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
    let ghost_mode = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .manage(OverlayState {
            ghost_mode: ghost_mode.clone(),
        })
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .invoke_handler(tauri::generate_handler![set_click_through])
        .setup(move |app| {
            if let Some(main_window) = app.get_webview_window("main") {
                position_bottom_right(&main_window);
                start_ghost_hud_watcher(main_window, ghost_mode.clone());
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
