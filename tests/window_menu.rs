use dors::native_app::window_menu::{
    HoveredWindow, activation_script_for_window, filtered_hovered_windows, should_show_window_menu,
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
