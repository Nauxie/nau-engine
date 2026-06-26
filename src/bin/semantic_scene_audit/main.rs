mod checkpoint;
mod materials;
mod report;
mod thresholds;
mod types;

#[cfg(test)]
mod tests;

use checkpoint::audit_checkpoint_path;
use report::{report_checks, report_json};
use std::{env, path::PathBuf, process};

fn main() {
    let paths = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("Usage: cargo run --bin semantic_scene_audit -- <markers.json> [...]");
        process::exit(2);
    }

    let mut checkpoints = Vec::with_capacity(paths.len());
    for path in &paths {
        match audit_checkpoint_path(path) {
            Ok(checkpoint) => checkpoints.push(checkpoint),
            Err(error) => {
                eprintln!("failed to audit {}: {error}", path.display());
                process::exit(2);
            }
        }
    }

    let checks = report_checks(&checkpoints);
    let passed = checkpoints.iter().all(|checkpoint| checkpoint.passed)
        && checks.iter().all(|check| check.passed);
    println!("{}", report_json(passed, &checks, &checkpoints));
    if !passed {
        process::exit(1);
    }
}
