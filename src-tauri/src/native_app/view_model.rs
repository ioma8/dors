use std::path::PathBuf;

use crate::domain::DockItemView;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeDockItemModel {
    pub key: String,
    pub bundle_id: Option<String>,
    pub path: PathBuf,
    pub display_name: String,
    pub icon_src: String,
    pub shows_indicator: bool,
    pub uses_placeholder_icon: bool,
    pub is_running: bool,
    pub is_active: bool,
    pub is_pinned: bool,
    pub is_degraded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconcileOp {
    Insert { index: usize, item: NativeDockItemModel },
    Update { index: usize, item: NativeDockItemModel },
    Replace { index: usize, item: NativeDockItemModel },
    Remove { index: usize },
}

impl NativeDockItemModel {
    pub fn from_dock_item(item: &DockItemView) -> Self {
        Self {
            key: item.stable_key(),
            bundle_id: item.identity.bundle_id.clone(),
            path: item.identity.path.clone(),
            display_name: item.display_name.clone(),
            icon_src: item.icon_src.clone(),
            shows_indicator: item.is_running,
            uses_placeholder_icon: item.icon_src.is_empty(),
            is_running: item.is_running,
            is_active: item.is_active,
            is_pinned: item.is_pinned,
            is_degraded: item.is_degraded,
        }
    }
}

pub fn build_models(items: &[DockItemView]) -> Vec<NativeDockItemModel> {
    items.iter().map(NativeDockItemModel::from_dock_item).collect()
}

pub fn reconcile_items(
    current: &[NativeDockItemModel],
    next: &[NativeDockItemModel],
) -> Vec<ReconcileOp> {
    let shared_len = current.len().min(next.len());
    let mut ops = Vec::new();

    for index in 0..shared_len {
        if current[index].key != next[index].key {
            ops.push(ReconcileOp::Replace {
                index,
                item: next[index].clone(),
            });
            continue;
        }

        if current[index] != next[index] {
            ops.push(ReconcileOp::Update {
                index,
                item: next[index].clone(),
            });
        }
    }

    for (offset, item) in next.iter().skip(shared_len).cloned().enumerate() {
        ops.push(ReconcileOp::Insert {
            index: shared_len + offset,
            item,
        });
    }

    for index in (next.len()..current.len()).rev() {
        ops.push(ReconcileOp::Remove { index });
    }

    ops
}
