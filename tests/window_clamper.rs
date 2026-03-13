use dors::native_app::window_clamper::{
    ClampOperation, CustomZoomTracker, ScreenFrame, WindowCandidate, WindowFrame, WindowSignal,
    WorkingArea,
    build_allowed_work_area, build_clamp_script_preview, build_query_windows_script_preview,
    build_apply_operation_script_preview, clamp_window_frame, normalize_ax_value,
    should_clamp_candidate,
};

#[test]
fn allowed_work_area_reserves_menu_bar_and_custom_dock() {
    let screen = ScreenFrame {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };

    let area = build_allowed_work_area(screen, 31, 83);

    assert_eq!(
        area,
        WorkingArea {
            x: 0,
            y: 31,
            width: 1920,
            height: 966,
        }
    );
}

#[test]
fn clamp_window_frame_moves_window_up_when_only_bottom_overflows() {
    let area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 966,
    };
    let frame = WindowFrame {
        x: 100,
        y: 400,
        width: 1200,
        height: 700,
    };

    assert_eq!(
        clamp_window_frame(frame, area),
        Some(WindowFrame {
            x: 100,
            y: 297,
            width: 1200,
            height: 700,
        })
    );
}

#[test]
fn clamp_window_frame_shrinks_window_that_is_taller_than_allowed_area() {
    let area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 966,
    };
    let frame = WindowFrame {
        x: 0,
        y: 0,
        width: 1512,
        height: 1100,
    };

    assert_eq!(
        clamp_window_frame(frame, area),
        Some(WindowFrame {
            x: 0,
            y: 31,
            width: 1512,
            height: 966,
        })
    );
}

#[test]
fn clamp_window_frame_leaves_compliant_window_unchanged() {
    let area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 966,
    };
    let frame = WindowFrame {
        x: 200,
        y: 120,
        width: 1000,
        height: 800,
    };

    assert_eq!(clamp_window_frame(frame, area), None);
}

#[test]
fn should_clamp_candidate_accepts_normal_resizable_visible_window() {
    let screen = ScreenFrame {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };
    let candidate = WindowCandidate {
        owner_name: "Firefox".to_string(),
        stable_key: "Firefox::main".to_string(),
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1512,
            height: 1018,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: false,
        is_visible: true,
    };

    assert!(should_clamp_candidate(&candidate, screen));
}

#[test]
fn should_clamp_candidate_rejects_fullscreen_or_non_resizable_or_tiny_windows() {
    let screen = ScreenFrame {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };

    let fullscreen = WindowCandidate {
        owner_name: "Firefox".to_string(),
        stable_key: "Firefox::fullscreen".to_string(),
        frame: WindowFrame {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: true,
        is_visible: true,
    };
    let fixed = WindowCandidate {
        owner_name: "Finder".to_string(),
        stable_key: "Finder::fixed".to_string(),
        frame: WindowFrame {
            x: 238,
            y: 33,
            width: 920,
            height: 436,
        },
        is_standard: true,
        is_resizable: false,
        is_fullscreen: false,
        is_visible: true,
    };
    let tiny = WindowCandidate {
        owner_name: "Tooltip".to_string(),
        stable_key: "Tooltip::tiny".to_string(),
        frame: WindowFrame {
            x: 1500,
            y: 20,
            width: 120,
            height: 40,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: false,
        is_visible: true,
    };

    assert!(!should_clamp_candidate(&fullscreen, screen));
    assert!(!should_clamp_candidate(&fixed, screen));
    assert!(!should_clamp_candidate(&tiny, screen));
}

#[test]
fn clamp_script_does_not_skip_windows_based_on_axresizable_attribute() {
    let script = build_clamp_script_preview(WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 966,
    });

    assert!(!script.contains("AXResizable"));
    assert!(!script.contains("isResizable"));
}

#[test]
fn query_windows_script_closes_outer_system_events_tell_block() {
    let script = build_query_windows_script_preview();

    assert!(script.contains("tell application \"System Events\""));
    assert!(script.contains("set procNames to name of every application process"));
    assert!(script.contains("set winIndex to 0"));
    assert!(script.trim_end().ends_with("end tell"));
}

#[test]
fn query_windows_script_for_pid_filters_by_unix_id() {
    let script = dors::native_app::window_clamper::build_query_windows_for_pid_script_preview(812);

    assert!(script.contains("first application process whose unix id is 812"));
    assert!(script.trim_end().ends_with("end tell"));
}

#[test]
fn apply_operation_script_targets_window_by_index() {
    let script = build_apply_operation_script_preview(
        "idea",
        2,
        WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 993,
        },
    );

    assert!(script.contains("set win to item 2 of windows"));
    assert!(!script.contains("windowTitle is"));
    assert!(!script.contains("xPos is"));
    assert!(script.trim_end().ends_with("end try\nend tell\nend tell"));
}

#[test]
fn custom_zoom_tracker_toggles_between_custom_zoom_and_restore() {
    let native_area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 1018,
    };
    let custom_area = WorkingArea {
        x: 0,
        y: 31,
        width: 1920,
        height: 966,
    };
    let mut tracker = CustomZoomTracker::default();
    let regular = WindowCandidate {
        owner_name: "Code".to_string(),
        stable_key: "Code::doc".to_string(),
        frame: WindowFrame {
            x: 120,
            y: 80,
            width: 1200,
            height: 800,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: false,
        is_visible: true,
    };
    assert_eq!(tracker.plan_operation(&regular, native_area, custom_area), None);

    let native_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 1018,
        },
        ..regular.clone()
    };
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );

    let custom_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 966,
        },
        ..regular.clone()
    };
    assert_eq!(tracker.plan_operation(&custom_zoomed, native_area, custom_area), None);
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(regular.frame))
    );
}

#[test]
fn custom_zoom_tracker_uses_latest_regular_frame_as_restore_target() {
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
    let mut tracker = CustomZoomTracker::default();
    let key = "firefox::main".to_string();
    let regular = WindowCandidate {
        owner_name: "firefox".to_string(),
        stable_key: key.clone(),
        frame: WindowFrame {
            x: 120,
            y: 90,
            width: 1400,
            height: 820,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: false,
        is_visible: true,
    };
    assert_eq!(tracker.plan_operation(&regular, native_area, custom_area), None);

    let updated_regular = WindowCandidate {
        frame: WindowFrame {
            x: 80,
            y: 70,
            width: 1500,
            height: 860,
        },
        ..regular.clone()
    };
    assert_eq!(
        tracker.plan_operation(&updated_regular, native_area, custom_area),
        None
    );

    let native_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 1049,
        },
        ..regular
    };
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(updated_regular.frame))
    );
}

#[test]
fn normalize_ax_value_drops_missing_value_marker() {
    assert_eq!(normalize_ax_value("missing value"), "");
    assert_eq!(normalize_ax_value("  missing value  "), "");
    assert_eq!(normalize_ax_value("Document.txt"), "Document.txt");
}

#[test]
fn custom_zoom_tracker_clears_managed_state_after_manual_resize() {
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
    let mut tracker = CustomZoomTracker::default();
    let regular = WindowCandidate {
        owner_name: "firefox".to_string(),
        stable_key: "firefox::main".to_string(),
        frame: WindowFrame {
            x: 120,
            y: 90,
            width: 1400,
            height: 820,
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
    let custom_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 993,
        },
        ..regular.clone()
    };
    let manually_resized = WindowCandidate {
        frame: WindowFrame {
            x: 40,
            y: 60,
            width: 1300,
            height: 700,
        },
        ..regular.clone()
    };

    assert_eq!(tracker.plan_operation(&regular, native_area, custom_area), None);
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
    assert_eq!(tracker.plan_operation(&custom_zoomed, native_area, custom_area), None);

    for _ in 0..5 {
        tracker.observe_window_frame(&manually_resized, native_area, custom_area);
    }
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(manually_resized.frame))
    );
}

#[test]
fn custom_zoom_tracker_keeps_state_during_short_settling_frames() {
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
    let mut tracker = CustomZoomTracker::default();
    let regular = WindowCandidate {
        owner_name: "firefox".to_string(),
        stable_key: "firefox::main".to_string(),
        frame: WindowFrame {
            x: 120,
            y: 90,
            width: 1400,
            height: 820,
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
    let settling_frame = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 1010,
        },
        ..regular.clone()
    };

    assert_eq!(tracker.plan_operation(&regular, native_area, custom_area), None);
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );

    for _ in 0..3 {
        tracker.observe_window_frame(&settling_frame, native_area, custom_area);
    }

    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(regular.frame))
    );
}

#[test]
fn custom_zoom_tracker_rehydrates_state_for_existing_custom_zoomed_window() {
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
    let mut tracker = CustomZoomTracker::default();
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
    let custom_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 993,
        },
        ..regular.clone()
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

    assert_eq!(tracker.plan_operation(&regular, native_area, custom_area), None);
    assert_eq!(tracker.plan_operation(&custom_zoomed, native_area, custom_area), None);
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(regular.frame))
    );
}

#[test]
fn custom_zoom_tracker_skips_duplicate_frame_checks() {
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
    let mut tracker = CustomZoomTracker::default();
    let regular = WindowCandidate {
        owner_name: "Code".to_string(),
        stable_key: "pid-123::window-1".to_string(),
        frame: WindowFrame {
            x: 120,
            y: 80,
            width: 1200,
            height: 800,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: false,
        is_visible: true,
    };

    assert!(!tracker.should_skip_frame(&regular));
    assert_eq!(tracker.plan_operation(&regular, native_area, custom_area), None);
    assert!(tracker.should_skip_frame(&regular));
}

#[test]
fn custom_zoom_tracker_fast_noops_when_custom_frame_is_already_managed() {
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
    let mut tracker = CustomZoomTracker::default();
    let regular = WindowCandidate {
        owner_name: "Code".to_string(),
        stable_key: "pid-123::window-1".to_string(),
        frame: WindowFrame {
            x: 120,
            y: 80,
            width: 1200,
            height: 800,
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
    let custom_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 993,
        },
        ..regular.clone()
    };

    assert_eq!(tracker.plan_operation(&regular, native_area, custom_area), None);
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
    assert_eq!(tracker.plan_operation(&custom_zoomed, native_area, custom_area), None);
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(WindowFrame {
            x: 120,
            y: 80,
            width: 1200,
            height: 800,
        }))
    );
}

#[test]
fn managed_zoom_reducer_handles_passive_and_active_signals() {
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
    let mut tracker = CustomZoomTracker::default();
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

    assert_eq!(
        tracker.handle_window_signal(
            &regular,
            native_area,
            custom_area,
            WindowSignal::PassiveObservation
        ),
        None
    );
    assert_eq!(
        tracker.handle_window_signal(
            &native_zoomed,
            native_area,
            custom_area,
            WindowSignal::GeometryChanged
        ),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
}

#[test]
fn startup_initialization_records_regular_frame_without_operation() {
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
    let mut tracker = CustomZoomTracker::default();
    let regular = WindowCandidate {
        owner_name: "idea".to_string(),
        stable_key: "idea::window-1".to_string(),
        frame: WindowFrame {
            x: 120,
            y: 90,
            width: 1500,
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

    assert_eq!(
        tracker.plan_startup_operation(&regular, native_area, custom_area),
        None
    );
    assert_eq!(
        tracker.plan_startup_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(regular.frame))
    );
}

#[test]
fn startup_initialization_reclamps_native_zoom_without_unknown_restore_frame() {
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
    let mut tracker = CustomZoomTracker::default();
    let native_zoomed = WindowCandidate {
        owner_name: "idea".to_string(),
        stable_key: "idea::window-1".to_string(),
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 1049,
        },
        is_standard: true,
        is_resizable: true,
        is_fullscreen: false,
        is_visible: true,
    };
    let custom_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 993,
        },
        ..native_zoomed.clone()
    };

    assert_eq!(
        tracker.plan_startup_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
    assert_eq!(
        tracker.plan_operation(&custom_zoomed, native_area, custom_area),
        None
    );
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(native_zoomed.frame))
    );
}

#[test]
fn startup_initialization_does_not_seed_custom_zoomed_window_as_regular() {
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
    let mut tracker = CustomZoomTracker::default();
    let custom_zoomed = WindowCandidate {
        owner_name: "slack".to_string(),
        stable_key: "pid-1039::window-1".to_string(),
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 993,
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
        ..custom_zoomed.clone()
    };

    assert_eq!(
        tracker.plan_startup_operation(&custom_zoomed, native_area, custom_area),
        None
    );
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
}

#[test]
fn startup_initialization_preserves_restore_frame_for_reclamped_window() {
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
    let mut tracker = CustomZoomTracker::default();
    let legacy_maximized = WindowCandidate {
        owner_name: "firefox".to_string(),
        stable_key: "pid-1767::window-1".to_string(),
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
    let custom_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 993,
        },
        ..legacy_maximized.clone()
    };
    let native_zoomed = WindowCandidate {
        frame: WindowFrame {
            x: 0,
            y: 31,
            width: 1920,
            height: 1049,
        },
        ..legacy_maximized.clone()
    };

    assert_eq!(
        tracker.plan_startup_operation(&legacy_maximized, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
    assert_eq!(
        tracker.plan_operation(&custom_zoomed, native_area, custom_area),
        None
    );
    assert_eq!(
        tracker.plan_operation(&native_zoomed, native_area, custom_area),
        Some(ClampOperation::Restore(legacy_maximized.frame))
    );
}

#[test]
fn startup_initialization_reclamps_legacy_dock_maximized_window() {
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
    let mut tracker = CustomZoomTracker::default();
    let legacy_maximized = WindowCandidate {
        owner_name: "slack".to_string(),
        stable_key: "pid-1039::window-1".to_string(),
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

    assert_eq!(
        tracker.plan_startup_operation(&legacy_maximized, native_area, custom_area),
        Some(ClampOperation::ResizeToArea(custom_area))
    );
}
