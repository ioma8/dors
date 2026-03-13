use dors::dock_injection_experiment::{
    ExperimentPaths, InjectionExperimentConfig, RestoreStrategy, manual_experiment_steps,
    materialize_staged_files, payload_info_plist, staged_files,
};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn experiment_paths_use_expected_bundle_layout() {
    let paths = ExperimentPaths::new("/tmp/dors-dock-injection");

    assert_eq!(paths.root, PathBuf::from("/tmp/dors-dock-injection"));
    assert_eq!(
        paths.payload_bundle,
        PathBuf::from("/tmp/dors-dock-injection/dock-hide.payload.bundle")
    );
    assert_eq!(
        paths.payload_binary,
        PathBuf::from(
            "/tmp/dors-dock-injection/dock-hide.payload.bundle/Contents/MacOS/dock-hide-payload"
        )
    );
    assert_eq!(
        paths.payload_plist,
        PathBuf::from("/tmp/dors-dock-injection/dock-hide.payload.bundle/Contents/Info.plist")
    );
    assert_eq!(
        paths.loader_binary,
        PathBuf::from("/tmp/dors-dock-injection/dock-hide-loader")
    );
}

#[test]
fn injection_experiment_defaults_to_restart_dock_restore_strategy() {
    let config = InjectionExperimentConfig::new("/tmp/dors-dock-injection");

    assert_eq!(config.experiment_name, "dock-hide-experiment");
    assert_eq!(config.restore_strategy, RestoreStrategy::RestartDock);
}

#[test]
fn payload_info_plist_includes_identifier_and_executable() {
    let plist = payload_info_plist("com.example.payload", "payload-bin");

    assert!(plist.contains("<string>com.example.payload</string>"));
    assert!(plist.contains("<string>payload-bin</string>"));
    assert!(plist.contains("<string>BNDL</string>"));
}

#[test]
fn staged_files_cover_payload_bundle_and_loader() {
    let config = InjectionExperimentConfig::new("/tmp/dors-dock-injection");

    let files = staged_files(&config);

    assert_eq!(files.len(), 3);
    assert_eq!(files[0].path, config.paths.payload_plist);
    assert_eq!(files[1].path, config.paths.payload_binary.with_extension("m"));
    assert_eq!(files[2].path, config.paths.loader_binary.with_extension("m"));
}

#[test]
fn materialize_staged_files_writes_expected_files_to_disk() {
    let dir = tempdir().expect("tempdir");
    let config = InjectionExperimentConfig::new(dir.path());

    let written = materialize_staged_files(&config).expect("materialize staged files");

    assert_eq!(written.len(), 3);
    assert!(config.paths.payload_plist.exists());
    assert!(config.paths.payload_binary.with_extension("m").exists());
    assert!(config.paths.loader_binary.with_extension("m").exists());
}

#[test]
fn manual_experiment_steps_reference_staged_paths_and_restore_expectation() {
    let config = InjectionExperimentConfig::new("/tmp/dors-dock-injection");

    let steps = manual_experiment_steps(&config);

    assert!(steps.iter().any(|step| step.contains("/tmp/dors-dock-injection")));
    assert!(steps.iter().any(|step| step.contains("Payload placeholder bundle")));
    assert!(steps.iter().any(|step| step.contains("Compile payload source")));
    assert!(steps.iter().any(|step| step.contains("Compile loader source")));
    assert!(steps.iter().any(|step| step.contains("killall Dock")));
}
