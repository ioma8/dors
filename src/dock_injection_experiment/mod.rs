use std::path::PathBuf;
use std::{fs, io};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreStrategy {
    RestartDock,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentPaths {
    pub root: PathBuf,
    pub payload_bundle: PathBuf,
    pub payload_binary: PathBuf,
    pub payload_plist: PathBuf,
    pub loader_binary: PathBuf,
}

impl ExperimentPaths {
    pub fn new(staging_root: impl Into<PathBuf>) -> Self {
        let root = staging_root.into();
        let payload_bundle = root.join("dock-hide.payload.bundle");
        let payload_contents = payload_bundle.join("Contents");
        let payload_macos = payload_contents.join("MacOS");

        Self {
            root: root.clone(),
            payload_bundle,
            payload_binary: payload_macos.join("dock-hide-payload"),
            payload_plist: payload_contents.join("Info.plist"),
            loader_binary: root.join("dock-hide-loader"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InjectionExperimentConfig {
    pub experiment_name: String,
    pub restore_strategy: RestoreStrategy,
    pub paths: ExperimentPaths,
}

impl InjectionExperimentConfig {
    pub fn new(staging_root: impl Into<PathBuf>) -> Self {
        let experiment_name = "dock-hide-experiment".to_string();
        Self {
            experiment_name,
            restore_strategy: RestoreStrategy::RestartDock,
            paths: ExperimentPaths::new(staging_root),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedFile {
    pub path: PathBuf,
    pub contents: String,
}

pub fn payload_info_plist(bundle_identifier: &str, executable_name: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>{bundle_identifier}</string>
  <key>CFBundleExecutable</key>
  <string>{executable_name}</string>
  <key>CFBundlePackageType</key>
  <string>BNDL</string>
</dict>
</plist>
"#
    )
}

pub fn payload_source() -> String {
    r#"#import <Cocoa/Cocoa.h>

__attribute__((constructor))
static void dors_dock_hide_payload_init(void) {
    @autoreleasepool {
        NSLog(@"[dors-dock-hide] payload loaded into Dock");

        NSApplication *app = [NSApplication sharedApplication];
        NSArray<NSWindow *> *windows = [app windows];
        NSLog(@"[dors-dock-hide] dock windows count=%lu", (unsigned long)[windows count]);

        for (NSWindow *window in windows) {
            NSLog(@"[dors-dock-hide] window=%@ level=%ld alpha=%f frame=%@",
                  window,
                  (long)[window level],
                  [window alphaValue],
                  NSStringFromRect([window frame]));

            [window setAlphaValue:0.0];
            [window orderOut:nil];
        }
    }
}
"#
    .to_string()
}

pub fn loader_source(payload_dylib_path: &str) -> String {
    format!(
        r#"#import <Cocoa/Cocoa.h>
#import <mach/mach.h>
#import <mach/mach_vm.h>
#import <dlfcn.h>
#include <stdio.h>
#include <unistd.h>

static const char *payload_path = "{payload_dylib_path}";

static pid_t get_dock_pid(void) {{
    NSArray *list = [NSRunningApplication runningApplicationsWithBundleIdentifier:@"com.apple.dock"];
    if ([list count] == 1) {{
        NSRunningApplication *dock = list[0];
        if ([dock isFinishedLaunching] == YES) {{
            return [dock processIdentifier];
        }}
    }}
    return 0;
}}

int main(void) {{
    pid_t pid = get_dock_pid();
    if (!pid) {{
        fprintf(stderr, "could not locate Dock.app pid\n");
        return 1;
    }}

    mach_port_t task = 0;
    if (task_for_pid(mach_task_self(), pid, &task) != KERN_SUCCESS) {{
        fprintf(stderr, "task_for_pid failed for Dock pid %d\n", pid);
        return 1;
    }}

    fprintf(stdout, "dock pid=%d task=%u payload=%s\n", pid, task, payload_path);
    fprintf(stdout, "manual remote thread injection still needs to be implemented in this experiment\n");
    return 2;
}}
"#
    )
}

pub fn staged_files(config: &InjectionExperimentConfig) -> Vec<StagedFile> {
    vec![
        StagedFile {
            path: config.paths.payload_plist.clone(),
            contents: payload_info_plist(
                "com.jakubkolcar.dors.dock-hide-payload",
                "dock-hide-payload",
            ),
        },
        StagedFile {
            path: config.paths.payload_binary.with_extension("m"),
            contents: payload_source(),
        },
        StagedFile {
            path: config.paths.loader_binary.with_extension("m"),
            contents: loader_source(&config.paths.payload_binary.display().to_string()),
        },
    ]
}

pub fn materialize_staged_files(config: &InjectionExperimentConfig) -> Result<Vec<PathBuf>, io::Error> {
    let mut written = Vec::new();

    for file in staged_files(config) {
        if let Some(parent) = file.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file.path, file.contents)?;
        written.push(file.path);
    }

    Ok(written)
}

pub fn manual_experiment_steps(config: &InjectionExperimentConfig) -> Vec<String> {
    vec![
        format!(
            "Stage files were written under: {}",
            config.paths.root.display()
        ),
        "Next step requires a manual privileged Dock injection path; this binary does not perform it automatically.".to_string(),
        format!(
            "Payload placeholder bundle: {}",
            config.paths.payload_bundle.display()
        ),
        format!(
            "Compile payload source: clang -dynamiclib -framework Cocoa -o {} {}",
            config.paths.payload_binary.display(),
            config.paths.payload_binary.with_extension("m").display()
        ),
        format!(
            "Compile loader source: clang -framework Cocoa -o {} {}",
            config.paths.loader_binary.display(),
            config.paths.loader_binary.with_extension("m").display()
        ),
        format!(
            "Run loader manually (likely with sudo / SIP caveats): {}",
            config.paths.loader_binary.display()
        ),
        "After a manual load attempt, capture: loader output, any Dock/payload logs, whether Dock became invisible, and whether `killall Dock` restored the baseline.".to_string(),
    ]
}
