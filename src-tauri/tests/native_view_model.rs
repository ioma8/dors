use std::path::PathBuf;

use dors_tauri_lib::domain::{AppIdentity, DockItemView};
use dors_tauri_lib::native_app::view_model::{reconcile_items, NativeDockItemModel, ReconcileOp};

fn dock_item(
    bundle_id: Option<&str>,
    path: &str,
    display_name: &str,
    is_running: bool,
    is_active: bool,
) -> DockItemView {
    DockItemView {
        identity: AppIdentity {
            bundle_id: bundle_id.map(ToOwned::to_owned),
            path: PathBuf::from(path),
        },
        display_name: display_name.to_string(),
        icon_src: String::new(),
        is_pinned: true,
        is_running,
        is_active,
        is_degraded: false,
    }
}

#[test]
fn reconcile_items_detects_insert_remove_and_update() {
    let current = vec![
        NativeDockItemModel::from_dock_item(&dock_item(
            Some("com.apple.finder"),
            "/System/Library/CoreServices/Finder.app",
            "Finder",
            true,
            false,
        )),
        NativeDockItemModel::from_dock_item(&dock_item(
            Some("com.apple.Notes"),
            "/System/Applications/Notes.app",
            "Notes",
            true,
            false,
        )),
    ];
    let next = vec![
        NativeDockItemModel::from_dock_item(&dock_item(
            Some("com.apple.finder"),
            "/System/Library/CoreServices/Finder.app",
            "Finder",
            true,
            true,
        )),
        NativeDockItemModel::from_dock_item(&dock_item(
            None,
            "/Applications/WezTerm.app",
            "WezTerm",
            true,
            false,
        )),
    ];

    let ops = reconcile_items(&current, &next);

    assert_eq!(
        ops,
        vec![
            ReconcileOp::Update {
                index: 0,
                item: next[0].clone(),
            },
            ReconcileOp::Replace {
                index: 1,
                item: next[1].clone(),
            },
        ]
    );
}
