use bevy::prelude::*;
use nau_engine::{
    eval::{
        EvalAccumulator, EvalArtifacts, EvalSample, EvalScenario, SCENARIO_NAMES, scenario_named,
    },
    movement::Facing,
};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub(crate) struct EvalOptions {
    pub(crate) scenario: EvalScenario,
    pub(crate) output_dir: PathBuf,
    pub(crate) capture_screenshot: bool,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RunMode {
    Debug,
    Play,
}

impl RunMode {
    pub(crate) fn debug_readout_enabled(self) -> bool {
        matches!(self, Self::Debug)
    }

    pub(crate) fn debug_visuals_enabled(self) -> bool {
        matches!(self, Self::Debug)
    }

    pub(crate) fn debug_visual_toggle_enabled(self) -> bool {
        matches!(self, Self::Debug)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CliAction {
    Run {
        eval: Option<Box<EvalOptions>>,
        mode: RunMode,
    },
    ExportTerrain {
        output_dir: PathBuf,
    },
    ExportVisualContent {
        output_dir: PathBuf,
    },
    ExportWindVisuals {
        output_dir: PathBuf,
    },
    Help,
}

impl CliAction {
    pub(crate) fn from_env() -> Result<Self, String> {
        parse_cli_args(env::args().skip(1))
    }
}

#[derive(Resource, Debug)]
pub(crate) struct EvalRun {
    pub(crate) scenario: EvalScenario,
    pub(crate) samples_path: PathBuf,
    pub(crate) summary_path: PathBuf,
    pub(crate) screenshot_path: Option<PathBuf>,
    pub(crate) checkpoint_captures: Vec<EvalCheckpointCapture>,
    pub(crate) accumulator: EvalAccumulator,
    pub(crate) frame: u32,
    pub(crate) finalized: bool,
    pub(crate) screenshot_wait_frames: u32,
    pub(crate) pending_screenshot_exit_success: Option<bool>,
    pub(crate) io_error: Option<String>,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct EvalMovementBasis {
    pub(crate) frame: u32,
    pub(crate) facing: Option<Facing>,
}

#[derive(Debug)]
pub(crate) struct EvalCheckpointCapture {
    pub(crate) frame: u32,
    pub(crate) name: &'static str,
    pub(crate) path: PathBuf,
    pub(crate) marker_metadata_path: PathBuf,
    pub(crate) captured: bool,
    pub(crate) marker_metadata_written: bool,
}

impl EvalRun {
    pub(crate) fn new(options: EvalOptions) -> std::io::Result<Self> {
        fs::create_dir_all(&options.output_dir)?;

        let samples_path = options.output_dir.join("samples.ndjson");
        let summary_path = options.output_dir.join("summary.json");
        let screenshot_path = options
            .capture_screenshot
            .then(|| options.output_dir.join("final.png"));
        let mut checkpoint_captures = Vec::new();

        remove_existing_file(&summary_path)?;
        if let Some(path) = &screenshot_path {
            remove_existing_file(path)?;
        }
        if options.capture_screenshot {
            let checkpoint_dir = options.output_dir.join("checkpoints");
            remove_existing_dir(&checkpoint_dir)?;
            fs::create_dir_all(&checkpoint_dir)?;
            checkpoint_captures = options
                .scenario
                .checkpoints
                .iter()
                .map(|checkpoint| EvalCheckpointCapture {
                    frame: checkpoint.frame,
                    name: checkpoint.name,
                    path: checkpoint_dir
                        .join(format!("{:04}_{}.png", checkpoint.frame, checkpoint.name)),
                    marker_metadata_path: checkpoint_dir.join(format!(
                        "{:04}_{}.markers.json",
                        checkpoint.frame, checkpoint.name
                    )),
                    captured: false,
                    marker_metadata_written: false,
                })
                .collect();
        }
        File::create(&samples_path)?;

        Ok(Self {
            scenario: options.scenario,
            samples_path,
            summary_path,
            screenshot_path,
            checkpoint_captures,
            accumulator: EvalAccumulator::default(),
            frame: 0,
            finalized: false,
            screenshot_wait_frames: 0,
            pending_screenshot_exit_success: None,
            io_error: None,
        })
    }

    pub(crate) fn record_sample(&mut self, sample: EvalSample) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new().append(true).open(&self.samples_path)?;
        writeln!(file, "{}", sample.to_json())?;
        self.accumulator.observe_for_scenario(sample, self.scenario);
        Ok(())
    }

    pub(crate) fn write_summary(&self) -> Result<bool, std::io::Error> {
        let artifacts = EvalArtifacts {
            summary_json: path_string(&self.summary_path),
            samples_ndjson: path_string(&self.samples_path),
            screenshot_png: self.screenshot_path.as_deref().map(path_string),
            checkpoint_screenshots: self
                .checkpoint_captures
                .iter()
                .map(|checkpoint| path_string(&checkpoint.path))
                .collect(),
            checkpoint_marker_metadata: self
                .checkpoint_captures
                .iter()
                .map(|checkpoint| path_string(&checkpoint.marker_metadata_path))
                .collect(),
        };
        let summary = self.accumulator.summary(self.scenario, artifacts);
        let passed = summary.passed;

        fs::write(&self.summary_path, summary.to_json())?;
        Ok(passed)
    }
}

pub(crate) fn parse_cli_args(args: impl IntoIterator<Item = String>) -> Result<CliAction, String> {
    let mut eval_name = None;
    let mut eval_output = None;
    let mut export_terrain_output = None;
    let mut export_visual_content_output = None;
    let mut export_wind_visuals_output = None;
    let mut capture_screenshot = true;
    let mut saw_eval = false;
    let mut requested_run_mode = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            return Ok(CliAction::Help);
        } else if arg == "--play" {
            set_requested_run_mode(&mut requested_run_mode, RunMode::Play)?;
        } else if arg == "--debug" {
            set_requested_run_mode(&mut requested_run_mode, RunMode::Debug)?;
        } else if arg == "--eval" {
            saw_eval = true;
            eval_name = Some(
                args.next()
                    .ok_or_else(|| "--eval requires a scenario name".to_string())?,
            );
        } else if let Some(value) = arg.strip_prefix("--eval=") {
            saw_eval = true;
            eval_name = Some(value.to_string());
        } else if arg == "--eval-output" {
            eval_output =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    "--eval-output requires a path".to_string()
                })?));
        } else if let Some(value) = arg.strip_prefix("--eval-output=") {
            eval_output = Some(PathBuf::from(value));
        } else if arg == "--eval-no-screenshot" {
            capture_screenshot = false;
        } else if arg == "--export-terrain" {
            export_terrain_output =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    "--export-terrain requires an output directory".to_string()
                })?));
        } else if let Some(value) = arg.strip_prefix("--export-terrain=") {
            export_terrain_output = Some(PathBuf::from(value));
        } else if arg == "--export-visual-content" {
            export_visual_content_output = Some(PathBuf::from(args.next().ok_or_else(|| {
                "--export-visual-content requires an output directory".to_string()
            })?));
        } else if let Some(value) = arg.strip_prefix("--export-visual-content=") {
            export_visual_content_output = Some(PathBuf::from(value));
        } else if arg == "--export-wind-visuals" {
            export_wind_visuals_output = Some(PathBuf::from(args.next().ok_or_else(|| {
                "--export-wind-visuals requires an output directory".to_string()
            })?));
        } else if let Some(value) = arg.strip_prefix("--export-wind-visuals=") {
            export_wind_visuals_output = Some(PathBuf::from(value));
        } else {
            return Err(format!("unknown argument: {arg}"));
        }
    }

    let export_path_count = [
        export_terrain_output.is_some(),
        export_visual_content_output.is_some(),
        export_wind_visuals_output.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if export_path_count > 1 {
        return Err("export paths cannot be combined".to_string());
    }
    if requested_run_mode.is_some() && export_path_count > 0 {
        return Err("run mode flags cannot be combined with export commands".to_string());
    }

    if let Some(output_dir) = export_terrain_output {
        if saw_eval {
            return Err("--export-terrain cannot be combined with --eval".to_string());
        }
        return Ok(CliAction::ExportTerrain { output_dir });
    }
    if let Some(output_dir) = export_visual_content_output {
        if saw_eval {
            return Err("--export-visual-content cannot be combined with --eval".to_string());
        }
        return Ok(CliAction::ExportVisualContent { output_dir });
    }
    if let Some(output_dir) = export_wind_visuals_output {
        if saw_eval {
            return Err("--export-wind-visuals cannot be combined with --eval".to_string());
        }
        return Ok(CliAction::ExportWindVisuals { output_dir });
    }

    let eval = if saw_eval {
        let name = eval_name.unwrap_or_else(|| "baseline_route".to_string());
        let scenario = scenario_named(&name).ok_or_else(|| {
            format!(
                "unknown eval scenario: {name}. available scenarios: {}",
                SCENARIO_NAMES.join(", ")
            )
        })?;
        let output_dir = eval_output.unwrap_or_else(|| PathBuf::from("target/eval").join(name));

        Some(Box::new(EvalOptions {
            scenario,
            output_dir,
            capture_screenshot,
        }))
    } else {
        None
    };

    Ok(CliAction::Run {
        eval,
        mode: requested_run_mode.unwrap_or(RunMode::Play),
    })
}

pub(crate) fn usage() -> String {
    format!(
        "Usage:\n  cargo run\n  cargo run -- --debug\n  cargo run -- --play\n  cargo run -- --eval <scenario> [--eval-output <dir>] [--eval-no-screenshot] [--debug]\n  cargo run -- --export-terrain <dir>\n  cargo run -- --export-visual-content <dir>\n  cargo run -- --export-wind-visuals <dir>\n\nScenarios: {}",
        SCENARIO_NAMES.join(", ")
    )
}

fn set_requested_run_mode(
    requested_run_mode: &mut Option<RunMode>,
    mode: RunMode,
) -> Result<(), String> {
    if requested_run_mode.is_some_and(|requested| requested != mode) {
        return Err("--play and --debug cannot be combined".to_string());
    }

    *requested_run_mode = Some(mode);
    Ok(())
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn remove_existing_file(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(crate) fn remove_existing_dir(path: &Path) -> std::io::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}
