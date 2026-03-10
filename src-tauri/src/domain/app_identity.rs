use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppIdentity {
    pub bundle_id: Option<String>,
    pub path: PathBuf,
}
