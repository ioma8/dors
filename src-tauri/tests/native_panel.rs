use dors_tauri_lib::native_app::panel::{panel_configuration, PanelConfiguration};

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
