use dors::native_app::window_clamper::{
    ScreenFrame, WindowFrame, WorkingArea, build_allowed_work_area, clamp_window_frame,
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
            y: 83,
            width: 1920,
            height: 966,
        }
    );
}

#[test]
fn clamp_window_frame_moves_window_up_when_only_bottom_overflows() {
    let area = WorkingArea {
        x: 0,
        y: 83,
        width: 1920,
        height: 966,
    };
    let frame = WindowFrame {
        x: 100,
        y: 40,
        width: 1200,
        height: 700,
    };

    assert_eq!(
        clamp_window_frame(frame, area),
        Some(WindowFrame {
            x: 100,
            y: 83,
            width: 1200,
            height: 700,
        })
    );
}

#[test]
fn clamp_window_frame_shrinks_window_that_is_taller_than_allowed_area() {
    let area = WorkingArea {
        x: 0,
        y: 83,
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
            y: 83,
            width: 1512,
            height: 966,
        })
    );
}

#[test]
fn clamp_window_frame_leaves_compliant_window_unchanged() {
    let area = WorkingArea {
        x: 0,
        y: 83,
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
