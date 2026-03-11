use dors::native_app::dock_item_view::{ITEM_HEIGHT, ITEM_WIDTH, hover_style, item_layout};
use dors::native_app::dock_view::{dock_glass_style, required_height};

#[test]
fn item_layout_keeps_icon_and_indicator_inside_the_button_bounds() {
    let layout = item_layout();

    assert!(layout.button_origin_y >= 0.0);
    assert!(layout.icon_origin_x >= 0.0);
    assert!(layout.icon_origin_y >= 0.0);
    assert!(layout.icon_origin_x + layout.icon_size <= ITEM_WIDTH);
    assert!(layout.icon_origin_y + layout.icon_size <= ITEM_HEIGHT);
    assert!(layout.indicator_origin_y >= 0.0);
}

#[test]
fn dock_glass_style_wraps_the_item_row_inside_panel_height() {
    let style = dock_glass_style();

    assert!(style.horizontal_padding >= 16.0);
    assert!(style.vertical_origin_y >= 0.0);
    assert!(style.height > ITEM_HEIGHT);
    assert!(style.vertical_origin_y + style.height <= required_height() as f64);
}

#[test]
fn hover_style_uses_hover_only_button_chrome() {
    let style = hover_style();

    assert!(style.bordered);
    assert!(!style.transparent);
    assert!(style.hover_only_border);
}
