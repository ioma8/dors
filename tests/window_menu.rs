use dors::native_app::window_menu::{
    HoverDelayState, HoveredWindow, activation_script_for_window, filtered_hovered_windows,
    parse_window_title_lines, should_show_window_menu,
};

#[test]
fn window_menu_only_shows_for_more_than_one_window() {
    assert!(!should_show_window_menu(0));
    assert!(!should_show_window_menu(1));
    assert!(should_show_window_menu(2));
}

#[test]
fn window_menu_filters_blank_window_titles() {
    let windows = filtered_hovered_windows(&[
        HoveredWindow::new(0, "Main"),
        HoveredWindow::new(1, "   "),
        HoveredWindow::new(2, ""),
        HoveredWindow::new(3, "Preferences"),
    ]);

    assert_eq!(
        windows,
        vec![HoveredWindow::new(0, "Main"), HoveredWindow::new(3, "Preferences")]
    );
}

#[test]
fn window_menu_builds_specific_window_activation_script() {
    let script = activation_script_for_window("Cursor", "README.md");

    assert!(script.contains("application process \"Cursor\""));
    assert!(script.contains("first window whose name is \"README.md\""));
    assert!(script.contains("perform action \"AXRaise\""));
}

#[test]
fn hover_menu_uses_a_short_default_delay() {
    let state = HoverDelayState::new();

    assert_eq!(state.delay_millis(), 180);
}

#[test]
fn hover_menu_invalidates_stale_tokens_after_cancel_or_replace() {
    let mut state = HoverDelayState::new();

    let first = state.schedule_for_item(1);
    assert!(state.is_current(first, 1));

    state.cancel();
    assert!(!state.is_current(first, 1));

    let second = state.schedule_for_item(2);
    assert!(state.is_current(second, 2));
    assert!(!state.is_current(first, 1));
    assert!(!state.is_current(second, 1));
}

#[test]
fn window_menu_parses_line_based_window_output_and_skips_blank_titles() {
    let windows = parse_window_title_lines("Main\n\n   \nPreferences\n");

    assert_eq!(
        windows,
        vec![HoveredWindow::new(0, "Main"), HoveredWindow::new(1, "Preferences")]
    );
}
