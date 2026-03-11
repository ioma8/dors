use dors::native_app::app::startup_configuration;
use dors::native_app::layout::{PanelPlacement, bottom_center_panel_placement};

#[test]
fn bottom_center_panel_anchors_to_full_monitor_bounds() {
    let placement = bottom_center_panel_placement(0, 0, 1728, 1117, 1180, 168);

    assert_eq!(placement, PanelPlacement { x: 274, y: 949 });
}

#[test]
fn bottom_center_panel_tracks_monitor_offset() {
    let placement = bottom_center_panel_placement(1728, 0, 1728, 1117, 1180, 168);

    assert_eq!(placement, PanelPlacement { x: 2002, y: 949 });
}

#[test]
fn native_app_builds_startup_configuration() {
    let config = startup_configuration();

    assert_eq!(config.activation_policy, "accessory");
    assert!(config.panel_level > 20);
    assert!(config.panel_width > 0);
    assert!(config.panel_height > 0);
}
