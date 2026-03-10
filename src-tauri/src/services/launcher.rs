use std::path::PathBuf;
use std::process::Command;

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

pub fn activate_app(request: &LaunchRequest) -> LaunchResult {
    let Some(bundle_id) = &request.bundle_id else {
        return LaunchResult::ActivationFailed;
    };

    let script = format!("tell application id \"{bundle_id}\" to activate");
    match Command::new("osascript").args(["-e", &script]).status() {
        Ok(status) if status.success() => LaunchResult::Activated,
        Ok(_) | Err(_) => LaunchResult::ActivationFailed,
    }
}

pub fn launch_app(request: &LaunchRequest) -> LaunchResult {
    match Command::new("open").args(["-a"]).arg(&request.path).status() {
        Ok(status) if status.success() => LaunchResult::Launched,
        Ok(_) | Err(_) => LaunchResult::LaunchFailed,
    }
}
