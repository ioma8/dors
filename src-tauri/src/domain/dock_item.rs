use serde::{Deserialize, Serialize};

use super::AppIdentity;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PinnedApp {
    pub identity: AppIdentity,
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunningApp {
    pub identity: AppIdentity,
    pub display_name: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DockItemView {
    pub identity: AppIdentity,
    pub display_name: String,
    pub is_pinned: bool,
    pub is_running: bool,
    pub is_active: bool,
    pub is_degraded: bool,
}
