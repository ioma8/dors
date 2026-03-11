use std::path::PathBuf;

use dors_tauri_lib::native_app::interaction::launch_request_from_model;
use dors_tauri_lib::native_app::view_model::NativeDockItemModel;
use dors_tauri_lib::services::launcher::LaunchRequest;

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
