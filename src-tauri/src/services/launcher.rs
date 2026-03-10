use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LaunchRequest {
    pub bundle_id: Option<String>,
    pub path: PathBuf,
    pub is_running: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LaunchResult {
    Activated,
    ActivationFailed,
    Launched,
    LaunchFailed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LaunchAction {
    Activate,
    Launch,
    LaunchFallback,
    NoOp,
}

pub fn launch_or_activate<Activate, Launch>(
    request: &LaunchRequest,
    activate: Activate,
    launch: Launch,
) -> LaunchAction
where
    Activate: Fn(&LaunchRequest) -> LaunchResult,
    Launch: Fn(&LaunchRequest) -> LaunchResult,
{
    if request.is_running {
        match activate(request) {
            LaunchResult::Activated => LaunchAction::Activate,
            LaunchResult::ActivationFailed => match launch(request) {
                LaunchResult::Launched => LaunchAction::LaunchFallback,
                LaunchResult::LaunchFailed
                | LaunchResult::Activated
                | LaunchResult::ActivationFailed => LaunchAction::NoOp,
            },
            LaunchResult::Launched => LaunchAction::Launch,
            LaunchResult::LaunchFailed => LaunchAction::NoOp,
        }
    } else {
        match launch(request) {
            LaunchResult::Launched => LaunchAction::Launch,
            LaunchResult::Activated => LaunchAction::Activate,
            LaunchResult::ActivationFailed | LaunchResult::LaunchFailed => LaunchAction::NoOp,
        }
    }
}
