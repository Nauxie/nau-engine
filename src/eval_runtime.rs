use crate::play_profile_runtime::PlayProfileScript;
use bevy::prelude::*;
use nau_engine::{
    eval::{
        EvalAccumulator, EvalArtifacts, EvalSample, EvalScenario, SCENARIO_NAMES, scenario_named,
    },
    movement::Facing,
    world::{IslandReviewPlan, IslandReviewPose, IslandReviewView, LodBand, SkyRoute},
};
use serde_json::json;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub(crate) use nau_engine::eval::ISLAND_HERO_GALLERY;
const EVAL_ARTIFACT_FRAME_COOLDOWN: u32 = 1;

#[derive(Clone, Debug)]
pub(crate) struct EvalOptions {
    pub(crate) scenario: EvalScenario,
    pub(crate) output_dir: PathBuf,
    pub(crate) capture_screenshot: bool,
    pub(crate) visible_window: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct PlayProfileOptions {
    pub(crate) output_path: PathBuf,
    pub(crate) duration_secs: Option<f64>,
    pub(crate) script: Option<PlayProfileScript>,
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
        play_profile: Option<PlayProfileOptions>,
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
    pub(crate) island_review_plan: Option<IslandReviewPlan>,
    pub(crate) island_review_manifest_path: Option<PathBuf>,
    pub(crate) accumulator: EvalAccumulator,
    pub(crate) frame: u32,
    pub(crate) finalized: bool,
    pub(crate) screenshot_wait_frames: u32,
    pub(crate) screenshot_ready_frames: u32,
    artifact_frame_cooldown: u32,
    pub(crate) pending_screenshot_exit_success: Option<bool>,
    pub(crate) io_error: Option<String>,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct EvalMovementBasis {
    pub(crate) frame: u32,
    pub(crate) facing: Option<Facing>,
    pub(crate) follow_direction: Option<Vec3>,
}

#[derive(Debug)]
pub(crate) struct EvalCheckpointCapture {
    pub(crate) frame: u32,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) marker_metadata_path: PathBuf,
    pub(crate) capture_requested: bool,
    pub(crate) target_island_name: Option<&'static str>,
    pub(crate) target_view: Option<IslandReviewView>,
    pub(crate) island_index: Option<usize>,
    pub(crate) pose: Option<IslandReviewPose>,
    pub(crate) captured: bool,
    pub(crate) marker_metadata_written: bool,
}

impl EvalRun {
    pub(crate) fn new(options: EvalOptions) -> std::io::Result<Self> {
        fs::create_dir_all(&options.output_dir)?;

        let samples_path = options.output_dir.join("samples.ndjson");
        let summary_path = options.output_dir.join("summary.json");
        let final_screenshot_path = options.output_dir.join("final.png");
        let screenshot_path = options
            .capture_screenshot
            .then(|| final_screenshot_path.clone());
        let gallery_timing = options.scenario.island_hero_gallery_timing();
        let island_review_route = gallery_timing.is_some().then(SkyRoute::default);
        let island_review_plan = island_review_route
            .as_ref()
            .map(IslandReviewPlan::from_route);
        let island_review_manifest_artifact_path =
            options.output_dir.join("island_review_manifest.json");
        let island_review_manifest_path = island_review_plan
            .as_ref()
            .map(|_| island_review_manifest_artifact_path.clone());
        let checkpoint_dir = options.output_dir.join("checkpoints");

        remove_existing_file(&summary_path)?;
        remove_existing_file(&final_screenshot_path)?;
        remove_existing_file(&island_review_manifest_artifact_path)?;
        remove_existing_dir(&checkpoint_dir)?;
        if options.capture_screenshot {
            fs::create_dir_all(&checkpoint_dir)?;
        }
        let checkpoint_captures = if let (Some(plan), Some(route), Some(_)) =
            (&island_review_plan, &island_review_route, gallery_timing)
        {
            island_review_checkpoint_captures(
                plan,
                route,
                &checkpoint_dir,
                options.capture_screenshot,
                options.scenario,
            )
        } else if options.capture_screenshot {
            options
                .scenario
                .checkpoints
                .iter()
                .map(|checkpoint| EvalCheckpointCapture {
                    frame: checkpoint.frame,
                    name: checkpoint.name.to_string(),
                    path: checkpoint_dir
                        .join(format!("{:04}_{}.png", checkpoint.frame, checkpoint.name)),
                    marker_metadata_path: checkpoint_dir.join(format!(
                        "{:04}_{}.markers.json",
                        checkpoint.frame, checkpoint.name
                    )),
                    capture_requested: true,
                    target_island_name: options.scenario.target_island_name,
                    target_view: None,
                    island_index: None,
                    pose: None,
                    captured: false,
                    marker_metadata_written: false,
                })
                .collect()
        } else {
            Vec::new()
        };
        File::create(&samples_path)?;

        let run = Self {
            scenario: options.scenario,
            samples_path,
            summary_path,
            screenshot_path,
            checkpoint_captures,
            island_review_plan,
            island_review_manifest_path,
            accumulator: EvalAccumulator::default(),
            frame: 0,
            finalized: false,
            screenshot_wait_frames: 0,
            screenshot_ready_frames: 0,
            artifact_frame_cooldown: 0,
            pending_screenshot_exit_success: None,
            io_error: None,
        };
        run.write_island_review_manifest()?;
        Ok(run)
    }

    pub(crate) fn island_review_pose(&self) -> Option<IslandReviewPose> {
        let timing = self.scenario.island_hero_gallery_timing()?;
        let capture_index = (self.frame / timing.frames_per_view) as usize;
        self.checkpoint_captures
            .get(capture_index)
            .and_then(|capture| capture.pose)
    }

    pub(crate) fn classify_current_frame_time(&mut self, screenshot_work_active: bool) {
        let checkpoint_due = self.checkpoint_captures.iter().any(|checkpoint| {
            checkpoint.capture_requested && !checkpoint.captured && checkpoint.frame == self.frame
        });
        let artifact_work_active = screenshot_work_active || checkpoint_due;
        let artifact_frame = artifact_work_active || self.artifact_frame_cooldown > 0;
        self.artifact_frame_cooldown = if artifact_work_active {
            EVAL_ARTIFACT_FRAME_COOLDOWN
        } else {
            self.artifact_frame_cooldown.saturating_sub(1)
        };

        if artifact_frame {
            self.accumulator
                .reclassify_latest_runtime_frame_as_eval_artifact();
        }
    }

    pub(crate) fn record_sample(&mut self, sample: EvalSample) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new().append(true).open(&self.samples_path)?;
        writeln!(file, "{}", sample.to_json())?;
        self.accumulator.observe_for_scenario(sample, self.scenario);
        Ok(())
    }

    pub(crate) fn write_summary(&self) -> Result<bool, std::io::Error> {
        self.write_island_review_manifest()?;
        let artifacts = EvalArtifacts {
            summary_json: path_string(&self.summary_path),
            samples_ndjson: path_string(&self.samples_path),
            screenshot_png: self.screenshot_path.as_deref().map(path_string),
            checkpoint_screenshots: self
                .checkpoint_captures
                .iter()
                .filter(|checkpoint| checkpoint.capture_requested)
                .map(|checkpoint| path_string(&checkpoint.path))
                .collect(),
            checkpoint_marker_metadata: self
                .checkpoint_captures
                .iter()
                .filter(|checkpoint| checkpoint.capture_requested)
                .map(|checkpoint| path_string(&checkpoint.marker_metadata_path))
                .collect(),
        };
        let summary = self.accumulator.summary(self.scenario, artifacts);
        let passed = summary.passed;

        fs::write(&self.summary_path, summary.to_json())?;
        Ok(passed)
    }

    pub(crate) fn write_island_review_manifest(&self) -> std::io::Result<()> {
        let (Some(plan), Some(path)) = (
            self.island_review_plan.as_ref(),
            self.island_review_manifest_path.as_ref(),
        ) else {
            return Ok(());
        };
        let islands = plan
            .islands
            .iter()
            .map(|island| {
                json!({
                    "island_index": island.island_index,
                    "island_name": island.island_name,
                    "island_slug": island.island_slug,
                    "epithet": island.epithet,
                    "environmental_story": island.environmental_story,
                    "capture_count": island.views.len(),
                })
            })
            .collect::<Vec<_>>();
        let captures = self
            .checkpoint_captures
            .iter()
            .map(|checkpoint| {
                let island_index = checkpoint
                    .island_index
                    .expect("gallery checkpoints must retain their island index");
                let island = &plan.islands[island_index];
                let pose = checkpoint
                    .pose
                    .expect("gallery checkpoints must retain their review pose");
                debug_assert_eq!(checkpoint.target_island_name, Some(island.island_name));
                json!({
                    "island_index": island_index,
                    "island_name": island.island_name,
                    "island_slug": island.island_slug,
                    "frame": checkpoint.frame,
                    "checkpoint": checkpoint.name,
                    "target_island": checkpoint.target_island_name,
                    "view": pose.view.label(),
                    "approach_island": pose.approach_island_name,
                    "player_position": vec3_json(pose.player_position),
                    "camera_position": vec3_json(pose.camera_position),
                    "camera_target": vec3_json(pose.camera_target),
                    "expected_lod": lod_band_label(pose.expected_lod),
                    "png_path": path_string(&checkpoint.path),
                    "sidecar_path": path_string(&checkpoint.marker_metadata_path),
                    "capture_requested": checkpoint.capture_requested,
                    "screenshot_requested": checkpoint.captured,
                    "sidecar_written": checkpoint.marker_metadata_written,
                    "png_exists": checkpoint.path.is_file(),
                    "sidecar_exists": checkpoint.marker_metadata_path.is_file(),
                })
            })
            .collect::<Vec<_>>();
        debug_assert_eq!(captures.len(), plan.capture_count());
        let timing = self
            .scenario
            .island_hero_gallery_timing()
            .expect("gallery manifest requires gallery timing");
        let manifest = json!({
            "scenario": self.scenario.name,
            "island_count": plan.islands.len(),
            "capture_count": plan.capture_count(),
            "settle_frames": timing.settle_frames,
            "hold_frames": timing.hold_frames,
            "frames_per_view": timing.frames_per_view,
            "islands": islands,
            "captures": captures,
        });

        fs::write(path, serde_json::to_vec_pretty(&manifest)?)
    }
}

fn island_review_checkpoint_captures(
    plan: &IslandReviewPlan,
    route: &SkyRoute,
    checkpoint_dir: &Path,
    capture_requested: bool,
    scenario: EvalScenario,
) -> Vec<EvalCheckpointCapture> {
    let timing = scenario
        .island_hero_gallery_timing()
        .expect("gallery captures require gallery timing");
    plan.islands
        .iter()
        .flat_map(|island| {
            island
                .views
                .iter()
                .enumerate()
                .map(move |(view_index, pose)| {
                    let capture_index = island.island_index
                        * nau_engine::world::ISLAND_REVIEW_VIEWS_PER_ISLAND
                        + view_index;
                    let frame = timing
                        .capture_frame(capture_index)
                        .expect("gallery plan capture count must match timing");
                    let pose = runtime_island_review_pose(*pose, route);
                    let name = format!(
                        "island_{:02}_{}_{}",
                        island.island_index,
                        island.island_slug,
                        pose.view.label()
                    );
                    EvalCheckpointCapture {
                        frame,
                        path: checkpoint_dir.join(format!("{frame:04}_{name}.png")),
                        marker_metadata_path: checkpoint_dir
                            .join(format!("{frame:04}_{name}.markers.json")),
                        name,
                        capture_requested,
                        target_island_name: Some(island.island_name),
                        target_view: Some(pose.view),
                        island_index: Some(island.island_index),
                        pose: Some(pose),
                        captured: false,
                        marker_metadata_written: false,
                    }
                })
        })
        .collect()
}

fn runtime_island_review_pose(mut pose: IslandReviewPose, route: &SkyRoute) -> IslandReviewPose {
    if pose.view == IslandReviewView::Near {
        pose.player_position.y = route.ground_at(pose.player_position).floor_y;
    }
    pose
}

fn vec3_json(value: Vec3) -> serde_json::Value {
    json!([value.x, value.y, value.z])
}

const fn lod_band_label(lod: LodBand) -> &'static str {
    match lod {
        LodBand::Near => "near",
        LodBand::Mid => "mid",
        LodBand::Far => "far",
    }
}

pub(crate) fn parse_cli_args(args: impl IntoIterator<Item = String>) -> Result<CliAction, String> {
    let mut eval_name = None;
    let mut eval_output = None;
    let mut export_terrain_output = None;
    let mut export_visual_content_output = None;
    let mut export_wind_visuals_output = None;
    let mut play_profile_output = None;
    let mut play_profile_duration_secs = None;
    let mut play_profile_script = None;
    let mut capture_screenshot = true;
    let mut visible_window = false;
    let mut saw_eval = false;
    let mut saw_eval_option = false;
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
            saw_eval_option = true;
            eval_output =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    "--eval-output requires a path".to_string()
                })?));
        } else if let Some(value) = arg.strip_prefix("--eval-output=") {
            saw_eval_option = true;
            eval_output = Some(PathBuf::from(value));
        } else if arg == "--eval-no-screenshot" {
            saw_eval_option = true;
            capture_screenshot = false;
        } else if arg == "--eval-visible-window" {
            saw_eval_option = true;
            visible_window = true;
        } else if arg == "--play-profile" {
            play_profile_output =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    "--play-profile requires an output file".to_string()
                })?));
        } else if let Some(value) = arg.strip_prefix("--play-profile=") {
            play_profile_output = Some(PathBuf::from(value));
        } else if arg == "--play-profile-duration" {
            play_profile_duration_secs =
                Some(parse_play_profile_duration(&args.next().ok_or_else(
                    || "--play-profile-duration requires seconds".to_string(),
                )?)?);
        } else if let Some(value) = arg.strip_prefix("--play-profile-duration=") {
            play_profile_duration_secs = Some(parse_play_profile_duration(value)?);
        } else if arg == "--play-profile-script" {
            play_profile_script =
                Some(PlayProfileScript::parse(&args.next().ok_or_else(
                    || "--play-profile-script requires a script name".to_string(),
                )?)?);
        } else if let Some(value) = arg.strip_prefix("--play-profile-script=") {
            play_profile_script = Some(PlayProfileScript::parse(value)?);
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
    if saw_eval_option && !saw_eval {
        return Err("eval options require --eval".to_string());
    }
    if play_profile_duration_secs.is_some() && play_profile_output.is_none() {
        return Err("--play-profile-duration requires --play-profile".to_string());
    }
    if play_profile_script.is_some() && play_profile_output.is_none() {
        return Err("--play-profile-script requires --play-profile".to_string());
    }
    if play_profile_output.is_some() && saw_eval {
        return Err("--play-profile cannot be combined with --eval".to_string());
    }
    if play_profile_output.is_some() && export_path_count > 0 {
        return Err("--play-profile cannot be combined with export commands".to_string());
    }
    if play_profile_output.is_some() && requested_run_mode == Some(RunMode::Debug) {
        return Err("--play-profile requires --play mode".to_string());
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
            visible_window,
        }))
    } else {
        None
    };

    Ok(CliAction::Run {
        eval,
        mode: requested_run_mode.unwrap_or(RunMode::Play),
        play_profile: play_profile_output.map(|output_path| PlayProfileOptions {
            output_path,
            duration_secs: play_profile_duration_secs,
            script: play_profile_script,
        }),
    })
}

pub(crate) fn usage() -> String {
    format!(
        "Usage:\n  cargo run\n  cargo run -- --debug\n  cargo run -- --play\n  cargo run --release -- --play --play-profile <file> [--play-profile-duration <seconds>] [--play-profile-script <freeflight|ground_traversal>]\n  cargo run -- --eval <scenario> [--eval-output <dir>] [--eval-no-screenshot] [--eval-visible-window] [--debug]\n  cargo run -- --export-terrain <dir>\n  cargo run -- --export-visual-content <dir>\n  cargo run -- --export-wind-visuals <dir>\n\nScenarios: {}",
        SCENARIO_NAMES.join(", ")
    )
}

fn parse_play_profile_duration(value: &str) -> Result<f64, String> {
    let duration_secs = value.parse::<f64>().map_err(|_| {
        "--play-profile-duration requires a positive finite number of seconds".to_string()
    })?;

    if duration_secs.is_finite() && duration_secs > 0.0 {
        Ok(duration_secs)
    } else {
        Err("--play-profile-duration requires a positive finite number of seconds".to_string())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::world::{ISLAND_REVIEW_VIEWS_PER_ISLAND, IslandReviewView, SkyRoute};
    use std::{
        collections::HashSet,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn gallery_output_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        env::temp_dir().join(format!(
            "nau_island_hero_gallery_{label}_{}_{}",
            process::id(),
            nonce
        ))
    }

    #[test]
    fn gallery_run_materializes_all_catalog_captures_and_manifest_metadata() {
        let output_dir = gallery_output_dir("captures");
        let scenario = scenario_named(ISLAND_HERO_GALLERY).expect("gallery scenario");
        let mut run = EvalRun::new(EvalOptions {
            scenario,
            output_dir: output_dir.clone(),
            capture_screenshot: true,
            visible_window: false,
        })
        .expect("gallery run");
        let plan = run.island_review_plan.as_ref().expect("review plan");
        let timing = scenario
            .island_hero_gallery_timing()
            .expect("gallery timing");
        let route = SkyRoute::default();

        assert_eq!(plan.islands.len(), 41);
        assert_eq!(run.checkpoint_captures.len(), 123);
        assert_eq!(
            run.checkpoint_captures.len(),
            plan.islands.len() * ISLAND_REVIEW_VIEWS_PER_ISLAND
        );
        assert_eq!(
            run.checkpoint_captures
                .iter()
                .map(|capture| capture.name.as_str())
                .collect::<HashSet<_>>()
                .len(),
            123
        );

        for (capture_index, capture) in run.checkpoint_captures.iter().enumerate() {
            let island_index = capture_index / ISLAND_REVIEW_VIEWS_PER_ISLAND;
            let view_index = capture_index % ISLAND_REVIEW_VIEWS_PER_ISLAND;
            let island = &plan.islands[island_index];
            let pose = runtime_island_review_pose(island.views[view_index], &route);

            assert_eq!(capture.frame, timing.capture_frame(capture_index).unwrap());
            assert_eq!(capture.target_island_name, Some(island.island_name));
            assert_eq!(capture.target_view, Some(pose.view));
            assert_eq!(capture.island_index, Some(island_index));
            assert_eq!(capture.pose, Some(pose));
            if pose.view == IslandReviewView::Near {
                assert!(
                    (pose.player_position.y - route.ground_at(pose.player_position).floor_y).abs()
                        <= f32::EPSILON,
                    "{} near review pose must be grounded",
                    island.island_name
                );
            }
            assert!(capture.capture_requested);
            assert!(
                capture
                    .path
                    .ends_with(format!("{:04}_{}.png", capture.frame, capture.name))
            );
        }

        let sample_capture = &run.checkpoint_captures[73];
        run.frame = sample_capture.frame;
        assert_eq!(run.island_review_pose(), sample_capture.pose);

        let manifest_path = run
            .island_review_manifest_path
            .as_ref()
            .expect("manifest path");
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(manifest_path).expect("manifest bytes"))
                .expect("manifest json");
        assert_eq!(manifest["island_count"], 41);
        assert_eq!(manifest["capture_count"], 123);
        assert_eq!(manifest["settle_frames"], 32);
        assert_eq!(manifest["hold_frames"], 4);
        assert_eq!(manifest["frames_per_view"], 36);
        assert_eq!(
            manifest["islands"].as_array().expect("island array").len(),
            41
        );
        let captures = manifest["captures"].as_array().expect("capture array");
        assert_eq!(captures.len(), 123);
        assert!(captures.iter().all(|capture| {
            capture["target_island"].is_string()
                && capture["view"].is_string()
                && capture["player_position"]
                    .as_array()
                    .is_some_and(|pose| pose.len() == 3)
                && capture["camera_position"]
                    .as_array()
                    .is_some_and(|pose| pose.len() == 3)
                && capture["camera_target"]
                    .as_array()
                    .is_some_and(|pose| pose.len() == 3)
                && capture["expected_lod"].is_string()
                && capture["png_path"].is_string()
                && capture["sidecar_path"].is_string()
                && capture["capture_requested"] == true
                && capture["screenshot_requested"] == false
                && capture["sidecar_written"] == false
        }));
        assert_eq!(captures[0]["view"], IslandReviewView::Near.label());
        assert_eq!(captures[1]["view"], IslandReviewView::Mid.label());
        assert_eq!(captures[2]["view"], IslandReviewView::Traversal.label());

        let late_capture_path = run
            .checkpoint_captures
            .last()
            .expect("final gallery capture")
            .path
            .clone();
        fs::write(&late_capture_path, b"late checkpoint screenshot").expect("late screenshot");
        run.write_island_review_manifest()
            .expect("refreshed gallery manifest");
        let refreshed_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(manifest_path).expect("refreshed manifest bytes"))
                .expect("refreshed manifest json");
        assert_eq!(
            refreshed_manifest["captures"][122]["png_exists"], true,
            "manifest rewrites must observe screenshots that finish after the initial write"
        );

        let _ = fs::remove_dir_all(output_dir);
    }

    #[test]
    fn existing_scenarios_keep_static_checkpoint_capture_behavior() {
        let output_dir = gallery_output_dir("baseline");
        let scenario = scenario_named("baseline_route").expect("baseline scenario");
        let run = EvalRun::new(EvalOptions {
            scenario,
            output_dir: output_dir.clone(),
            capture_screenshot: true,
            visible_window: false,
        })
        .expect("baseline run");

        assert!(run.island_review_plan.is_none());
        assert!(run.island_review_manifest_path.is_none());
        assert_eq!(run.checkpoint_captures.len(), scenario.checkpoints.len());
        assert!(run.checkpoint_captures.iter().all(|capture| {
            capture.capture_requested
                && capture.target_island_name == scenario.target_island_name
                && capture.target_view.is_none()
                && capture.island_index.is_none()
                && capture.pose.is_none()
        }));

        let _ = fs::remove_dir_all(output_dir);
    }

    #[test]
    fn screenshot_disabled_run_clears_stale_final_and_checkpoint_artifacts() {
        let output_dir = gallery_output_dir("cleanup");
        let checkpoint_dir = output_dir.join("checkpoints");
        fs::create_dir_all(&checkpoint_dir).expect("checkpoint dir");
        fs::write(output_dir.join("final.png"), b"stale final").expect("stale final");
        fs::write(checkpoint_dir.join("stale.png"), b"stale checkpoint").expect("stale checkpoint");
        fs::write(checkpoint_dir.join("stale.markers.json"), b"{}").expect("stale sidecar");

        let scenario = scenario_named(ISLAND_HERO_GALLERY).expect("gallery scenario");
        let run = EvalRun::new(EvalOptions {
            scenario,
            output_dir: output_dir.clone(),
            capture_screenshot: false,
            visible_window: false,
        })
        .expect("gallery run without screenshots");

        assert!(run.screenshot_path.is_none());
        assert!(!output_dir.join("final.png").exists());
        assert!(!checkpoint_dir.exists());
        assert!(
            run.checkpoint_captures
                .iter()
                .all(|capture| !capture.capture_requested)
        );

        drop(run);
        let manifest_path = output_dir.join("island_review_manifest.json");
        fs::write(&manifest_path, b"stale manifest").expect("stale manifest");
        EvalRun::new(EvalOptions {
            scenario: scenario_named("baseline_route").expect("baseline scenario"),
            output_dir: output_dir.clone(),
            capture_screenshot: false,
            visible_window: false,
        })
        .expect("baseline run without screenshots");
        assert!(!manifest_path.exists());

        let _ = fs::remove_dir_all(output_dir);
    }

    #[test]
    fn capture_work_and_cleanup_frames_are_excluded_from_runtime_timing() {
        let output_dir = gallery_output_dir("artifact_timing");
        let scenario = scenario_named("baseline_route").expect("baseline scenario");
        let mut run = EvalRun::new(EvalOptions {
            scenario,
            output_dir: output_dir.clone(),
            capture_screenshot: true,
            visible_window: false,
        })
        .expect("baseline run");
        run.frame = run.checkpoint_captures[0].frame;

        run.accumulator.observe_frame_time_ms(200.0);
        run.classify_current_frame_time(false);
        run.checkpoint_captures[0].captured = true;
        run.frame += 1;
        run.accumulator.observe_frame_time_ms(150.0);
        run.classify_current_frame_time(false);
        run.frame += 1;
        run.accumulator.observe_frame_time_ms(10.0);
        run.classify_current_frame_time(false);

        let summary = run.accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: String::new(),
                samples_ndjson: String::new(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        assert_eq!(summary.metrics.runtime_frame_time_sample_count, 1);
        assert_eq!(summary.metrics.max_runtime_frame_time_ms, 10.0);
        assert_eq!(summary.metrics.eval_artifact_frame_time_sample_count, 2);
        assert_eq!(summary.metrics.max_eval_artifact_frame_time_ms, 200.0);

        let _ = fs::remove_dir_all(output_dir);
    }
}
