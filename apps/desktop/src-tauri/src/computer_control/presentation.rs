//! Optional native UI. Visibility and hotkeys never own execution authority.
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct Presentation {
    pub edge: AtomicBool,
    pub shortcut: AtomicBool,
    pub shortcut_unavailable: AtomicBool,
}

pub fn update_edge(app: &AppHandle, activity: bool) {
    let enabled = app.state::<Presentation>().edge.load(Ordering::Acquire);
    let monitors = app.available_monitors().unwrap_or_default();
    for (index, monitor) in monitors.iter().enumerate().take(16) {
        let label = format!("computer-activity-{index}");
        let existing = app.get_webview_window(&label);
        if !enabled || !activity {
            if let Some(window) = existing {
                let _ = window.hide();
            }
            continue;
        }
        let window = existing.or_else(|| {
            tauri::WebviewWindowBuilder::new(
                app,
                &label,
                tauri::WebviewUrl::App("computer-activity.html".into()),
            )
            .title("WebCodex Computer activity")
            .decorations(false)
            .resizable(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .focused(false)
            .focusable(false)
            .content_protected(true)
            .visible(false)
            .inner_size(
                f64::from(monitor.size().width) / monitor.scale_factor(),
                30.0,
            )
            .build()
            .ok()
        });
        if let Some(window) = window {
            if window.set_ignore_cursor_events(true).is_err() {
                let _ = window.hide();
                continue;
            }
            let _ = window.set_position(*monitor.position());
            let _ = window.set_size(tauri::LogicalSize::new(
                f64::from(monitor.size().width) / monitor.scale_factor(),
                30.0,
            ));
            let _ = window.show();
        }
    }
    // Hide orphaned overlays after a display is removed.
    for (label, window) in app.webview_windows() {
        if label
            .strip_prefix("computer-activity-")
            .and_then(|suffix| suffix.parse::<usize>().ok())
            .is_some_and(|index| index >= monitors.len())
        {
            let _ = window.hide();
        }
    }
}

#[cfg(windows)]
pub fn start_shortcut_listener(app: &AppHandle) {
    let listener_app = app.clone();
    let result = std::thread::Builder::new()
        .name("computer-pause-shortcut".into())
        .spawn(move || {
            let app = listener_app;
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                RegisterHotKey, UnregisterHotKey,
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                PeekMessageW, MSG, PM_REMOVE, WM_HOTKEY,
            };
            const ID: i32 = 1339;
            let mut registered = false;
            loop {
                let desired = app.state::<Presentation>().shortcut.load(Ordering::Acquire);
                if desired != registered {
                    if desired {
                        // Ctrl+Alt+Shift+F10, no autorepeat. Escape remains available
                        // to the foreground application and Computer key vocabulary.
                        registered =
                            unsafe { RegisterHotKey(std::ptr::null_mut(), ID, 0x4007, 0x79) } != 0;
                        app.state::<Presentation>()
                            .shortcut_unavailable
                            .store(!registered, Ordering::Release);
                        if !registered {
                            app.state::<Presentation>()
                                .shortcut
                                .store(false, Ordering::Release);
                        }
                    } else {
                        unsafe { UnregisterHotKey(std::ptr::null_mut(), ID) };
                        registered = false;
                    }
                }
                let mut message: MSG = unsafe { std::mem::zeroed() };
                while unsafe { PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) }
                    != 0
                {
                    if registered && message.message == WM_HOTKEY && message.wParam == ID as usize {
                        crate::tray::pause_computer(&app);
                    }
                }
                if app
                    .state::<crate::desktop_shell::DesktopShellState>()
                    .close_disposition()
                    == crate::desktop_shell::CloseDisposition::AllowExit
                {
                    if registered {
                        unsafe { UnregisterHotKey(std::ptr::null_mut(), ID) };
                    }
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(30));
            }
        });
    if result.is_err() {
        app.state::<Presentation>()
            .shortcut_unavailable
            .store(true, Ordering::Release);
    }
}
