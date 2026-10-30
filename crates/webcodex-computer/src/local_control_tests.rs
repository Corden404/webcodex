#[test]
fn every_native_effect_is_blocked_even_without_a_runner_request() {
    let runtime = crate::ComputerRuntime::new(crate::ComputerConfig {
        max_encoded_image_bytes: 1024,
    });
    let control = runtime.local_control();
    let epoch = control.snapshot().unwrap().epoch;
    control.set_paused(&epoch, true).unwrap();
    for result in [
        runtime.write_clipboard("fixture"),
        runtime.activate_window("invalid"),
        runtime.launch_application("invalid"),
        runtime.control("invalid", "invalid", crate::ComputerAction::Press),
        runtime.scroll_to_element("invalid", "invalid"),
        runtime.key_input("invalid", "a", &[]),
        runtime.input_text("invalid", "invalid", "fixture"),
        runtime.pointer_effect(crate::PointerAction::Move, "invalid", 1, 0, 0),
    ] {
        assert!(result.unwrap_err().starts_with("computer_control_paused:"));
    }
}
