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
use report::{
    VisualAuditProfile, audit_report_json_for_profile, report_checks_for_profile,
    report_passed_for_profile,
};

const USAGE: &str = "Usage: cargo run --bin visual_audit -- [--profile default|route_marker_optional|close_obstruction|island_gallery] <png> [<png> ...]";

fn main() {
    let (profile, paths) = match parse_args(env::args().skip(1)) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{USAGE}");
            process::exit(2);
        }
    };
    if paths.is_empty() {
        eprintln!("{USAGE}");
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

    let report_checks = report_checks_for_profile(&audits, profile);
    let passed = report_passed_for_profile(&audits, &report_checks, profile);
    println!(
        "{}",
        audit_report_json_for_profile(&report_checks, &audits, profile)
    );
    if !passed {
        process::exit(1);
    }
}

fn parse_args(
    args: impl IntoIterator<Item = String>,
) -> Result<(VisualAuditProfile, Vec<PathBuf>), String> {
    let mut profile = VisualAuditProfile::Default;
    let mut paths = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--profile" {
            let Some(name) = args.next() else {
                return Err("--profile requires a value".to_string());
            };
            profile = VisualAuditProfile::parse(&name)
                .ok_or_else(|| format!("unknown visual audit profile: {name}"))?;
        } else {
            paths.push(PathBuf::from(arg));
        }
    }

    Ok((profile, paths))
}

#[cfg(test)]
#[path = "visual_audit/tests.rs"]
mod tests;
