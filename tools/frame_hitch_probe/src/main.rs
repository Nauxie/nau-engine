use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::window::{PresentMode, PrimaryWindow, WindowPlugin};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

const DEFAULT_DURATION_SECS: f64 = 30.0;
const HITCH_THRESHOLD_MS: f64 = 25.0;
const MIN_FOCUSED_RATIO: f64 = 0.95;
const WARMUP_EXCLUDED_SECS: f64 = 3.0;

#[derive(Clone, Copy)]
enum ProbeVariant {
    Default,
    NoAudio,
    NoGilrs,
    NoAudioGilrs,
    NoPipelinedRendering,
}

impl ProbeVariant {
    fn from_env() -> Result<Self, String> {
        match std::env::var("NAU_HITCH_PROBE_VARIANT").as_deref() {
            Ok("default") | Err(std::env::VarError::NotPresent) => Ok(Self::Default),
            Ok("no-audio") => Ok(Self::NoAudio),
            Ok("no-gilrs") => Ok(Self::NoGilrs),
            Ok("no-audio-gilrs") => Ok(Self::NoAudioGilrs),
            Ok("no-pipelined-rendering") => Ok(Self::NoPipelinedRendering),
            Ok(value) => Err(format!(
                "unsupported NAU_HITCH_PROBE_VARIANT {value:?}; expected default, no-audio, \
                 no-gilrs, no-audio-gilrs, or no-pipelined-rendering"
            )),
            Err(error) => Err(format!("failed to read NAU_HITCH_PROBE_VARIANT: {error}")),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::NoAudio => "no-audio",
            Self::NoGilrs => "no-gilrs",
            Self::NoAudioGilrs => "no-audio-gilrs",
            Self::NoPipelinedRendering => "no-pipelined-rendering",
        }
    }
}

#[derive(Resource)]
struct ProbeState {
    variant: ProbeVariant,
    output_path: PathBuf,
    duration_secs: f64,
    started_at: Instant,
    previous_frame_at: Option<Instant>,
    frame_times_ms: Vec<f64>,
    steady_frame_times_ms: Vec<f64>,
    hitch_events: Vec<HitchEvent>,
    focused_samples: usize,
    unfocused_samples: usize,
    focused_secs: f64,
    unfocused_secs: f64,
    previous_focused: Option<bool>,
    finished: bool,
}

#[derive(Serialize)]
struct HitchEvent {
    elapsed_secs: f64,
    frame_time_ms: f64,
    focused: bool,
    steady: bool,
}

#[derive(Serialize)]
struct ProbeReport {
    schema_version: u32,
    bevy_version: &'static str,
    variant: &'static str,
    duration_secs: f64,
    warmup_excluded_secs: f64,
    sample_count: usize,
    steady_sample_count: usize,
    focused_samples: usize,
    unfocused_samples: usize,
    focused_secs: f64,
    unfocused_secs: f64,
    focused_ratio: f64,
    minimum_focused_ratio: f64,
    valid_focus: bool,
    present_mode: &'static str,
    avg_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    max_ms: f64,
    frames_over_25ms: usize,
    frames_over_50ms: usize,
    frames_over_100ms: usize,
    steady_avg_ms: f64,
    steady_p95_ms: f64,
    steady_p99_ms: f64,
    steady_max_ms: f64,
    steady_frames_over_25ms: usize,
    steady_frames_over_50ms: usize,
    steady_frames_over_100ms: usize,
    hitch_events: Vec<HitchEvent>,
}

fn main() -> AppExit {
    let variant = match ProbeVariant::from_env() {
        Ok(variant) => variant,
        Err(error) => {
            eprintln!("{error}");
            return AppExit::from_code(2);
        }
    };
    let output_path = std::env::var_os("NAU_HITCH_PROBE_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/eval/frame_hitch_probe/bevy-0.18.1.json"));
    let duration_secs = match parse_duration_secs(
        std::env::var("NAU_HITCH_PROBE_DURATION_SECS")
            .ok()
            .as_deref(),
    ) {
        Ok(duration_secs) => duration_secs,
        Err(error) => {
            eprintln!("{error}");
            return AppExit::from_code(2);
        }
    };
    let default_plugins = DefaultPlugins.build().set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("NAU Bevy Frame Hitch Probe ({})", variant.label()),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    });
    let plugins = match variant {
        ProbeVariant::Default => default_plugins,
        ProbeVariant::NoAudio => default_plugins.disable::<bevy::audio::AudioPlugin>(),
        ProbeVariant::NoGilrs => default_plugins.disable::<bevy::gilrs::GilrsPlugin>(),
        ProbeVariant::NoAudioGilrs => default_plugins
            .disable::<bevy::audio::AudioPlugin>()
            .disable::<bevy::gilrs::GilrsPlugin>(),
        ProbeVariant::NoPipelinedRendering => default_plugins.disable::<PipelinedRenderingPlugin>(),
    };
    App::new()
        .insert_resource(ProbeState {
            variant,
            output_path,
            duration_secs,
            started_at: Instant::now(),
            previous_frame_at: None,
            frame_times_ms: Vec::new(),
            steady_frame_times_ms: Vec::new(),
            hitch_events: Vec::new(),
            focused_samples: 0,
            unfocused_samples: 0,
            focused_secs: 0.0,
            unfocused_secs: 0.0,
            previous_focused: None,
            finished: false,
        })
        .add_plugins(plugins)
        .add_systems(Startup, setup)
        .add_systems(Update, collect_frame_time)
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn collect_frame_time(
    mut state: ResMut<ProbeState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if state.finished {
        return;
    }

    let now = Instant::now();
    let elapsed_secs = now.duration_since(state.started_at).as_secs_f64();
    let focused = window.focused;
    if focused {
        state.focused_samples += 1;
    } else {
        state.unfocused_samples += 1;
    }

    if let Some(previous_frame_at) = state.previous_frame_at {
        let frame_time_secs = now.duration_since(previous_frame_at).as_secs_f64();
        let frame_time_ms = frame_time_secs * 1000.0;
        let interval_focused = state.previous_focused.unwrap_or(focused) && focused;
        if interval_focused {
            state.focused_secs += frame_time_secs;
        } else {
            state.unfocused_secs += frame_time_secs;
        }
        state.frame_times_ms.push(frame_time_ms);
        let steady = elapsed_secs >= WARMUP_EXCLUDED_SECS;
        if steady {
            state.steady_frame_times_ms.push(frame_time_ms);
        }
        if frame_time_ms > HITCH_THRESHOLD_MS {
            state.hitch_events.push(HitchEvent {
                elapsed_secs,
                frame_time_ms,
                focused: interval_focused,
                steady,
            });
        }
    }
    state.previous_frame_at = Some(now);
    state.previous_focused = Some(focused);

    if elapsed_secs < state.duration_secs {
        return;
    }

    state.finished = true;
    match write_report(&state, window.present_mode) {
        Err(error) => {
            eprintln!("frame hitch probe failed to write report: {error}");
            app_exit.write(AppExit::error());
        }
        Ok(false) => {
            eprintln!(
                "frame hitch probe capture is invalid: focused ratio was below {:.2}",
                MIN_FOCUSED_RATIO
            );
            app_exit.write(AppExit::error());
        }
        Ok(true) => {
            app_exit.write(AppExit::Success);
        }
    }
}

fn write_report(state: &ProbeState, present_mode: PresentMode) -> std::io::Result<bool> {
    if let Some(parent) = state
        .output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let focused_ratio = focused_ratio(state.focused_secs, state.unfocused_secs);
    let valid_focus = focused_ratio >= MIN_FOCUSED_RATIO;
    let report = ProbeReport {
        schema_version: 2,
        bevy_version: "0.18.1",
        variant: state.variant.label(),
        duration_secs: state.started_at.elapsed().as_secs_f64(),
        warmup_excluded_secs: WARMUP_EXCLUDED_SECS,
        sample_count: state.frame_times_ms.len(),
        steady_sample_count: state.steady_frame_times_ms.len(),
        focused_samples: state.focused_samples,
        unfocused_samples: state.unfocused_samples,
        focused_secs: state.focused_secs,
        unfocused_secs: state.unfocused_secs,
        focused_ratio,
        minimum_focused_ratio: MIN_FOCUSED_RATIO,
        valid_focus,
        present_mode: present_mode_label(present_mode),
        avg_ms: average(&state.frame_times_ms),
        p95_ms: percentile(&state.frame_times_ms, 0.95),
        p99_ms: percentile(&state.frame_times_ms, 0.99),
        max_ms: state.frame_times_ms.iter().copied().fold(0.0, f64::max),
        frames_over_25ms: count_over(&state.frame_times_ms, 25.0),
        frames_over_50ms: count_over(&state.frame_times_ms, 50.0),
        frames_over_100ms: count_over(&state.frame_times_ms, 100.0),
        steady_avg_ms: average(&state.steady_frame_times_ms),
        steady_p95_ms: percentile(&state.steady_frame_times_ms, 0.95),
        steady_p99_ms: percentile(&state.steady_frame_times_ms, 0.99),
        steady_max_ms: state
            .steady_frame_times_ms
            .iter()
            .copied()
            .fold(0.0, f64::max),
        steady_frames_over_25ms: count_over(&state.steady_frame_times_ms, 25.0),
        steady_frames_over_50ms: count_over(&state.steady_frame_times_ms, 50.0),
        steady_frames_over_100ms: count_over(&state.steady_frame_times_ms, 100.0),
        hitch_events: state
            .hitch_events
            .iter()
            .map(|event| HitchEvent {
                elapsed_secs: event.elapsed_secs,
                frame_time_ms: event.frame_time_ms,
                focused: event.focused,
                steady: event.steady,
            })
            .collect(),
    };
    fs::write(
        &state.output_path,
        serde_json::to_string_pretty(&report).expect("probe report should serialize"),
    )?;
    Ok(valid_focus)
}

fn parse_duration_secs(value: Option<&str>) -> Result<f64, String> {
    let Some(value) = value else {
        return Ok(DEFAULT_DURATION_SECS);
    };
    let duration_secs = value.parse::<f64>().map_err(|_| {
        format!("NAU_HITCH_PROBE_DURATION_SECS must be a positive number, got {value:?}")
    })?;
    if duration_secs.is_finite() && duration_secs > 0.0 {
        Ok(duration_secs)
    } else {
        Err(format!(
            "NAU_HITCH_PROBE_DURATION_SECS must be a positive number, got {value:?}"
        ))
    }
}

fn focused_ratio(focused_secs: f64, unfocused_secs: f64) -> f64 {
    let total_secs = focused_secs + unfocused_secs;
    if total_secs > 0.0 {
        focused_secs / total_secs
    } else {
        0.0
    }
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let index = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[index]
}

fn count_over(values: &[f64], threshold_ms: f64) -> usize {
    values.iter().filter(|value| **value > threshold_ms).count()
}

fn present_mode_label(present_mode: PresentMode) -> &'static str {
    match present_mode {
        PresentMode::AutoVsync => "auto_vsync",
        PresentMode::AutoNoVsync => "auto_no_vsync",
        PresentMode::Fifo => "fifo",
        PresentMode::FifoRelaxed => "fifo_relaxed",
        PresentMode::Immediate => "immediate",
        PresentMode::Mailbox => "mailbox",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_defaults_only_when_the_environment_value_is_absent() {
        assert_eq!(parse_duration_secs(None), Ok(DEFAULT_DURATION_SECS));
        assert_eq!(parse_duration_secs(Some("300")), Ok(300.0));
        assert!(parse_duration_secs(Some("five minutes")).is_err());
        assert!(parse_duration_secs(Some("0")).is_err());
        assert!(parse_duration_secs(Some("NaN")).is_err());
    }

    #[test]
    fn focus_ratio_is_weighted_by_elapsed_time() {
        assert_eq!(focused_ratio(95.0, 5.0), 0.95);
        assert_eq!(focused_ratio(0.0, 0.0), 0.0);
    }

    #[test]
    fn report_marks_insufficient_focus_as_invalid() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_frame_hitch_probe_{}_invalid_focus.json",
            std::process::id()
        ));
        let state = ProbeState {
            variant: ProbeVariant::NoAudioGilrs,
            output_path: output_path.clone(),
            duration_secs: 30.0,
            started_at: Instant::now(),
            previous_frame_at: None,
            frame_times_ms: vec![16.0],
            steady_frame_times_ms: vec![16.0],
            hitch_events: Vec::new(),
            focused_samples: 94,
            unfocused_samples: 6,
            focused_secs: 94.0,
            unfocused_secs: 6.0,
            previous_focused: Some(false),
            finished: true,
        };

        assert!(!write_report(&state, PresentMode::Fifo).expect("report should write"));
        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&output_path).expect("report should exist"))
                .expect("report should be valid json");
        assert_eq!(report["valid_focus"], false);
        assert_eq!(report["focused_ratio"], 0.94);
        fs::remove_file(output_path).expect("report should be removable");
    }

    #[test]
    fn frame_statistics_handle_empty_and_populated_samples() {
        assert_eq!(average(&[]), 0.0);
        assert_eq!(percentile(&[], 0.95), 0.0);
        assert_eq!(average(&[10.0, 20.0, 30.0]), 20.0);
        assert_eq!(percentile(&[30.0, 10.0, 20.0], 0.95), 30.0);
        assert_eq!(count_over(&[25.0, 25.1, 50.1], 25.0), 2);
    }
}
