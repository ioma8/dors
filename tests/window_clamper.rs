use dors::native_app::window_clamper::{
    ScreenFrame, WindowCandidate, WindowFrame, WorkingArea, build_allowed_work_area,
    build_clamp_script_preview, clamp_window_frame, should_clamp_candidate,
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
