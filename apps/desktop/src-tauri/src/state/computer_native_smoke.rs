//! Runs only in the explicitly built native smoke host, with isolated AppData.
use super::*;
use tauri::{AppHandle, Manager};
use webcodex_computer_control::{self as control, LocalControl};

fn wait_until(label: &str, mut check: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while !check() {
        assert!(
            std::time::Instant::now() < deadline,
            "Computer native smoke: {label}"
        );
        std::thread::sleep(Duration::from_millis(40));
    }
}

fn menu(app: &AppHandle, id: impl Into<String>) {
    let id = id.into();
    let handle = app.clone();
    app.run_on_main_thread(move || crate::tray::handle_menu_event(&handle, &id))
        .unwrap();
}

pub(crate) fn exercise(app: &AppHandle) {
    use crate::computer_control::presentation::Presentation;
    use webcodex_environment::{
        EnvironmentMode, EnvironmentRecord, EnvironmentStore, RuntimeBinaries, SetupRequest,
        ENVIRONMENT_SCHEMA,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        keybd_event, RegisterHotKey, UnregisterHotKey, KEYEVENTF_KEYUP,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GetForegroundWindow, GetWindowDisplayAffinity,
        GetWindowLongPtrW, SetForegroundWindow, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
        WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    let state = app.state::<AppState>();
    // Keep the unrelated service observer out of this private metadata fixture.
    // No Runtime is installed, adopted or started by this test.
    let _core = tauri::async_runtime::block_on(state.core.lock());
    let store =
        EnvironmentStore::open(webcodex_environment::default_environment_dir().unwrap()).unwrap();
    let exe = std::env::current_exe().unwrap();
    let environment_id = "computer-native-smoke";
    store
        .save_environment(&EnvironmentRecord {
            schema_version: ENVIRONMENT_SCHEMA,
            environment_id: environment_id.into(),
            request: SetupRequest {
                service_scope: webcodex_environment::service::ServiceScope::System,
                mode: EnvironmentMode::Join,
                server_url: "http://127.0.0.1:1".into(),
                project: None,
                runner: Some(true),
                runner_display_name: None,
                account: webcodex_environment::current_account().unwrap(),
                binaries: RuntimeBinaries {
                    cli: exe.clone(),
                    server: exe.clone(),
                    runner: exe,
                },
            },
            username: None,
            runner_client_id: None,
            projects: vec![],
            configured: true,
        })
        .unwrap();
    let dir = store.root().join("cu");
    std::fs::create_dir_all(&dir).unwrap();
    let gate = LocalControl::persistent(&dir).unwrap();
    control::serve(&dir, gate.clone()).unwrap();
    state.published.write().unwrap().persistent_environment = Some(environment_id.into());
    let settings = app.state::<Presentation>();
    assert!(!settings.edge.load(Ordering::Acquire));
    assert!(!settings.shortcut.load(Ordering::Acquire));
    assert!(app.get_webview_window("main").is_none());

    let initial = gate.snapshot().unwrap();
    wait_until("tray pause without a main WebView", || {
        menu(app, format!("tray.pause_computer:{}", initial.epoch));
        gate.snapshot().unwrap().paused
    });
    let paused = gate.snapshot().unwrap();
    assert!(
        LocalControl::persistent(&dir)
            .unwrap()
            .snapshot()
            .unwrap()
            .paused
    );
    assert!(gate.begin(true, None).is_err());
    let observation = gate.begin(false, None).unwrap();
    let observed = crate::computer_control::observe(&state.get_state()).unwrap();
    assert!(observed.state.paused && observed.state.observing);
    assert!(!observed.state.controlling);
    assert_eq!(
        crate::computer_control::status_key(Some(&observed.state)),
        "tray.computerPausedObserving"
    );
    drop(observation);
    wait_until("tray resume without a main WebView", || {
        menu(app, format!("tray.resume_computer:{}", paused.epoch));
        !gate.snapshot().unwrap().paused
    });
    // A queued click from the previous menu cannot resume a later local pause.
    let current = gate.snapshot().unwrap();
    gate.set_paused(&current.epoch, true).unwrap();
    menu(app, format!("tray.resume_computer:{}", paused.epoch));
    let fresh = crate::computer_control::observe(&state.get_state()).unwrap();
    assert!(fresh.state.paused);
    wait_until("fresh tray resume", || {
        menu(app, format!("tray.resume_computer:{}", fresh.state.epoch));
        !gate.snapshot().unwrap().paused
    });
    eprintln!("NATIVE_COMPUTER tray_pause_resume=passed main=absent observation=available stale_resume=rejected");

    crate::desktop_shell::show_main_window(app).unwrap();
    wait_until("own foreground window", || {
        app.get_webview_window("main").is_some_and(|window| {
            window.is_visible().unwrap_or(false)
                && window
                    .hwnd()
                    .is_ok_and(|hwnd| unsafe { GetForegroundWindow() } == hwnd.0)
        })
    });
    app.get_webview_window("main")
        .unwrap()
        .set_title("WebCodex Computer Control smoke")
        .unwrap();
    let foreground = unsafe { GetForegroundWindow() };
    let activity = gate.begin(true, None).unwrap();
    menu(app, "tray.computer_edge");
    wait_until("native overlay on every attached monitor", || {
        app.available_monitors()
            .unwrap()
            .iter()
            .enumerate()
            .all(|(index, monitor)| {
                app.get_webview_window(&format!("computer-activity-{index}"))
                    .is_some_and(|window| {
                        window.is_visible().unwrap_or(false)
                            && window
                                .outer_position()
                                .is_ok_and(|position| position == *monitor.position())
                    })
            })
    });
    assert_eq!(
        unsafe { GetForegroundWindow() },
        foreground,
        "overlay stole keyboard focus"
    );
    let monitor_count = app.available_monitors().unwrap().len();
    assert!((1..=16).contains(&monitor_count));
    for index in 0..monitor_count {
        let overlay = app
            .get_webview_window(&format!("computer-activity-{index}"))
            .unwrap();
        let hwnd = overlay.hwnd().unwrap().0;
        let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32;
        assert_ne!(
            style & WS_EX_TRANSPARENT,
            0,
            "overlay must pass pointer input through"
        );
        assert_ne!(style & WS_EX_NOACTIVATE, 0, "overlay must not activate");
        let mut affinity = 0;
        assert_ne!(unsafe { GetWindowDisplayAffinity(hwnd, &mut affinity) }, 0);
        assert_eq!(affinity, 0x11, "overlay must be excluded from capture");
    }
    menu(app, "tray.lightweight");
    wait_until("activity overlay survives unloaded main", || {
        app.get_webview_window("main").is_none()
            && app
                .get_webview_window("computer-activity-0")
                .unwrap()
                .is_visible()
                .unwrap()
    });
    drop(activity);
    wait_until("activity overlay hides after input finishes", || {
        app.get_webview_window("computer-activity-0")
            .is_some_and(|window| !window.is_visible().unwrap())
    });
    menu(app, "tray.computer_edge");
    eprintln!("NATIVE_COMPUTER overlay monitors={monitor_count} pointer=transparent focus=preserved capture=excluded main=absent");

    // A plain owned native window receives any unhandled keys. It contains no
    // WebView: the production shortcut must keep working with main unloaded.
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    app.run_on_main_thread(move || {
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let title: Vec<u16> = "WebCodex Computer shortcut fixture\0"
            .encode_utf16()
            .collect();
        let window = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                120,
                120,
                360,
                120,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        assert!(!window.is_null());
        unsafe { SetForegroundWindow(window) };
        tx.send(window as usize).unwrap();
    })
    .unwrap();
    let keyboard_target = rx.recv_timeout(Duration::from_secs(3)).unwrap();
    wait_until("own native window before global shortcut", || {
        (unsafe { GetForegroundWindow() }) as usize == keyboard_target
    });
    assert!(app.get_webview_window("main").is_none());
    // Confirm availability before enabling, then observe the production listener's
    // real OS registration. Inject only while this isolated test window has focus.
    assert_ne!(
        unsafe { RegisterHotKey(std::ptr::null_mut(), 1340, 0x4007, 0x79) },
        0
    );
    unsafe { UnregisterHotKey(std::ptr::null_mut(), 1340) };
    menu(app, "tray.computer_shortcut");
    wait_until("OS hotkey registered", || {
        let reserved = unsafe { RegisterHotKey(std::ptr::null_mut(), 1340, 0x4007, 0x79) } != 0;
        if reserved {
            unsafe { UnregisterHotKey(std::ptr::null_mut(), 1340) };
        }
        !reserved
            && settings.shortcut.load(Ordering::Acquire)
            && !settings.shortcut_unavailable.load(Ordering::Acquire)
    });
    unsafe {
        for key in [0x11, 0x12, 0x10, 0x79] {
            keybd_event(key, 0, 0, 0);
        }
        for key in [0x79, 0x10, 0x12, 0x11] {
            keybd_event(key, 0, KEYEVENTF_KEYUP, 0);
        }
    }
    wait_until("real WM_HOTKEY pauses local gate", || {
        gate.snapshot().unwrap().paused
    });
    assert!(app.get_webview_window("main").is_none());
    menu(app, "tray.computer_shortcut");
    wait_until("shortcut disabled", || {
        !settings.shortcut.load(Ordering::Acquire)
    });
    assert!(
        gate.snapshot().unwrap().paused,
        "disabling shortcut must not resume input"
    );
    app.run_on_main_thread(move || {
        assert_ne!(unsafe { DestroyWindow(keyboard_target as _) }, 0);
    })
    .unwrap();
    eprintln!("NATIVE_COMPUTER global_hotkey=os_registered pause=confirmed main=absent disable_keeps=paused");
    state.published.write().unwrap().persistent_environment = None;
}
