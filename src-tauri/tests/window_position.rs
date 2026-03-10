use dors_tauri_lib::window_position::{bottom_center_placement, DockWindowPlacement};

#[test]
fn window_position_centers_the_dock_on_the_full_monitor_bounds() {
    let placement = bottom_center_placement(0, 0, 1728, 1117, 1180, 168, 0);

    assert_eq!(placement, DockWindowPlacement { x: 274, y: 949 });
}

#[test]
fn window_position_respects_monitor_offset_and_bottom_edge() {
    let placement = bottom_center_placement(1728, 0, 1728, 1117, 1180, 168, 0);

    assert_eq!(placement, DockWindowPlacement { x: 2002, y: 949 });
}
