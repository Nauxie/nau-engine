#[path = "visual_audit/analysis.rs"]
mod analysis;
#[path = "visual_audit/image_metrics.rs"]
mod image_metrics;
#[path = "visual_audit/pixel_rules.rs"]
mod pixel_rules;
#[path = "visual_audit/report.rs"]
mod report;
#[path = "visual_audit/thresholds.rs"]
mod thresholds;
#[path = "visual_audit/types.rs"]
mod types;

use std::{env, path::PathBuf, process};

use analysis::audit_path;
use report::{audit_report_json, report_checks, report_passed};

fn main() {
    let paths = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("Usage: cargo run --bin visual_audit -- <png> [<png> ...]");
        process::exit(2);
    }

    let mut audits = Vec::with_capacity(paths.len());
    for path in &paths {
        match audit_path(path) {
            Ok(audit) => audits.push(audit),
            Err(error) => {
                eprintln!("failed to audit {}: {error}", path.display());
                process::exit(2);
            }
        }
    }

    let report_checks = report_checks(&audits);
    let passed = report_passed(&audits, &report_checks);
    println!("{}", audit_report_json(passed, &report_checks, &audits));
    if !passed {
        process::exit(1);
    }
}

#[cfg(test)]
#[path = "visual_audit/tests.rs"]
mod tests;
