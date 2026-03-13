use dors::native_app::system_dock::DockWindowCandidate;
use dors::private_dock_experiment::{
    DockSuppressionPlan, DockVisualSnapshot, DockVisualTarget, build_restore_snapshots,
    plan_visual_suppression,
};

fn candidate(owner: &str, layer: i32, width: f64, height: f64, x: f64, y: f64) -> DockWindowCandidate {
    DockWindowCandidate {
        owner_name: owner.to_string(),
        window_name: None,
        layer,
        width,
        height,
        x,
        y,
    }
}

#[test]
fn plan_visual_suppression_prefers_visible_dock_strip_candidates() {
    let windows = vec![
        candidate("Dock", 20, 1920.0, 1080.0, 0.0, 0.0),
        candidate("Window Server", 24, 1920.0, 30.0, 0.0, 0.0),
        candidate("Window Server", 24, 1512.0, 33.0, -1512.0, 0.0),
        candidate("Ovládací centrum", 25, 42.0, 30.0, 1700.0, 0.0),
    ];

    let plan = plan_visual_suppression(&windows);

    assert_eq!(
        plan.targets,
        vec![
            DockVisualTarget {
                window_id: 0,
                owner_name: "Dock".to_string(),
                layer: 20,
                width: 1920.0,
                height: 1080.0,
                x: 0.0,
                y: 0.0,
                alpha: 1.0,
            },
            DockVisualTarget {
                window_id: 1,
                owner_name: "Window Server".to_string(),
                layer: 24,
                width: 1920.0,
                height: 30.0,
                x: 0.0,
                y: 0.0,
                alpha: 1.0,
            },
            DockVisualTarget {
                window_id: 2,
                owner_name: "Window Server".to_string(),
                layer: 24,
                width: 1512.0,
                height: 33.0,
                x: -1512.0,
                y: 0.0,
                alpha: 1.0,
            },
        ]
    );
}

#[test]
fn plan_visual_suppression_returns_empty_when_no_strip_candidates_exist() {
    let windows = vec![candidate("Finder", 0, 920.0, 436.0, 238.0, 33.0)];

    let plan = plan_visual_suppression(&windows);

    assert_eq!(plan, DockSuppressionPlan { targets: vec![] });
}

#[test]
fn build_restore_snapshots_preserves_original_alpha_by_window_id() {
    let targets = vec![
        DockVisualTarget {
            window_id: 41,
            owner_name: "Window Server".to_string(),
            layer: 24,
            width: 1920.0,
            height: 30.0,
            x: 0.0,
            y: 0.0,
            alpha: 1.0,
        },
        DockVisualTarget {
            window_id: 42,
            owner_name: "Window Server".to_string(),
            layer: 24,
            width: 1512.0,
            height: 33.0,
            x: -1512.0,
            y: 0.0,
            alpha: 0.75,
        },
    ];

    let snapshots = build_restore_snapshots(&targets);

    assert_eq!(
        snapshots,
        vec![
            DockVisualSnapshot {
                window_id: 41,
                alpha_before: 1.0,
            },
            DockVisualSnapshot {
                window_id: 42,
                alpha_before: 0.75,
            },
        ]
    );
}
