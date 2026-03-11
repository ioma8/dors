use dors::native_app::dock_item_view::{ITEM_HEIGHT, ITEM_WIDTH, hover_style, item_layout};
use dors::native_app::dock_view::{content_start_x, dock_glass_style, required_height};

#[test]
fn item_layout_keeps_icon_and_indicator_inside_the_button_bounds() {
    let layout = item_layout();

    assert!(layout.button_origin_y >= 0.0);
    assert!(layout.icon_origin_x >= 0.0);
    assert!(layout.icon_origin_y >= 0.0);
    assert!(layout.icon_origin_x + layout.icon_size <= ITEM_WIDTH);
    assert!(layout.icon_origin_y + layout.icon_size <= ITEM_HEIGHT);
    assert!(layout.indicator_origin_y >= 0.0);
    assert!(layout.indicator_origin_y < ITEM_HEIGHT);
}

#[test]
fn dock_glass_style_wraps_the_item_row_inside_panel_height() {
    let style = dock_glass_style();

    assert!(style.horizontal_padding >= 16.0);
    assert!(style.vertical_origin_y >= 0.0);
    assert!(style.height >= ITEM_HEIGHT);
    assert!(style.corner_radius >= 24.0);
    assert!(style.container_spacing >= 0.0);
    assert!(style.tint_alpha > 0.0);
    assert!(style.vertical_origin_y + style.height <= required_height() as f64);
}

#[test]
fn hover_style_uses_hover_only_button_chrome() {
    let style = hover_style(false);

    assert!(!style.active_ring);
    assert!(style.hover_shadow);
}

#[test]
fn active_item_style_keeps_visible_chrome() {
    let style = hover_style(true);

    assert!(style.active_ring);
    assert!(style.hover_shadow);
}

#[test]
fn content_start_x_centers_items_inside_the_glass_width() {
    assert_eq!(content_start_x(600.0, 652.0), 26.0);
    assert_eq!(content_start_x(220.0, 180.0), 0.0);
}
