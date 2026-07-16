use super::{metrics::SimResult, run_simulation};
use nau_engine::eval::{APP_ONLY_SCENARIO_NAMES, EvalScenario, SCENARIO_NAMES, scenario_named};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Clone, Debug)]
pub(crate) struct SimOptions {
    scenario: EvalScenario,
    output_dir: PathBuf,
}

impl SimOptions {
    pub(crate) fn from_env() -> Result<Self, String> {
        parse_args(env::args().skip(1))
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<SimOptions, String> {
    let mut scenario_name = None;
    let mut output_dir = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Err("help requested".to_string()),
            "--scenario" => {
                scenario_name = Some(
                    args.next()
                        .ok_or_else(|| "--scenario requires a scenario name".to_string())?,
                );
            }
            "--output" => {
                output_dir =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "--output requires a directory".to_string()
                    })?));
            }
            _ if arg.starts_with("--scenario=") => {
                scenario_name = Some(arg.trim_start_matches("--scenario=").to_string());
            }
            _ if arg.starts_with("--output=") => {
                output_dir = Some(PathBuf::from(arg.trim_start_matches("--output=")));
            }
            _ if scenario_name.is_none() => scenario_name = Some(arg),
            _ if output_dir.is_none() => output_dir = Some(PathBuf::from(arg)),
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    let scenario_name = scenario_name.unwrap_or_else(|| "baseline_route".to_string());
    let scenario = scenario_named(&scenario_name).ok_or_else(|| {
        format!(
            "unknown eval scenario: {scenario_name}. available scenarios: {}",
            SCENARIO_NAMES.join(", ")
        )
    })?;
    if APP_ONLY_SCENARIO_NAMES.contains(&scenario.name) {
        return Err(format!(
            "{} is app-only because it depends on Bevy-spawned world-collision proxies; run it without NAU_EVAL_SIM_ONLY=1",
            scenario.name
        ));
    }
    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("target/eval").join(scenario.name));

    Ok(SimOptions {
        scenario,
        output_dir,
    })
}

pub(crate) fn usage() -> String {
    format!(
        "Usage:\n  cargo run --bin traversal_sim_eval -- [scenario] [output_dir]\n  cargo run --bin traversal_sim_eval -- --scenario <scenario> --output <dir>\n\nSimulation-supported scenarios: {}\nApp-only scenarios: {}",
        simulation_scenario_names().join(", "),
        APP_ONLY_SCENARIO_NAMES.join(", ")
    )
}

fn simulation_scenario_names() -> Vec<&'static str> {
    SCENARIO_NAMES
        .iter()
        .copied()
        .filter(|scenario| !APP_ONLY_SCENARIO_NAMES.contains(scenario))
        .collect()
}

pub(crate) fn run_and_write(options: SimOptions) -> Result<(), String> {
    fs::create_dir_all(&options.output_dir)
        .map_err(|error| format!("failed to create output directory: {error}"))?;
    let summary_path = options.output_dir.join("summary.json");
    let samples_path = options.output_dir.join("samples.ndjson");
    remove_existing_file(&summary_path)?;
    remove_existing_file(&samples_path)?;
    File::create(&samples_path)
        .map_err(|error| format!("failed to create samples file: {error}"))?;

    let started = Instant::now();
    let mut result = run_simulation(options.scenario);
    result.elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    result.summary_path = path_string(&summary_path);
    result.samples_path = path_string(&samples_path);

    write_samples(&samples_path, &result)?;
    fs::write(&summary_path, result.to_summary_json())
        .map_err(|error| format!("failed to write summary: {error}"))?;

    eprintln!("traversal sim summary: {}", path_string(&summary_path));
    if result.passed {
        Ok(())
    } else {
        Err(format!(
            "simulation checks failed: {}",
            path_string(&summary_path)
        ))
    }
}

fn write_samples(samples_path: &Path, result: &SimResult) -> Result<(), String> {
    let mut samples_file = OpenOptions::new()
        .append(true)
        .open(samples_path)
        .map_err(|error| format!("failed to open samples file: {error}"))?;
    for sample in &result.samples {
        writeln!(samples_file, "{}", sample.to_json())
            .map_err(|error| format!("failed to write sample: {error}"))?;
    }
    Ok(())
}

fn remove_existing_file(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to remove {}: {error}", path_string(path))),
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::eval::{TERRAIN_BODY_COLLISION_CONTACT, WORLD_COLLISION_CONTACT};

    #[test]
    fn parse_args_rejects_app_only_collision_contact_route() {
        let error = parse_args([WORLD_COLLISION_CONTACT.to_string()])
            .expect_err("world collision contact should be app-only");
        let body_error = parse_args([TERRAIN_BODY_COLLISION_CONTACT.to_string()])
            .expect_err("terrain body collision contact should be app-only");

        assert!(error.contains("app-only"));
        assert!(body_error.contains("app-only"));
        assert!(error.contains("NAU_EVAL_SIM_ONLY"));
        assert!(!simulation_scenario_names().contains(&WORLD_COLLISION_CONTACT));
        assert!(!simulation_scenario_names().contains(&TERRAIN_BODY_COLLISION_CONTACT));
        assert!(usage().contains("App-only scenarios:"));
        assert!(usage().contains(WORLD_COLLISION_CONTACT));
        assert!(usage().contains("terrain_body_collision_contact"));
    }
}
