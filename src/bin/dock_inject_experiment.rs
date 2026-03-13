#[cfg(target_os = "macos")]
fn main() {
    use dors::dock_injection_experiment::{
        InjectionExperimentConfig, manual_experiment_steps, materialize_staged_files, staged_files,
    };

    let staging_root = std::env::temp_dir().join("dors-dock-injection");
    let config = InjectionExperimentConfig::new(&staging_root);
    let files = staged_files(&config);

    println!("dock injection experiment");
    println!("staging root: {}", config.paths.root.display());
    println!("restore strategy: restart Dock");
    println!("requirements:");
    println!("- likely SIP concessions");
    println!("- likely manual privileged steps");
    println!("- restore is expected via Dock restart");
    println!("staged files:");
    for file in files {
        println!("- {}", file.path.display());
    }

    match materialize_staged_files(&config) {
        Ok(paths) => {
            println!("materialized:");
            for path in paths {
                println!("- {}", path.display());
            }
        }
        Err(error) => {
            eprintln!("failed to materialize experiment files: {error}");
            std::process::exit(1);
        }
    }

    println!("manual experiment steps:");
    for step in manual_experiment_steps(&config) {
        println!("- {step}");
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("dock_inject_experiment is only supported on macOS");
    std::process::exit(1);
}
