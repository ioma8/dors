use dors::native_app::panel::{PanelConfiguration, panel_configuration};

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
