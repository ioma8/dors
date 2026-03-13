use std::path::PathBuf;

use dors::native_app::interaction::{
    launch_request_from_model, should_present_hover_menu, should_schedule_hover_menu_dismiss,
};
use dors::native_app::view_model::NativeDockItemModel;
use dors::native_app::window_menu::HoveredWindow;
use dors::services::launcher::LaunchRequest;

#[test]
fn interaction_model_builds_launch_request_for_clicked_item() {
    let model = NativeDockItemModel {
        key: "bundle:com.github.wez.wezterm".to_string(),
        bundle_id: Some("com.github.wez.wezterm".to_string()),
        path: PathBuf::from("/Applications/WezTerm.app"),
        display_name: "WezTerm".to_string(),
        icon_src: String::new(),
        shows_indicator: true,
        uses_placeholder_icon: true,
        is_running: true,
        is_active: false,
        is_pinned: false,
        is_degraded: false,
    };

    assert_eq!(
        launch_request_from_model(&model),
        LaunchRequest {
            bundle_id: Some("com.github.wez.wezterm".to_string()),
            path: PathBuf::from("/Applications/WezTerm.app"),
            is_running: true,
        }
    );
}

#[test]
fn interaction_only_requests_hover_menu_for_multi_window_apps() {
    assert!(!should_present_hover_menu(&[HoveredWindow::new(0, "Main")]));
    assert!(should_present_hover_menu(&[
        HoveredWindow::new(0, "Main"),
        HoveredWindow::new(1, "Preferences"),
    ]));
}

#[test]
fn interaction_only_schedules_hover_menu_dismiss_after_leaving_both_surfaces() {
    assert!(!should_schedule_hover_menu_dismiss(false, false));
    assert!(should_schedule_hover_menu_dismiss(true, false));
    assert!(!should_schedule_hover_menu_dismiss(true, true));
}
