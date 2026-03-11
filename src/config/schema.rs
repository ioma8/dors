use serde::{Deserialize, Serialize};

use crate::domain::PinnedApp;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DockConfig {
    pub pinned_apps: Vec<PinnedApp>,
}
