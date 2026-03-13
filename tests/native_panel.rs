use dors::native_app::panel::{PanelConfiguration, panel_configuration};
use dors::native_app::window_popup::{
    popup_behavior, popup_height, popup_style, WINDOW_POPUP_ITEM_HEIGHT, WINDOW_POPUP_ROW_SPACING,
    WINDOW_POPUP_VERTICAL_PADDING,
};
use objc2_app_kit::NSPopoverBehavior;

#[test]
fn panel_configuration_uses_bottom_overlay_window_semantics() {
    let config = panel_configuration();

    assert_eq!(
        config,
        PanelConfiguration {
            level: 21,
            transparent: true,
            non_activating: true,
            borderless: true,
            ignores_mouse_events: false,
        }
    );
}

#[test]
fn popup_height_wraps_items_with_vertical_padding() {
    assert_eq!(
        popup_height(3),
        WINDOW_POPUP_VERTICAL_PADDING * 2.0
            + WINDOW_POPUP_ITEM_HEIGHT * 3.0
            + WINDOW_POPUP_ROW_SPACING * 2.0
    );
}

#[test]
fn popup_behavior_is_application_defined() {
    assert_eq!(popup_behavior(), NSPopoverBehavior::ApplicationDefined);
}

#[test]
fn popup_style_uses_compact_floating_palette_geometry() {
    let style = popup_style();

    assert!(style.width >= 300.0);
    assert!(style.horizontal_padding >= 12.0);
    assert!(style.vertical_padding >= 12.0);
    assert!(style.row_height >= 30.0);
    assert!(style.row_spacing >= 4.0);
    assert!(style.corner_radius >= 16.0);
}
