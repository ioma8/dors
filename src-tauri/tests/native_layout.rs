use dors_tauri_lib::native_app::layout::{bottom_center_panel_placement, PanelPlacement};

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
