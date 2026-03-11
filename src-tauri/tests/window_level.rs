use dors_tauri_lib::window_level::{
    DOCK_WINDOW_LEVEL, NONACTIVATING_PANEL_STYLE_MASK, OVERLAY_WINDOW_LEVEL,
    VISIBLE_DOCK_WINDOW_LAYER, overlay_style_mask,
};

#[test]
fn overlay_window_level_is_above_the_system_dock() {
    assert!(OVERLAY_WINDOW_LEVEL > DOCK_WINDOW_LEVEL);
}

#[test]
fn overlay_window_level_is_above_the_visible_dock_window_layer() {
    assert!(OVERLAY_WINDOW_LEVEL > VISIBLE_DOCK_WINDOW_LAYER);
}

#[test]
fn overlay_style_mask_marks_the_window_as_nonactivating() {
    assert_ne!(overlay_style_mask(0) & NONACTIVATING_PANEL_STYLE_MASK, 0);
}
