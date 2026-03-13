use dors::native_app::ax_window_manager::{
    AxEventDispatcher, AxObserverRegistry, ManagedWindowEvent, ObservedWindowId, WindowEventKind,
    apply_window_event_to_tracker, normalize_notification_name,
};
use dors::native_app::window_clamper::{
    ClampOperation, CustomZoomTracker, WindowCandidate, WindowFrame, WindowSignal, WorkingArea,
};

#[test]
fn notification_names_map_to_internal_event_kinds() {
    assert_eq!(
        normalize_notification_name("AXFocusedWindowChanged"),
        Some(WindowEventKind::FocusedWindowChanged)
    );
    assert_eq!(
        normalize_notification_name("AXMainWindowChanged"),
        Some(WindowEventKind::MainWindowChanged)
    );
    assert_eq!(
        normalize_notification_name("AXMoved"),
        Some(WindowEventKind::Moved)
    );
    assert_eq!(
        normalize_notification_name("AXWindowMoved"),
        Some(WindowEventKind::Moved)
    );
    assert_eq!(
        normalize_notification_name("AXResized"),
        Some(WindowEventKind::Resized)
    );
    assert_eq!(
        normalize_notification_name("AXWindowResized"),
        Some(WindowEventKind::Resized)
    );
    assert_eq!(
        normalize_notification_name("AXWindowMiniaturized"),
        Some(WindowEventKind::Miniaturized)
    );
    assert_eq!(normalize_notification_name("AXSomethingElse"), None);
}

#[test]
fn observed_window_id_uses_app_local_index_identity() {
    let window_id = ObservedWindowId::new(812, 3);

    assert_eq!(window_id.pid(), 812);
    assert_eq!(window_id.window_index(), 3);
}

#[test]
fn observer_registry_tracks_registered_and_removed_pids() {
    let mut registry = AxObserverRegistry::default();

    assert!(registry.register_pid(812));
    assert!(!registry.register_pid(812));
    assert!(registry.is_registered(812));

    assert!(registry.remove_pid(812));
    assert!(!registry.is_registered(812));
    assert!(!registry.remove_pid(812));
}

#[test]
fn managed_window_event_routes_geometry_change_into_tracker() {
    let native_area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 1049,
    };
    let custom_area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 993,
    };
    let regular = WindowCandidate {
        owner_name: "idea".to_string(),
        stable_key: "idea::window-1".to_string(),
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 893,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: false,
        is_visible: true,
    };
    let native_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 1049,
        },
        ..regular.clone()
    };
    let mut tracker = CustomZoomTracker::default();

    assert_eq!(
        apply_window_event_to_tracker(
            &mut tracker,
            &regular,
            ManagedWindowEvent {
                window_id: ObservedWindowId::new(812, 1),
                signal: WindowSignal::PassiveObservation,
            },
            native_area,
            custom_area,
        ),
        None
    );
    assert_eq!(
        apply_window_event_to_tracker(
            &mut tracker,
            &native_zoomed,
            ManagedWindowEvent {
                window_id: ObservedWindowId::new(812, 1),
                signal: WindowSignal::GeometryChanged,
            },
            native_area,
            custom_area,
        ),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
}

#[test]
fn event_dispatcher_applies_event_with_stored_work_areas() {
    let native_area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 1049,
    };
    let custom_area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 993,
    };
    let regular = WindowCandidate {
        owner_name: "idea".to_string(),
        stable_key: "idea::window-1".to_string(),
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 893,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: false,
        is_visible: true,
    };
    let native_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 1049,
        },
        ..regular.clone()
    };
    let mut dispatcher = AxEventDispatcher::new(native_area, custom_area);

    assert_eq!(
        dispatcher.apply_event(
            &regular,
            ManagedWindowEvent {
                window_id: ObservedWindowId::new(812, 1),
                signal: WindowSignal::PassiveObservation,
            }
        ),
        None
    );
    assert_eq!(
        dispatcher.apply_event(
            &native_zoomed,
            ManagedWindowEvent {
                window_id: ObservedWindowId::new(812, 1),
                signal: WindowSignal::GeometryChanged,
            }
        ),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
}
