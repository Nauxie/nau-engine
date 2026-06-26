mod artifact;
mod checks;
mod manifest;
mod thresholds;

#[cfg(test)]
mod tests;

use manifest::audit_manifest_path;
use serde_json::Value;
use std::{env, path::PathBuf, process};

fn main() {
    let args = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if args.len() != 1 {
        eprintln!("Usage: cargo run --bin terrain_export_audit -- <manifest.json>");
        process::exit(2);
    }

    match audit_manifest_path(&args[0]) {
        Ok(report) => {
            let passed = report
                .get("passed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("audit report should serialize")
            );
            if !passed {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("terrain export audit failed: {error}");
            process::exit(2);
        }
    }
}
