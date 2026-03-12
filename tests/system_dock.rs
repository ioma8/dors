use dors::native_app::system_dock::{
    DockPreferencePlan, DockPreferencesSnapshot, DockWindowCandidate, adjust_tilesize_guess,
    bottom_reserved_height_from_frames, is_missing_dock_process_error, parse_autohide_output,
    parse_tilesize_output, select_dock_window_height, tilesize_for_desired_real_height,
};

#[test]
fn dock_preference_plan_restores_only_changed_values() {
    let changed = DockPreferencePlan::from_snapshot(
        DockPreferencesSnapshot {
            autohide_before: true,
            tilesize_before: Some(64),
        },
        56,
    );
    let unchanged = DockPreferencePlan::from_snapshot(
        DockPreferencesSnapshot {
            autohide_before: false,
            tilesize_before: Some(56),
        },
        56,
    );

    assert!(!changed.target_autohide());
    assert_eq!(changed.target_tilesize(), 56);
    assert_eq!(changed.restore_autohide(), Some(true));
    assert_eq!(changed.restore_tilesize(), Some(Some(64)));

    assert!(!unchanged.target_autohide());
    assert_eq!(unchanged.target_tilesize(), 56);
    assert_eq!(unchanged.restore_autohide(), None);
    assert_eq!(unchanged.restore_tilesize(), None);
}

#[test]
fn dock_preference_plan_restores_deleted_tilesize_when_missing_before() {
    let plan = DockPreferencePlan::from_snapshot(
        DockPreferencesSnapshot {
            autohide_before: false,
            tilesize_before: None,
        },
        56,
    );

    assert_eq!(plan.restore_autohide(), None);
    assert_eq!(plan.restore_tilesize(), Some(None));
}

#[test]
fn dock_preference_parsers_accept_defaults_output_variants() {
    assert_eq!(parse_autohide_output("1\n"), Some(true));
    assert_eq!(parse_autohide_output("0\n"), Some(false));
    assert_eq!(parse_autohide_output("true\n"), Some(true));
    assert_eq!(parse_autohide_output("false\n"), Some(false));
    assert_eq!(parse_autohide_output("unexpected\n"), None);

    assert_eq!(parse_tilesize_output("56\n"), Some(56));
    assert_eq!(parse_tilesize_output(" 32 \n"), Some(32));
    assert_eq!(parse_tilesize_output("invalid\n"), None);
}

#[test]
fn dock_tilesize_guess_tracks_measured_reserved_height() {
    assert_eq!(adjust_tilesize_guess(56, 77, 56), 41);
    assert_eq!(adjust_tilesize_guess(56, 50, 56), 63);
    assert_eq!(adjust_tilesize_guess(8, 200, 56), 16);
}

#[test]
fn dock_restart_treats_missing_process_as_transient() {
    assert!(is_missing_dock_process_error(
        Some(1),
        "No matching processes belonging to you were found\n"
    ));
    assert!(!is_missing_dock_process_error(Some(1), "different error"));
    assert!(!is_missing_dock_process_error(Some(0), ""));
}

#[test]
fn dock_window_selector_prefers_the_real_bottom_dock_window() {
    let windows = vec![
        DockWindowCandidate {
            owner_name: "Dock".to_string(),
            layer: 20,
            width: 300.0,
            height: 40.0,
            y: 1000.0,
        },
        DockWindowCandidate {
            owner_name: "Dock".to_string(),
            layer: 20,
            width: 1200.0,
            height: 83.0,
            y: 0.0,
        },
        DockWindowCandidate {
            owner_name: "Finder".to_string(),
            layer: 20,
            width: 1400.0,
            height: 90.0,
            y: 0.0,
        },
    ];

    assert_eq!(select_dock_window_height(&windows), Some(83));
}

#[test]
fn bottom_reserved_height_uses_visible_frame_origin_not_total_difference() {
    assert_eq!(bottom_reserved_height_from_frames(0.0, 31.0), 31);
    assert_eq!(bottom_reserved_height_from_frames(100.0, 133.0), 33);
    assert_eq!(bottom_reserved_height_from_frames(0.0, -5.0), 0);
}

#[test]
fn tilesize_interpolation_matches_measured_anchor_points() {
    assert_eq!(tilesize_for_desired_real_height(52), 32);
    assert_eq!(tilesize_for_desired_real_height(90), 64);
    assert_eq!(tilesize_for_desired_real_height(156), 128);
    assert_eq!(tilesize_for_desired_real_height(71), 48);
    assert_eq!(tilesize_for_desired_real_height(123), 96);
}
