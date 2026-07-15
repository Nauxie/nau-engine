use crate::authored_assets::VisualAssetDiagnostics;
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::island_visuals::{IslandLodVisualCounts, IslandStreamDiagnostics};
use crate::player_runtime::Player;
use crate::world_floor_runtime::WorldFloorDiagnostics;
use bevy::ecs::system::SystemParam;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::window::{Monitor, PresentMode, PrimaryMonitor, PrimaryWindow, Window};
use nau_engine::{
    asset_pipeline::VisualAssetPipelineMetrics,
    camera::CameraInput,
    movement::{FlightController, FlightInput, FlightMode},
    world::{SkyRoute, StreamingLodStats},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

const PROFILE_WRITE_INTERVAL_SECS: f64 = 1.0;
const PROFILE_WARMUP_EXCLUDED_SECS: f64 = 2.0;
const PROFILE_MIN_DURATION_SECS: f64 = 30.0;
const PROFILE_MIN_STEADY_SAMPLE_COUNT: usize = 1;
const PROFILE_MIN_HORIZONTAL_TRAVEL_M: f64 = 50.0;
const PROFILE_ARMING_HORIZONTAL_TRAVEL_M: f64 = 1.0;
const PROFILE_MAX_ARMING_WAIT_SECS: f64 = 30.0;
const PROFILE_MAX_AVG_FRAME_TIME_MS: f64 = 24.0;
const PROFILE_MAX_P95_FRAME_TIME_MS: f64 = 45.0;
const PROFILE_MAX_P99_FRAME_TIME_MS: f64 = 80.0;
const PROFILE_MAX_STEADY_50MS_HITCH_COUNT: usize = 3;
const PROFILE_MAX_STEADY_100MS_HITCH_COUNT: usize = 1;
const PROFILE_MIN_FOCUSED_WINDOW_RATIO: f64 = 0.95;
const PROFILE_HITCH_EVENT_THRESHOLD_MS: f64 = 25.0;
const PROFILE_MAX_HITCH_EVENTS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayProfileScript {
    Freeflight,
    GroundTraversal,
}

impl PlayProfileScript {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "freeflight" | "baseline_freeflight" => Ok(Self::Freeflight),
            "ground_traversal" | "terrain_walk" => Ok(Self::GroundTraversal),
            _ => Err(format!(
                "unknown play profile script: {value}. available scripts: freeflight, ground_traversal"
            )),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Freeflight => "freeflight",
            Self::GroundTraversal => "ground_traversal",
        }
    }

    fn start_position(self, route: &SkyRoute) -> Option<Vec3> {
        match self {
            Self::Freeflight => None,
            Self::GroundTraversal => {
                let mut position = Vec3::new(2_500.0, 0.0, -340.0);
                position.y = route.ground_at(position).floor_y;
                Some(position)
            }
        }
    }
}

#[derive(Resource, Debug)]
pub(crate) struct PlayProfileRun {
    output_path: PathBuf,
    target_duration_secs: Option<f64>,
    script: Option<PlayProfileScript>,
    armed: bool,
    arming_elapsed_secs: f64,
    arming_activity: PlayProfileActivity,
    frame_times_ms: Vec<f64>,
    frame: u64,
    elapsed_secs: f64,
    write_accumulator_secs: f64,
    grounded_samples: usize,
    airborne_samples: usize,
    focused_window_samples: usize,
    unfocused_window_samples: usize,
    focused_window_secs: f64,
    unfocused_window_secs: f64,
    previous_window_focused: Option<bool>,
    activity: PlayProfileActivity,
    latest: PlayProfileSnapshot,
    max: PlayProfileMaxima,
    hitch_events: Vec<PlayProfileHitchEvent>,
    io_error: Option<String>,
}

impl PlayProfileRun {
    pub(crate) fn new(
        output_path: PathBuf,
        target_duration_secs: Option<f64>,
        script: Option<PlayProfileScript>,
    ) -> std::io::Result<Self> {
        if let Some(parent) = output_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }

        Ok(Self {
            output_path,
            target_duration_secs,
            script,
            armed: false,
            arming_elapsed_secs: 0.0,
            arming_activity: PlayProfileActivity::default(),
            frame_times_ms: Vec::new(),
            frame: 0,
            elapsed_secs: 0.0,
            write_accumulator_secs: 0.0,
            grounded_samples: 0,
            airborne_samples: 0,
            focused_window_samples: 0,
            unfocused_window_samples: 0,
            focused_window_secs: 0.0,
            unfocused_window_secs: 0.0,
            previous_window_focused: None,
            activity: PlayProfileActivity::default(),
            latest: PlayProfileSnapshot::default(),
            max: PlayProfileMaxima::default(),
            hitch_events: Vec::new(),
            io_error: None,
        })
    }

    pub(crate) fn scripted_flight_input(&self) -> Option<FlightInput> {
        self.script
            .map(|script| scripted_profile_input(script, self.control_elapsed_secs()))
    }

    pub(crate) fn scripted_camera_input(&self, dt: f32) -> Option<CameraInput> {
        self.script
            .map(|script| scripted_profile_camera_input(script, self.control_elapsed_secs(), dt))
    }

    pub(crate) fn scripted_start_position(&self, route: &SkyRoute) -> Option<Vec3> {
        self.script.and_then(|script| script.start_position(route))
    }

    fn observe_frame(&mut self, frame_time_ms: f64, mut snapshot: PlayProfileSnapshot) -> bool {
        let delta_secs = (frame_time_ms / 1000.0).max(0.0);
        if !self.armed {
            self.arming_elapsed_secs += delta_secs;
            self.write_accumulator_secs += delta_secs;

            let window_focused = snapshot.window.focused != Some(false);
            if window_focused {
                self.arming_activity
                    .observe_position(snapshot.player_position);
            } else {
                self.arming_activity = PlayProfileActivity::default();
            }
            let active_play_detected = self
                .arming_activity
                .has_horizontal_travel(PROFILE_ARMING_HORIZONTAL_TRAVEL_M);
            if !active_play_detected || !window_focused {
                snapshot.elapsed_secs = 0.0;
                self.latest = snapshot;
                let should_write =
                    self.frame == 0 || self.write_accumulator_secs >= PROFILE_WRITE_INTERVAL_SECS;
                if should_write {
                    self.write_accumulator_secs = 0.0;
                }
                return should_write;
            }

            self.arm_for_active_play();
        }

        self.frame += 1;
        self.elapsed_secs += delta_secs;
        self.write_accumulator_secs += delta_secs;
        self.frame_times_ms.push(frame_time_ms);
        self.activity.observe_position(snapshot.player_position);
        match snapshot.player_mode {
            Some(FlightMode::Grounded) => self.grounded_samples += 1,
            Some(_) => self.airborne_samples += 1,
            None => {}
        }
        let interval_focused = match (self.previous_window_focused, snapshot.window.focused) {
            (Some(previous), Some(current)) => Some(previous && current),
            (None, current) => current,
            _ => None,
        };
        match snapshot.window.focused {
            Some(true) => self.focused_window_samples += 1,
            Some(false) => self.unfocused_window_samples += 1,
            None => {}
        }
        match interval_focused {
            Some(true) => self.focused_window_secs += delta_secs,
            Some(false) => self.unfocused_window_secs += delta_secs,
            None => {}
        }
        self.previous_window_focused = snapshot.window.focused;

        snapshot.frame = self.frame;
        snapshot.elapsed_secs = self.elapsed_secs;
        self.max.observe_frame(snapshot);
        self.observe_hitch_event(frame_time_ms, snapshot);
        self.latest = snapshot;

        let should_write =
            self.frame == 1 || self.write_accumulator_secs >= PROFILE_WRITE_INTERVAL_SECS;
        if should_write {
            self.write_accumulator_secs = 0.0;
        }
        should_write
    }

    fn observe_assets(&mut self, assets: RuntimeAssetSnapshot) {
        self.latest.runtime_assets = assets;
        self.max.observe_assets(assets);
    }

    fn should_write_summary_during_run(&self) -> bool {
        self.target_duration_secs.is_none() && self.script.is_none()
    }

    fn should_refresh_assets_during_run(&self) -> bool {
        self.latest.runtime_assets.mesh_count == 0 || self.should_write_summary_during_run()
    }

    fn write_summary(&self) -> std::io::Result<()> {
        let frame_stats = PlayProfileFrameStats::from_frame_times(&self.frame_times_ms);
        let steady_frame_times =
            frame_times_after_warmup(&self.frame_times_ms, PROFILE_WARMUP_EXCLUDED_SECS);
        let steady_frame_stats = PlayProfileFrameStats::from_frame_times(&steady_frame_times);
        let mut checks = play_profile_checks(self.elapsed_secs, self.activity, steady_frame_stats);
        checks.push(play_profile_window_focus_check(
            self.focused_window_secs,
            self.unfocused_window_secs,
        ));
        let passed = checks.iter().all(|check| check.passed);
        let checks_json = checks
            .iter()
            .copied()
            .map(PlayProfileCheck::to_json)
            .collect::<Vec<_>>();
        let mut retained_hitch_events = self.hitch_events.clone();
        retained_hitch_events.sort_by_key(|event| event.snapshot.frame);
        let hitch_events = retained_hitch_events
            .iter()
            .copied()
            .map(PlayProfileHitchEvent::to_json)
            .collect::<Vec<_>>();
        let report = json!({
            "schema_version": 2,
            "profile_kind": self.profile_kind(),
            "control_source": self.control_source(),
            "script": self.script.map(PlayProfileScript::name),
            "output_path": path_string(&self.output_path),
            "passed": passed,
            "checks": checks_json,
            "armed": self.armed,
            "arming": {
                "elapsed_secs": round3(self.arming_elapsed_secs),
                "max_wait_secs": round3(PROFILE_MAX_ARMING_WAIT_SECS),
                "required_horizontal_travel_m": round3(PROFILE_ARMING_HORIZONTAL_TRAVEL_M),
                "required_window_focused": true,
                "activity": self.arming_activity.to_json(),
            },
            "sample_count": frame_stats.sample_count,
            "duration_secs": round3(self.elapsed_secs),
            "target_duration_secs": self.target_duration_secs.map(round3),
            "warmup_excluded_secs": round3(PROFILE_WARMUP_EXCLUDED_SECS),
            "activity": self.activity.to_json(),
            "ground_contact": {
                "grounded_samples": self.grounded_samples,
                "airborne_samples": self.airborne_samples,
                "grounded_ratio": round3(
                    self.grounded_samples as f64
                        / (self.grounded_samples + self.airborne_samples).max(1) as f64
                ),
            },
            "window_focus": {
                "focused_samples": self.focused_window_samples,
                "unfocused_samples": self.unfocused_window_samples,
                "focused_secs": round3(self.focused_window_secs),
                "unfocused_secs": round3(self.unfocused_window_secs),
                "focused_ratio": round6(focused_window_ratio(
                    self.focused_window_secs,
                    self.unfocused_window_secs,
                )),
            },
            "frame_time": frame_stats.to_json(),
            "steady_frame_time": steady_frame_stats.to_json(),
            "hitch_event_threshold_ms": round3(PROFILE_HITCH_EVENT_THRESHOLD_MS),
            "max_hitch_events": PROFILE_MAX_HITCH_EVENTS,
            "hitch_events": hitch_events,
            "latest": self.latest.to_json(),
            "max": self.max.to_json(),
            "io_error": self.io_error,
        });
        fs::write(
            &self.output_path,
            serde_json::to_string_pretty(&report).expect("play profile json should serialize"),
        )
    }

    fn record_io_error(&mut self, error: std::io::Error) {
        let message = error.to_string();
        if self.io_error.is_none() {
            eprintln!("play profile write failed: {message}");
        }
        self.io_error = Some(message);
    }

    fn target_duration_reached(&self) -> bool {
        self.target_duration_secs.is_some_and(|target_secs| {
            if self.armed {
                self.elapsed_secs >= target_secs
            } else {
                self.arming_elapsed_secs >= PROFILE_MAX_ARMING_WAIT_SECS
            }
        })
    }

    fn arm_for_active_play(&mut self) {
        self.armed = true;
        self.frame = 0;
        self.elapsed_secs = 0.0;
        self.write_accumulator_secs = 0.0;
        self.grounded_samples = 0;
        self.airborne_samples = 0;
        self.focused_window_samples = 0;
        self.unfocused_window_samples = 0;
        self.focused_window_secs = 0.0;
        self.unfocused_window_secs = 0.0;
        self.previous_window_focused = None;
        self.frame_times_ms.clear();
        self.activity = PlayProfileActivity::default();
        self.max = PlayProfileMaxima::default();
        self.hitch_events.clear();
    }

    fn observe_hitch_event(&mut self, frame_time_ms: f64, snapshot: PlayProfileSnapshot) {
        if frame_time_ms <= PROFILE_HITCH_EVENT_THRESHOLD_MS {
            return;
        }

        let hitch_event = PlayProfileHitchEvent {
            frame_time_ms,
            snapshot,
        };
        if self.hitch_events.len() < PROFILE_MAX_HITCH_EVENTS {
            self.hitch_events.push(hitch_event);
            return;
        }

        let Some((least_severe_index, least_severe_event)) = self
            .hitch_events
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| left.frame_time_ms.total_cmp(&right.frame_time_ms))
        else {
            return;
        };
        if frame_time_ms > least_severe_event.frame_time_ms {
            self.hitch_events[least_severe_index] = hitch_event;
        }
    }

    fn control_elapsed_secs(&self) -> f64 {
        if self.armed {
            self.elapsed_secs
        } else {
            self.arming_elapsed_secs
        }
    }

    fn profile_kind(&self) -> &'static str {
        if self.script.is_some() {
            "scripted_play_foreground"
        } else {
            "manual_play_foreground"
        }
    }

    fn control_source(&self) -> &'static str {
        if self.script.is_some() {
            "scripted"
        } else {
            "manual"
        }
    }
}

fn scripted_profile_input(script: PlayProfileScript, t: f64) -> FlightInput {
    match script {
        PlayProfileScript::Freeflight => FlightInput {
            launch: t <= 0.10,
            forward: t >= 0.05,
            right: (3.0..=5.5).contains(&t)
                || (16.0..=18.5).contains(&t)
                || (34.0..=37.0).contains(&t),
            left: (8.0..=10.0).contains(&t)
                || (24.0..=27.0).contains(&t)
                || (41.0..=43.0).contains(&t),
            backward: (20.0..=22.0).contains(&t),
            glide: t >= 0.45,
            dive: (12.0..=13.6).contains(&t) || (30.0..=31.4).contains(&t),
        },
        PlayProfileScript::GroundTraversal => FlightInput {
            forward: true,
            right: (8.0..=12.0).contains(&t) || (28.0..=32.0).contains(&t),
            left: (18.0..=22.0).contains(&t) || (38.0..=42.0).contains(&t),
            ..default()
        },
    }
}

fn scripted_profile_camera_input(script: PlayProfileScript, t: f64, dt: f32) -> CameraInput {
    let mouse_delta_per_60hz_frame = match script {
        PlayProfileScript::Freeflight if (6.0..=6.8).contains(&t) => Vec2::new(1.1, 0.0),
        PlayProfileScript::Freeflight if (14.0..=14.8).contains(&t) => Vec2::new(-1.0, 0.0),
        PlayProfileScript::Freeflight if (28.0..=29.0).contains(&t) => Vec2::new(0.0, -0.8),
        PlayProfileScript::Freeflight if (38.0..=39.0).contains(&t) => Vec2::new(0.0, 0.7),
        PlayProfileScript::GroundTraversal => Vec2::ZERO,
        _ => Vec2::ZERO,
    };
    CameraInput {
        mouse_delta: mouse_delta_per_60hz_frame * (dt.max(0.0) * 60.0),
    }
}

pub(crate) fn collect_play_profile_sample(
    time: Res<Time>,
    mut profile: ResMut<PlayProfileRun>,
    scene: PlayProfileScene,
    mut app_exit: MessageWriter<AppExit>,
) {
    let player_state = scene
        .player
        .single()
        .ok()
        .map(|(transform, controller)| (transform.translation, controller.mode));
    let player_position = player_state.map(|(position, _mode)| position);
    let streaming_lod = player_position
        .map(|position| scene.route.streaming_lod_stats(position))
        .unwrap_or_default();
    let frame_time_ms = time.delta_secs_f64() * 1000.0;
    let runtime_assets = profile.latest.runtime_assets;
    let primary_window = scene.primary_window.single().ok();
    let mut monitor_count = 0;
    let mut min_monitor_refresh_rate_hz: Option<f64> = None;
    let mut max_monitor_refresh_rate_hz: Option<f64> = None;
    for monitor in &scene.monitors {
        monitor_count += 1;
        let Some(refresh_rate_millihertz) = monitor.refresh_rate_millihertz else {
            continue;
        };
        let refresh_rate_hz = refresh_rate_millihertz as f64 / 1000.0;
        min_monitor_refresh_rate_hz = Some(
            min_monitor_refresh_rate_hz.map_or(refresh_rate_hz, |min| min.min(refresh_rate_hz)),
        );
        max_monitor_refresh_rate_hz = Some(
            max_monitor_refresh_rate_hz.map_or(refresh_rate_hz, |max| max.max(refresh_rate_hz)),
        );
    }
    let snapshot = PlayProfileSnapshot {
        frame: 0,
        elapsed_secs: 0.0,
        player_position,
        player_mode: player_state.map(|(_position, mode)| mode),
        window: PlayProfileWindowSnapshot {
            focused: primary_window.map(|window| window.focused),
            present_mode: primary_window.map(|window| window.present_mode),
            monitor_count,
            min_monitor_refresh_rate_hz,
            max_monitor_refresh_rate_hz,
            primary_monitor_refresh_rate_hz: scene
                .primary_monitor
                .single()
                .ok()
                .and_then(|monitor| monitor.refresh_rate_millihertz)
                .map(|refresh_rate_millihertz| refresh_rate_millihertz as f64 / 1000.0),
        },
        entity_count: scene.all_entities.iter().count(),
        streaming_lod,
        lod_visuals: scene.stream_diagnostics.counts,
        stream_diagnostics: *scene.stream_diagnostics,
        world_floor: *scene.world_floor,
        visual_asset_metrics: scene.asset_diagnostics.metrics,
        visible_authored_world_fixture_count: scene.asset_diagnostics.visible_world_fixture_count,
        content_diagnostics: *scene.content_diagnostics,
        runtime_assets,
    };

    if profile.observe_frame(frame_time_ms, snapshot) {
        if profile.should_refresh_assets_during_run() {
            let sampled_at_frame = profile.frame;
            profile.observe_assets(RuntimeAssetSnapshot::from_assets(
                &scene.meshes,
                &scene.materials,
                sampled_at_frame,
            ));
        }
        if profile.should_write_summary_during_run()
            && let Err(error) = profile.write_summary()
        {
            profile.record_io_error(error);
        }
    }

    if profile.target_duration_reached() {
        let sampled_at_frame = profile.frame;
        profile.observe_assets(RuntimeAssetSnapshot::from_assets(
            &scene.meshes,
            &scene.materials,
            sampled_at_frame,
        ));
        if let Err(error) = profile.write_summary() {
            profile.record_io_error(error);
            app_exit.write(AppExit::error());
        } else {
            app_exit.write(AppExit::Success);
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct PlayProfileScene<'w, 's> {
    route: Res<'w, SkyRoute>,
    stream_diagnostics: Res<'w, IslandStreamDiagnostics>,
    world_floor: Res<'w, WorldFloorDiagnostics>,
    asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    content_diagnostics: Res<'w, IslandContentDiagnostics>,
    meshes: Res<'w, Assets<Mesh>>,
    materials: Res<'w, Assets<StandardMaterial>>,
    player: Query<'w, 's, (&'static Transform, &'static FlightController), With<Player>>,
    primary_window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    monitors: Query<'w, 's, &'static Monitor>,
    primary_monitor: Query<'w, 's, &'static Monitor, With<PrimaryMonitor>>,
    all_entities: Query<'w, 's, Entity>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PlayProfileWindowSnapshot {
    focused: Option<bool>,
    present_mode: Option<PresentMode>,
    monitor_count: usize,
    min_monitor_refresh_rate_hz: Option<f64>,
    max_monitor_refresh_rate_hz: Option<f64>,
    primary_monitor_refresh_rate_hz: Option<f64>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PlayProfileSnapshot {
    frame: u64,
    elapsed_secs: f64,
    player_position: Option<Vec3>,
    player_mode: Option<FlightMode>,
    window: PlayProfileWindowSnapshot,
    entity_count: usize,
    streaming_lod: StreamingLodStats,
    lod_visuals: IslandLodVisualCounts,
    stream_diagnostics: IslandStreamDiagnostics,
    world_floor: WorldFloorDiagnostics,
    visual_asset_metrics: VisualAssetPipelineMetrics,
    visible_authored_world_fixture_count: usize,
    content_diagnostics: IslandContentDiagnostics,
    runtime_assets: RuntimeAssetSnapshot,
}

impl PlayProfileSnapshot {
    fn to_json(self) -> Value {
        json!({
            "frame": self.frame,
            "elapsed_secs": round3(self.elapsed_secs),
            "player_position": vec3_json(self.player_position),
            "player_mode": self.player_mode.map(FlightMode::label),
            "window": {
                "focused": self.window.focused,
                "present_mode": self.window.present_mode.map(present_mode_label),
                "monitor_count": self.window.monitor_count,
                "min_monitor_refresh_rate_hz": self
                    .window
                    .min_monitor_refresh_rate_hz
                    .map(round3),
                "max_monitor_refresh_rate_hz": self
                    .window
                    .max_monitor_refresh_rate_hz
                    .map(round3),
                "primary_monitor_refresh_rate_hz": self
                    .window
                    .primary_monitor_refresh_rate_hz
                    .map(round3),
            },
            "entity_count": self.entity_count,
            "streaming_lod": streaming_lod_json(self.streaming_lod),
            "island_visuals": island_lod_visuals_json(self.lod_visuals),
            "streaming": stream_diagnostics_json(self.stream_diagnostics),
            "world_floor": world_floor_diagnostics_json(self.world_floor),
            "runtime_assets": runtime_assets_json(self.runtime_assets, self.frame),
            "visual_asset_pipeline": visual_asset_metrics_json(
                self.visual_asset_metrics,
                self.visible_authored_world_fixture_count,
            ),
            "content_diagnostics": content_diagnostics_json(self.content_diagnostics),
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PlayProfileActivity {
    start_position: Option<Vec3>,
    previous_position: Option<Vec3>,
    total_travel_m: f64,
    horizontal_travel_m: f64,
    max_horizontal_displacement_m: f64,
}

impl PlayProfileActivity {
    fn observe_position(&mut self, position: Option<Vec3>) {
        let Some(position) = position else {
            return;
        };

        if self.start_position.is_none() {
            self.start_position = Some(position);
        }

        if let Some(previous_position) = self.previous_position {
            self.total_travel_m += position.distance(previous_position) as f64;
            self.horizontal_travel_m += Vec2::new(
                position.x - previous_position.x,
                position.z - previous_position.z,
            )
            .length() as f64;
        }

        if let Some(start_position) = self.start_position {
            let horizontal_displacement =
                Vec2::new(position.x - start_position.x, position.z - start_position.z).length()
                    as f64;
            self.max_horizontal_displacement_m = self
                .max_horizontal_displacement_m
                .max(horizontal_displacement);
        }

        self.previous_position = Some(position);
    }

    fn to_json(self) -> Value {
        json!({
            "total_travel_m": round3(self.total_travel_m),
            "horizontal_travel_m": round3(self.horizontal_travel_m),
            "max_horizontal_displacement_m": round3(self.max_horizontal_displacement_m),
        })
    }

    fn has_horizontal_travel(self, threshold_m: f64) -> bool {
        self.horizontal_travel_m >= threshold_m || self.max_horizontal_displacement_m >= threshold_m
    }
}

#[derive(Clone, Copy, Debug)]
struct PlayProfileHitchEvent {
    frame_time_ms: f64,
    snapshot: PlayProfileSnapshot,
}

impl PlayProfileHitchEvent {
    fn to_json(self) -> Value {
        json!({
            "frame": self.snapshot.frame,
            "elapsed_secs": round3(self.snapshot.elapsed_secs),
            "frame_time_ms": round3(self.frame_time_ms),
            "severity": self.severity(),
            "steady": self.snapshot.elapsed_secs > PROFILE_WARMUP_EXCLUDED_SECS,
            "snapshot": self.snapshot.to_json(),
        })
    }

    fn severity(self) -> &'static str {
        if self.frame_time_ms > 100.0 {
            "over_100ms"
        } else if self.frame_time_ms > 50.0 {
            "over_50ms"
        } else if self.frame_time_ms > 33.34 {
            "over_33_34ms"
        } else {
            "over_25ms"
        }
    }
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

#[derive(Clone, Copy, Debug, Default)]
struct PlayProfileMaxima {
    entity_count: usize,
    active_chunk_count: usize,
    active_island_count: usize,
    near_lod_islands: usize,
    mid_lod_islands: usize,
    far_lod_islands: usize,
    visible_terrain_count: usize,
    visible_detail_count: usize,
    visible_impostor_count: usize,
    visible_beacon_count: usize,
    resident_visual_count: usize,
    catalog_visual_count: usize,
    hidden_visual_count: usize,
    visibility_changes_this_frame: usize,
    max_visibility_changes_per_frame: usize,
    spawned_visuals_this_frame: usize,
    despawned_visuals_this_frame: usize,
    max_spawned_visuals_per_frame: usize,
    max_despawned_visuals_per_frame: usize,
    world_floor_visible_tile_count: usize,
    world_floor_resident_tile_count: usize,
    world_floor_initial_spawned_tile_count: usize,
    world_floor_spawned_tiles_per_frame: usize,
    world_floor_despawned_tiles_per_frame: usize,
    world_floor_mesh_vertex_count: usize,
    world_floor_mesh_triangle_count: usize,
    world_floor_material_count: usize,
    world_floor_biome_count: usize,
    world_floor_terrain_feature_count: usize,
    world_floor_color_band_count: usize,
    world_floor_river_vertex_count: usize,
    world_floor_relief_range_m: f32,
    mesh_count: usize,
    material_count: usize,
    loaded_mesh_vertices: usize,
    loaded_mesh_triangles: usize,
    visual_asset_slot_count: usize,
    loaded_visual_asset_scene_count: usize,
    ready_visual_asset_scene_count: usize,
    failed_visual_asset_scene_count: usize,
    visible_authored_world_fixture_count: usize,
}

impl PlayProfileMaxima {
    fn observe_frame(&mut self, snapshot: PlayProfileSnapshot) {
        self.entity_count = self.entity_count.max(snapshot.entity_count);
        self.active_chunk_count = self
            .active_chunk_count
            .max(snapshot.streaming_lod.active_chunk_count);
        self.active_island_count = self
            .active_island_count
            .max(snapshot.streaming_lod.active_island_count);
        self.near_lod_islands = self
            .near_lod_islands
            .max(snapshot.streaming_lod.near_lod_islands);
        self.mid_lod_islands = self
            .mid_lod_islands
            .max(snapshot.streaming_lod.mid_lod_islands);
        self.far_lod_islands = self
            .far_lod_islands
            .max(snapshot.streaming_lod.far_lod_islands);
        self.visible_terrain_count = self
            .visible_terrain_count
            .max(snapshot.lod_visuals.visible_terrain_count);
        self.visible_detail_count = self
            .visible_detail_count
            .max(snapshot.lod_visuals.visible_detail_count);
        self.visible_impostor_count = self
            .visible_impostor_count
            .max(snapshot.lod_visuals.visible_impostor_count);
        self.visible_beacon_count = self
            .visible_beacon_count
            .max(snapshot.lod_visuals.visible_beacon_count);
        self.resident_visual_count = self
            .resident_visual_count
            .max(snapshot.lod_visuals.resident_count());
        self.catalog_visual_count = self
            .catalog_visual_count
            .max(snapshot.lod_visuals.catalog_count());
        self.hidden_visual_count = self
            .hidden_visual_count
            .max(snapshot.lod_visuals.hidden_count());
        self.visibility_changes_this_frame = self
            .visibility_changes_this_frame
            .max(snapshot.stream_diagnostics.visibility_changes_this_frame);
        self.max_visibility_changes_per_frame = self
            .max_visibility_changes_per_frame
            .max(snapshot.stream_diagnostics.max_visibility_changes_per_frame);
        self.spawned_visuals_this_frame = self
            .spawned_visuals_this_frame
            .max(snapshot.stream_diagnostics.spawned_visuals_this_frame);
        self.despawned_visuals_this_frame = self
            .despawned_visuals_this_frame
            .max(snapshot.stream_diagnostics.despawned_visuals_this_frame);
        self.max_spawned_visuals_per_frame = self
            .max_spawned_visuals_per_frame
            .max(snapshot.stream_diagnostics.max_spawned_visuals_per_frame);
        self.max_despawned_visuals_per_frame = self
            .max_despawned_visuals_per_frame
            .max(snapshot.stream_diagnostics.max_despawned_visuals_per_frame);
        self.world_floor_visible_tile_count = self
            .world_floor_visible_tile_count
            .max(snapshot.world_floor.visible_tile_count);
        self.world_floor_resident_tile_count = self
            .world_floor_resident_tile_count
            .max(snapshot.world_floor.resident_tile_count);
        self.world_floor_initial_spawned_tile_count = self
            .world_floor_initial_spawned_tile_count
            .max(snapshot.world_floor.initial_spawned_tile_count);
        self.world_floor_spawned_tiles_per_frame = self
            .world_floor_spawned_tiles_per_frame
            .max(snapshot.world_floor.max_spawned_tiles_per_frame);
        self.world_floor_despawned_tiles_per_frame = self
            .world_floor_despawned_tiles_per_frame
            .max(snapshot.world_floor.max_despawned_tiles_per_frame);
        self.world_floor_mesh_vertex_count = self
            .world_floor_mesh_vertex_count
            .max(snapshot.world_floor.mesh_vertex_count);
        self.world_floor_mesh_triangle_count = self
            .world_floor_mesh_triangle_count
            .max(snapshot.world_floor.mesh_triangle_count);
        self.world_floor_material_count = self
            .world_floor_material_count
            .max(snapshot.world_floor.material_count);
        self.world_floor_biome_count = self
            .world_floor_biome_count
            .max(snapshot.world_floor.biome_count);
        self.world_floor_terrain_feature_count = self
            .world_floor_terrain_feature_count
            .max(snapshot.world_floor.terrain_feature_count);
        self.world_floor_color_band_count = self
            .world_floor_color_band_count
            .max(snapshot.world_floor.color_band_count);
        self.world_floor_river_vertex_count = self
            .world_floor_river_vertex_count
            .max(snapshot.world_floor.river_vertex_count);
        self.world_floor_relief_range_m = self
            .world_floor_relief_range_m
            .max(snapshot.world_floor.relief_range_m);
        self.visual_asset_slot_count = self
            .visual_asset_slot_count
            .max(snapshot.visual_asset_metrics.slot_count);
        self.loaded_visual_asset_scene_count = self
            .loaded_visual_asset_scene_count
            .max(snapshot.visual_asset_metrics.loaded_scene_count);
        self.ready_visual_asset_scene_count = self
            .ready_visual_asset_scene_count
            .max(snapshot.visual_asset_metrics.ready_scene_count);
        self.failed_visual_asset_scene_count = self
            .failed_visual_asset_scene_count
            .max(snapshot.visual_asset_metrics.failed_scene_count);
        self.visible_authored_world_fixture_count = self
            .visible_authored_world_fixture_count
            .max(snapshot.visible_authored_world_fixture_count);
    }

    fn observe_assets(&mut self, assets: RuntimeAssetSnapshot) {
        self.mesh_count = self.mesh_count.max(assets.mesh_count);
        self.material_count = self.material_count.max(assets.material_count);
        self.loaded_mesh_vertices = self.loaded_mesh_vertices.max(assets.loaded_mesh_vertices);
        self.loaded_mesh_triangles = self.loaded_mesh_triangles.max(assets.loaded_mesh_triangles);
    }

    fn to_json(self) -> Value {
        json!({
            "entity_count": self.entity_count,
            "active_chunk_count": self.active_chunk_count,
            "active_island_count": self.active_island_count,
            "near_lod_islands": self.near_lod_islands,
            "mid_lod_islands": self.mid_lod_islands,
            "far_lod_islands": self.far_lod_islands,
            "visible_island_terrain_count": self.visible_terrain_count,
            "visible_island_detail_count": self.visible_detail_count,
            "visible_island_impostor_count": self.visible_impostor_count,
            "visible_route_beacon_count": self.visible_beacon_count,
            "resident_island_visual_count": self.resident_visual_count,
            "catalog_island_visual_count": self.catalog_visual_count,
            "hidden_island_visual_count": self.hidden_visual_count,
            "stream_visibility_changes_this_frame": self.visibility_changes_this_frame,
            "stream_visibility_changes_per_frame": self.max_visibility_changes_per_frame,
            "stream_spawned_visuals_this_frame": self.spawned_visuals_this_frame,
            "stream_despawned_visuals_this_frame": self.despawned_visuals_this_frame,
            "stream_spawned_visuals_per_frame": self.max_spawned_visuals_per_frame,
            "stream_despawned_visuals_per_frame": self.max_despawned_visuals_per_frame,
            "world_floor_visible_tile_count": self.world_floor_visible_tile_count,
            "world_floor_resident_tile_count": self.world_floor_resident_tile_count,
            "world_floor_initial_spawned_tile_count": self.world_floor_initial_spawned_tile_count,
            "world_floor_spawned_tiles_per_frame": self.world_floor_spawned_tiles_per_frame,
            "world_floor_despawned_tiles_per_frame": self.world_floor_despawned_tiles_per_frame,
            "world_floor_mesh_vertex_count": self.world_floor_mesh_vertex_count,
            "world_floor_mesh_triangle_count": self.world_floor_mesh_triangle_count,
            "world_floor_material_count": self.world_floor_material_count,
            "world_floor_biome_count": self.world_floor_biome_count,
            "world_floor_terrain_feature_count": self.world_floor_terrain_feature_count,
            "world_floor_color_band_count": self.world_floor_color_band_count,
            "world_floor_river_vertex_count": self.world_floor_river_vertex_count,
            "world_floor_relief_range_m": round3(self.world_floor_relief_range_m as f64),
            "mesh_count": self.mesh_count,
            "material_count": self.material_count,
            "loaded_mesh_vertices": self.loaded_mesh_vertices,
            "loaded_mesh_triangles": self.loaded_mesh_triangles,
            "visual_asset_slot_count": self.visual_asset_slot_count,
            "loaded_visual_asset_scene_count": self.loaded_visual_asset_scene_count,
            "ready_visual_asset_scene_count": self.ready_visual_asset_scene_count,
            "failed_visual_asset_scene_count": self.failed_visual_asset_scene_count,
            "visible_authored_world_fixture_count": self.visible_authored_world_fixture_count,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct RuntimeAssetSnapshot {
    mesh_count: usize,
    material_count: usize,
    loaded_mesh_vertices: usize,
    loaded_mesh_triangles: usize,
    sampled_at_frame: u64,
}

impl RuntimeAssetSnapshot {
    fn from_assets(
        meshes: &Assets<Mesh>,
        materials: &Assets<StandardMaterial>,
        sampled_at_frame: u64,
    ) -> Self {
        let mut loaded_mesh_vertices = 0;
        let mut loaded_mesh_triangles = 0;
        for (_, mesh) in meshes.iter() {
            loaded_mesh_vertices += mesh.count_vertices();
            loaded_mesh_triangles += mesh_triangle_count(mesh);
        }

        Self {
            mesh_count: meshes.len(),
            material_count: materials.len(),
            loaded_mesh_vertices,
            loaded_mesh_triangles,
            sampled_at_frame,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PlayProfileFrameStats {
    sample_count: usize,
    avg_frame_time_ms: f64,
    p95_frame_time_ms: f64,
    p99_frame_time_ms: f64,
    max_frame_time_ms: f64,
    frames_over_16ms: usize,
    frames_over_33ms: usize,
    frames_over_50ms: usize,
    frames_over_100ms: usize,
}

impl PlayProfileFrameStats {
    fn from_frame_times(frame_times_ms: &[f64]) -> Self {
        if frame_times_ms.is_empty() {
            return Self::default();
        }

        let sum = frame_times_ms.iter().sum::<f64>();
        Self {
            sample_count: frame_times_ms.len(),
            avg_frame_time_ms: sum / frame_times_ms.len() as f64,
            p95_frame_time_ms: percentile(frame_times_ms, 0.95),
            p99_frame_time_ms: percentile(frame_times_ms, 0.99),
            max_frame_time_ms: percentile(frame_times_ms, 1.0),
            frames_over_16ms: frame_times_ms
                .iter()
                .filter(|frame_time_ms| **frame_time_ms > 16.67)
                .count(),
            frames_over_33ms: frame_times_ms
                .iter()
                .filter(|frame_time_ms| **frame_time_ms > 33.34)
                .count(),
            frames_over_50ms: frame_times_ms
                .iter()
                .filter(|frame_time_ms| **frame_time_ms > 50.0)
                .count(),
            frames_over_100ms: frame_times_ms
                .iter()
                .filter(|frame_time_ms| **frame_time_ms > 100.0)
                .count(),
        }
    }

    fn to_json(self) -> Value {
        json!({
            "sample_count": self.sample_count,
            "avg_ms": round3(self.avg_frame_time_ms),
            "p95_ms": round3(self.p95_frame_time_ms),
            "p99_ms": round3(self.p99_frame_time_ms),
            "max_ms": round3(self.max_frame_time_ms),
            "avg_fps": fps_from_frame_time_ms(self.avg_frame_time_ms),
            "p95_frame_time_fps": fps_from_frame_time_ms(self.p95_frame_time_ms),
            "p99_frame_time_fps": fps_from_frame_time_ms(self.p99_frame_time_ms),
            "frames_over_16_67ms": self.frames_over_16ms,
            "frames_over_33_34ms": self.frames_over_33ms,
            "frames_over_50ms": self.frames_over_50ms,
            "frames_over_100ms": self.frames_over_100ms,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct PlayProfileCheck {
    name: &'static str,
    passed: bool,
    value: f64,
    comparator: &'static str,
    threshold: f64,
    unit: &'static str,
}

impl PlayProfileCheck {
    fn to_json(self) -> Value {
        let round_value = if self.unit == "ratio" { round6 } else { round3 };
        json!({
            "name": self.name,
            "passed": self.passed,
            "value": round_value(self.value),
            "comparator": self.comparator,
            "threshold": round_value(self.threshold),
            "unit": self.unit,
        })
    }
}

fn play_profile_checks(
    duration_secs: f64,
    activity: PlayProfileActivity,
    steady_frame_stats: PlayProfileFrameStats,
) -> Vec<PlayProfileCheck> {
    vec![
        min_check(
            "play_profile_duration",
            duration_secs,
            PROFILE_MIN_DURATION_SECS,
            "secs",
        ),
        min_check(
            "play_profile_horizontal_travel",
            activity.horizontal_travel_m,
            PROFILE_MIN_HORIZONTAL_TRAVEL_M,
            "m",
        ),
        min_check(
            "play_profile_steady_sample_count",
            steady_frame_stats.sample_count as f64,
            PROFILE_MIN_STEADY_SAMPLE_COUNT as f64,
            "frames",
        ),
        max_check(
            "play_profile_steady_avg_frame_time_budget",
            steady_frame_stats.avg_frame_time_ms,
            PROFILE_MAX_AVG_FRAME_TIME_MS,
            "ms",
        ),
        max_check(
            "play_profile_steady_p95_frame_time_budget",
            steady_frame_stats.p95_frame_time_ms,
            PROFILE_MAX_P95_FRAME_TIME_MS,
            "ms",
        ),
        max_check(
            "play_profile_steady_p99_frame_time_budget",
            steady_frame_stats.p99_frame_time_ms,
            PROFILE_MAX_P99_FRAME_TIME_MS,
            "ms",
        ),
        max_check(
            "play_profile_steady_50ms_hitch_count",
            steady_frame_stats.frames_over_50ms as f64,
            PROFILE_MAX_STEADY_50MS_HITCH_COUNT as f64,
            "frames",
        ),
        max_check(
            "play_profile_steady_100ms_hitch_count",
            steady_frame_stats.frames_over_100ms as f64,
            PROFILE_MAX_STEADY_100MS_HITCH_COUNT as f64,
            "frames",
        ),
    ]
}

fn play_profile_window_focus_check(focused_secs: f64, unfocused_secs: f64) -> PlayProfileCheck {
    min_check(
        "play_profile_window_focused_ratio",
        focused_window_ratio(focused_secs, unfocused_secs),
        PROFILE_MIN_FOCUSED_WINDOW_RATIO,
        "ratio",
    )
}

fn focused_window_ratio(focused_secs: f64, unfocused_secs: f64) -> f64 {
    let total_secs = focused_secs + unfocused_secs;
    if total_secs > 0.0 {
        focused_secs / total_secs
    } else {
        0.0
    }
}

fn min_check(
    name: &'static str,
    value: f64,
    threshold: f64,
    unit: &'static str,
) -> PlayProfileCheck {
    PlayProfileCheck {
        name,
        passed: value >= threshold,
        value,
        comparator: ">=",
        threshold,
        unit,
    }
}

fn max_check(
    name: &'static str,
    value: f64,
    threshold: f64,
    unit: &'static str,
) -> PlayProfileCheck {
    PlayProfileCheck {
        name,
        passed: value <= threshold,
        value,
        comparator: "<=",
        threshold,
        unit,
    }
}

fn frame_times_after_warmup(frame_times_ms: &[f64], warmup_secs: f64) -> Vec<f64> {
    if warmup_secs <= 0.0 {
        return frame_times_ms.to_vec();
    }

    let mut elapsed_secs = 0.0;
    let mut first_steady_index = frame_times_ms.len();
    for (index, frame_time_ms) in frame_times_ms.iter().enumerate() {
        elapsed_secs += frame_time_ms / 1000.0;
        if elapsed_secs >= warmup_secs {
            first_steady_index = index + 1;
            break;
        }
    }

    frame_times_ms[first_steady_index..].to_vec()
}

fn streaming_lod_json(stats: StreamingLodStats) -> Value {
    json!({
        "player_chunk": {
            "x": stats.player_chunk.x,
            "z": stats.player_chunk.z,
        },
        "active_chunk_count": stats.active_chunk_count,
        "active_island_count": stats.active_island_count,
        "near_lod_islands": stats.near_lod_islands,
        "mid_lod_islands": stats.mid_lod_islands,
        "far_lod_islands": stats.far_lod_islands,
    })
}

fn island_lod_visuals_json(counts: IslandLodVisualCounts) -> Value {
    json!({
        "visible_terrain_count": counts.visible_terrain_count,
        "hidden_terrain_count": counts.hidden_terrain_count,
        "visible_detail_count": counts.visible_detail_count,
        "hidden_detail_count": counts.hidden_detail_count,
        "visible_beacon_count": counts.visible_beacon_count,
        "visible_impostor_count": counts.visible_impostor_count,
        "hidden_impostor_count": counts.hidden_impostor_count,
        "resident_count": counts.resident_count(),
        "catalog_count": counts.catalog_count(),
        "hidden_count": counts.hidden_count(),
        "resident_fraction": round3(counts.resident_fraction() as f64),
    })
}

fn stream_diagnostics_json(diagnostics: IslandStreamDiagnostics) -> Value {
    json!({
        "visibility_changes_this_frame": diagnostics.visibility_changes_this_frame,
        "max_visibility_changes_per_frame": diagnostics.max_visibility_changes_per_frame,
        "total_visibility_changes": diagnostics.total_visibility_changes,
        "spawned_visuals_this_frame": diagnostics.spawned_visuals_this_frame,
        "despawned_visuals_this_frame": diagnostics.despawned_visuals_this_frame,
        "max_spawned_visuals_per_frame": diagnostics.max_spawned_visuals_per_frame,
        "max_despawned_visuals_per_frame": diagnostics.max_despawned_visuals_per_frame,
        "total_spawned_visuals": diagnostics.total_spawned_visuals,
        "total_despawned_visuals": diagnostics.total_despawned_visuals,
    })
}

fn world_floor_diagnostics_json(diagnostics: WorldFloorDiagnostics) -> Value {
    json!({
        "visible_tile_count": diagnostics.visible_tile_count,
        "max_visible_tile_count": diagnostics.max_visible_tile_count,
        "resident_tile_count": diagnostics.resident_tile_count,
        "max_resident_tile_count": diagnostics.max_resident_tile_count,
        "initial_spawned_tile_count": diagnostics.initial_spawned_tile_count,
        "spawned_tiles_this_frame": diagnostics.spawned_tiles_this_frame,
        "despawned_tiles_this_frame": diagnostics.despawned_tiles_this_frame,
        "max_spawned_tiles_per_frame": diagnostics.max_spawned_tiles_per_frame,
        "max_despawned_tiles_per_frame": diagnostics.max_despawned_tiles_per_frame,
        "total_spawned_tiles": diagnostics.total_spawned_tiles,
        "total_despawned_tiles": diagnostics.total_despawned_tiles,
        "mesh_vertex_count": diagnostics.mesh_vertex_count,
        "mesh_triangle_count": diagnostics.mesh_triangle_count,
        "material_count": diagnostics.material_count,
        "biome_count": diagnostics.biome_count,
        "terrain_feature_count": diagnostics.terrain_feature_count,
        "color_band_count": diagnostics.color_band_count,
        "river_vertex_count": diagnostics.river_vertex_count,
        "min_height_y": round3(diagnostics.min_height_y as f64),
        "max_height_y": round3(diagnostics.max_height_y as f64),
        "relief_range_m": round3(diagnostics.relief_range_m as f64),
        "active_radius_tiles": diagnostics.active_radius_tiles,
        "tile_size_m": round3(diagnostics.tile_size_m as f64),
    })
}

fn runtime_assets_json(assets: RuntimeAssetSnapshot, current_frame: u64) -> Value {
    json!({
        "mesh_count": assets.mesh_count,
        "material_count": assets.material_count,
        "loaded_mesh_vertices": assets.loaded_mesh_vertices,
        "loaded_mesh_triangles": assets.loaded_mesh_triangles,
        "sampled_at_frame": assets.sampled_at_frame,
        "sample_age_frames": current_frame.saturating_sub(assets.sampled_at_frame),
    })
}

fn visual_asset_metrics_json(
    metrics: VisualAssetPipelineMetrics,
    visible_world_fixture_count: usize,
) -> Value {
    json!({
        "slot_count": metrics.slot_count,
        "gltf_scene_slot_count": metrics.gltf_scene_slot_count,
        "ready_slot_count": metrics.ready_slot_count,
        "placeholder_slot_count": metrics.placeholder_slot_count,
        "streaming_slot_count": metrics.streaming_slot_count,
        "missing_slot_count": metrics.missing_slot_count,
        "queued_scene_count": metrics.queued_scene_count,
        "loading_scene_count": metrics.loading_scene_count,
        "loaded_scene_count": metrics.loaded_scene_count,
        "preload_ready_scene_count": metrics.preload_ready_scene_count,
        "failed_scene_count": metrics.failed_scene_count,
        "spawned_scene_count": metrics.spawned_scene_count,
        "ready_scene_count": metrics.ready_scene_count,
        "always_slot_count": metrics.always_slot_count,
        "stream_window_slot_count": metrics.stream_window_slot_count,
        "near_lod_slot_count": metrics.near_lod_slot_count,
        "far_lod_slot_count": metrics.far_lod_slot_count,
        "weather_slot_count": metrics.weather_slot_count,
        "visible_world_fixture_count": visible_world_fixture_count,
    })
}

fn content_diagnostics_json(diagnostics: IslandContentDiagnostics) -> Value {
    json!({
        "island_terrain_surface_count": diagnostics.island_terrain_surface_count,
        "min_island_terrain_mesh_vertices": diagnostics.min_island_terrain_mesh_vertices,
        "min_island_terrain_color_bands": diagnostics.min_island_terrain_color_bands,
        "min_island_terrain_material_weight_bands": diagnostics.min_island_terrain_material_weight_bands,
        "min_island_terrain_material_channels": diagnostics.min_island_terrain_material_channels,
        "min_island_terrain_material_regions": diagnostics.min_island_terrain_material_regions,
        "min_island_terrain_texture_detail_bands": diagnostics.min_island_terrain_texture_detail_bands,
        "min_island_terrain_relief_range_m": round3(diagnostics.min_island_terrain_relief_range_m() as f64),
        "island_terrain_archetype_count": diagnostics.island_terrain_archetype_count(),
        "generated_ground_cover_patch_count": diagnostics.generated_ground_cover_patch_count,
        "generated_tree_trunk_count": diagnostics.generated_tree_trunk_count,
        "generated_tree_canopy_count": diagnostics.generated_tree_canopy_count,
        "generated_rock_count": diagnostics.generated_rock_count,
        "generated_landmark_count": diagnostics.generated_landmark_count,
        "generated_weather_cloud_count": diagnostics.generated_weather_cloud_count,
        "generated_weather_cloud_bank_count": diagnostics.generated_weather_cloud_bank_count,
    })
}

fn mesh_triangle_count(mesh: &Mesh) -> usize {
    match mesh.indices() {
        Some(Indices::U16(indices)) => indices.len() / 3,
        Some(Indices::U32(indices)) => indices.len() / 3,
        None => mesh.count_vertices() / 3,
    }
}

fn vec3_json(value: Option<Vec3>) -> Value {
    match value {
        Some(value) => json!({
            "x": round3(value.x as f64),
            "y": round3(value.y as f64),
            "z": round3(value.z as f64),
        }),
        None => Value::Null,
    }
}

fn percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let index =
        ((sorted.len().saturating_sub(1)) as f64 * percentile.clamp(0.0, 1.0)).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn fps_from_frame_time_ms(frame_time_ms: f64) -> f64 {
    if frame_time_ms <= f64::EPSILON {
        0.0
    } else {
        round3(1000.0 / frame_time_ms)
    }
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_stats_match_eval_percentile_convention() {
        let stats = PlayProfileFrameStats::from_frame_times(&[10.0, 20.0, 30.0, 40.0, 50.0]);

        assert_eq!(stats.sample_count, 5);
        assert_eq!(stats.avg_frame_time_ms, 30.0);
        assert_eq!(stats.p95_frame_time_ms, 50.0);
        assert_eq!(stats.p99_frame_time_ms, 50.0);
        assert_eq!(stats.max_frame_time_ms, 50.0);
        assert_eq!(stats.frames_over_16ms, 4);
        assert_eq!(stats.frames_over_33ms, 2);
        assert_eq!(stats.frames_over_50ms, 0);
        assert_eq!(stats.frames_over_100ms, 0);
    }

    #[test]
    fn frame_stats_count_visible_lag_spikes() {
        let stats = PlayProfileFrameStats::from_frame_times(&[16.0, 34.0, 51.0, 101.0]);

        assert_eq!(stats.frames_over_16ms, 3);
        assert_eq!(stats.frames_over_33ms, 3);
        assert_eq!(stats.frames_over_50ms, 2);
        assert_eq!(stats.frames_over_100ms, 1);
    }

    #[test]
    fn play_profile_checks_pass_for_long_smooth_play() {
        let stats = PlayProfileFrameStats::from_frame_times(&[16.0, 17.0, 18.0, 19.0]);
        let checks = play_profile_checks(45.0, active_play_activity(), stats);

        assert!(
            checks.iter().all(|check| check.passed),
            "expected all checks to pass: {checks:?}"
        );
    }

    #[test]
    fn play_profile_checks_reject_short_or_laggy_play() {
        let short_stats = PlayProfileFrameStats::from_frame_times(&[16.0, 17.0, 18.0, 19.0]);
        let short_checks = play_profile_checks(5.0, active_play_activity(), short_stats);
        let duration_check = named_check(&short_checks, "play_profile_duration");

        assert!(!duration_check.passed);

        let laggy_stats = PlayProfileFrameStats::from_frame_times(&[16.0, 40.0, 90.0, 120.0]);
        let laggy_checks = play_profile_checks(45.0, active_play_activity(), laggy_stats);

        assert!(!named_check(&laggy_checks, "play_profile_steady_avg_frame_time_budget").passed);
        assert!(!named_check(&laggy_checks, "play_profile_steady_p95_frame_time_budget").passed);
        assert!(!named_check(&laggy_checks, "play_profile_steady_p99_frame_time_budget").passed);

        let spiky_stats =
            PlayProfileFrameStats::from_frame_times(&[16.0, 51.0, 52.0, 53.0, 54.0, 120.0, 121.0]);
        let spiky_checks = play_profile_checks(45.0, active_play_activity(), spiky_stats);

        assert!(!named_check(&spiky_checks, "play_profile_steady_50ms_hitch_count").passed);
        assert!(!named_check(&spiky_checks, "play_profile_steady_100ms_hitch_count").passed);
    }

    #[test]
    fn play_profile_checks_reject_idle_play() {
        let stats = PlayProfileFrameStats::from_frame_times(&[16.0, 17.0, 18.0, 19.0]);
        let checks = play_profile_checks(45.0, PlayProfileActivity::default(), stats);

        assert!(!named_check(&checks, "play_profile_horizontal_travel").passed);
    }

    #[test]
    fn play_profile_checks_reject_an_empty_steady_window() {
        let checks = play_profile_checks(
            45.0,
            active_play_activity(),
            PlayProfileFrameStats::default(),
        );

        assert!(!named_check(&checks, "play_profile_steady_sample_count").passed);
    }

    #[test]
    fn steady_profile_checks_ignore_startup_warmup_hitches() {
        let frame_times = [1500.0, 600.0, 16.0, 17.0, 18.0, 19.0];
        let total_stats = PlayProfileFrameStats::from_frame_times(&frame_times);
        let steady_frame_times = frame_times_after_warmup(&frame_times, 2.0);
        let steady_stats = PlayProfileFrameStats::from_frame_times(&steady_frame_times);
        let checks = play_profile_checks(45.0, active_play_activity(), steady_stats);

        assert_eq!(total_stats.frames_over_100ms, 2);
        assert_eq!(steady_frame_times, vec![16.0, 17.0, 18.0, 19.0]);
        assert!(
            named_check(&checks, "play_profile_steady_100ms_hitch_count").passed,
            "startup hitches should remain in total stats but not fail steady-play checks"
        );
    }

    #[test]
    fn play_profile_run_writes_foreground_report_shape() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "smooth"
        ));
        let _ = fs::remove_file(&output_path);
        let mut profile = PlayProfileRun::new(output_path.clone(), None, None)
            .expect("profile output should initialize");

        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::ZERO),
                ..default()
            },
        );
        assert!(!profile.armed);

        for frame in 0..2000 {
            profile.observe_frame(
                16.0,
                PlayProfileSnapshot {
                    player_position: Some(Vec3::new(2.0 + frame as f32 * 0.05, 0.0, 0.0)),
                    window: PlayProfileWindowSnapshot {
                        focused: Some(true),
                        present_mode: Some(PresentMode::Fifo),
                        monitor_count: 2,
                        min_monitor_refresh_rate_hz: Some(100.0),
                        max_monitor_refresh_rate_hz: Some(120.0),
                        primary_monitor_refresh_rate_hz: Some(120.0),
                    },
                    world_floor: WorldFloorDiagnostics {
                        initial_spawned_tile_count: 9,
                        ..default()
                    },
                    ..default()
                },
            );
        }
        assert!(profile.armed);
        profile.observe_assets(RuntimeAssetSnapshot {
            mesh_count: 3,
            material_count: 2,
            loaded_mesh_vertices: 300,
            loaded_mesh_triangles: 100,
            sampled_at_frame: 2000,
        });
        profile
            .write_summary()
            .expect("profile report should be written");

        let report: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&output_path).expect("profile report should exist"),
        )
        .expect("profile report should be valid json");
        let check_names = report["checks"]
            .as_array()
            .expect("profile checks should be an array")
            .iter()
            .map(|check| {
                check["name"]
                    .as_str()
                    .expect("check name should be a string")
            })
            .collect::<Vec<_>>();

        assert_eq!(report["schema_version"], 2);
        assert_eq!(report["profile_kind"], "manual_play_foreground");
        assert_eq!(report["control_source"], "manual");
        assert_eq!(report["script"], serde_json::Value::Null);
        assert_eq!(report["armed"], true);
        assert_eq!(report["arming"]["required_horizontal_travel_m"], 1.0);
        assert_eq!(report["arming"]["required_window_focused"], true);
        assert_eq!(report["passed"], true);
        assert_eq!(report["duration_secs"], 32.0);
        assert_eq!(report["target_duration_secs"], serde_json::Value::Null);
        assert_eq!(report["activity"]["horizontal_travel_m"], 99.95);
        assert_eq!(report["activity"]["max_horizontal_displacement_m"], 99.95);
        assert_eq!(report["window_focus"]["focused_samples"], 2000);
        assert_eq!(report["window_focus"]["unfocused_samples"], 0);
        assert_eq!(report["window_focus"]["focused_secs"], 32.0);
        assert_eq!(report["window_focus"]["unfocused_secs"], 0.0);
        assert_eq!(report["window_focus"]["focused_ratio"], 1.0);
        assert_eq!(report["frame_time"]["sample_count"], 2000);
        assert_eq!(report["steady_frame_time"]["avg_ms"], 16.0);
        assert_eq!(report["steady_frame_time"]["frames_over_100ms"], 0);
        assert_eq!(report["hitch_event_threshold_ms"], 25.0);
        assert_eq!(report["max_hitch_events"], PROFILE_MAX_HITCH_EVENTS);
        assert_eq!(report["latest"]["window"]["focused"], true);
        assert_eq!(report["latest"]["window"]["present_mode"], "fifo");
        assert_eq!(report["latest"]["window"]["monitor_count"], 2);
        assert_eq!(
            report["latest"]["window"]["min_monitor_refresh_rate_hz"],
            100.0
        );
        assert_eq!(
            report["latest"]["window"]["max_monitor_refresh_rate_hz"],
            120.0
        );
        assert_eq!(
            report["latest"]["window"]["primary_monitor_refresh_rate_hz"],
            120.0
        );
        assert_eq!(report["latest"]["runtime_assets"]["sampled_at_frame"], 2000);
        assert_eq!(report["latest"]["runtime_assets"]["sample_age_frames"], 0);
        assert_eq!(
            report["hitch_events"]
                .as_array()
                .expect("hitch events should be an array")
                .len(),
            0
        );
        assert_eq!(report["max"]["mesh_count"], 3);
        assert_eq!(report["max"]["loaded_mesh_triangles"], 100);
        assert_eq!(report["max"]["world_floor_initial_spawned_tile_count"], 9);
        assert!(check_names.contains(&"play_profile_duration"));
        assert!(check_names.contains(&"play_profile_horizontal_travel"));
        assert!(check_names.contains(&"play_profile_steady_sample_count"));
        assert!(check_names.contains(&"play_profile_steady_avg_frame_time_budget"));
        assert!(check_names.contains(&"play_profile_steady_p95_frame_time_budget"));
        assert!(check_names.contains(&"play_profile_steady_p99_frame_time_budget"));
        assert!(check_names.contains(&"play_profile_steady_50ms_hitch_count"));
        assert!(check_names.contains(&"play_profile_steady_100ms_hitch_count"));
        assert!(check_names.contains(&"play_profile_window_focused_ratio"));

        fs::remove_file(output_path).expect("profile report should be removable");
    }

    #[test]
    fn manual_play_profile_does_not_override_runtime_input() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "manual_input"
        ));
        let profile = PlayProfileRun::new(output_path.clone(), Some(30.0), None)
            .expect("manual profile output should initialize");

        assert_eq!(profile.scripted_flight_input(), None);
        assert_eq!(profile.scripted_camera_input(1.0 / 60.0), None);

        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn scripted_camera_motion_integrates_equally_across_frame_rates() {
        let mut integrated = Vec::new();
        for frame_rate in [30.0, 60.0, 120.0, 144.0] {
            let dt = 1.0 / frame_rate;
            let delta = (0..frame_rate as usize).fold(Vec2::ZERO, |sum, _| {
                sum + scripted_profile_camera_input(PlayProfileScript::Freeflight, 6.4, dt)
                    .mouse_delta
            });
            integrated.push(delta);
        }

        for delta in integrated.iter().skip(1) {
            assert!((*delta - integrated[0]).length() <= 0.01);
        }
    }

    #[test]
    fn timed_play_profiles_refresh_deep_asset_metrics_only_at_boundaries() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "timed_asset_refresh"
        ));
        let mut timed_profile = PlayProfileRun::new(output_path.clone(), Some(30.0), None)
            .expect("timed profile output should initialize");
        assert!(timed_profile.should_refresh_assets_during_run());

        timed_profile.observe_assets(RuntimeAssetSnapshot {
            mesh_count: 10,
            material_count: 4,
            loaded_mesh_vertices: 100,
            loaded_mesh_triangles: 50,
            sampled_at_frame: 1,
        });
        assert!(!timed_profile.should_refresh_assets_during_run());

        let mut continuous_profile = PlayProfileRun::new(output_path.clone(), None, None)
            .expect("continuous profile output should initialize");
        continuous_profile.observe_assets(RuntimeAssetSnapshot {
            mesh_count: 10,
            material_count: 4,
            loaded_mesh_vertices: 100,
            loaded_mesh_triangles: 50,
            sampled_at_frame: 1,
        });
        assert!(continuous_profile.should_refresh_assets_during_run());

        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn play_profile_run_records_bounded_hitch_events() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "hitches"
        ));
        let _ = fs::remove_file(&output_path);
        let mut profile = PlayProfileRun::new(output_path.clone(), None, None)
            .expect("profile output should initialize");

        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::ZERO),
                ..default()
            },
        );
        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(2.0, 0.0, 0.0)),
                ..default()
            },
        );
        assert!(profile.armed);

        for frame in 0..(PROFILE_MAX_HITCH_EVENTS + 2) {
            profile.observe_frame(
                120.0,
                PlayProfileSnapshot {
                    player_position: Some(Vec3::new(4.0 + frame as f32, 0.0, 0.0)),
                    entity_count: 100 + frame,
                    runtime_assets: RuntimeAssetSnapshot {
                        mesh_count: 10 + frame,
                        material_count: 4,
                        loaded_mesh_vertices: 1000,
                        loaded_mesh_triangles: 500,
                        sampled_at_frame: frame as u64 + 1,
                    },
                    ..default()
                },
            );
        }

        profile
            .write_summary()
            .expect("profile report should be written");
        let report: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&output_path).expect("profile report should exist"),
        )
        .expect("profile report should be valid json");
        let hitches = report["hitch_events"]
            .as_array()
            .expect("hitch events should be an array");

        assert_eq!(hitches.len(), PROFILE_MAX_HITCH_EVENTS);
        assert_eq!(hitches[0]["severity"], "over_100ms");
        assert_eq!(hitches[0]["frame_time_ms"], 120.0);
        assert_eq!(hitches[0]["snapshot"]["entity_count"], 100);
        assert_eq!(hitches[0]["snapshot"]["runtime_assets"]["mesh_count"], 10);
        assert!(
            hitches
                .iter()
                .any(|hitch| hitch["steady"] == serde_json::Value::Bool(true)),
            "long hitch sequence should include steady-window events"
        );

        fs::remove_file(output_path).expect("profile report should be removable");
    }

    #[test]
    fn play_profile_retains_late_severe_hitches_after_event_buffer_fills() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "late_severe_hitch"
        ));
        let _ = fs::remove_file(&output_path);
        let mut profile = PlayProfileRun::new(output_path.clone(), None, None)
            .expect("profile output should initialize");

        for frame in 0..PROFILE_MAX_HITCH_EVENTS {
            profile.observe_hitch_event(
                PROFILE_HITCH_EVENT_THRESHOLD_MS + frame as f64,
                PlayProfileSnapshot {
                    frame: frame as u64 + 1,
                    elapsed_secs: frame as f64,
                    ..default()
                },
            );
        }
        profile.observe_hitch_event(
            200.0,
            PlayProfileSnapshot {
                frame: 100,
                elapsed_secs: 10.0,
                ..default()
            },
        );
        profile.observe_hitch_event(
            PROFILE_HITCH_EVENT_THRESHOLD_MS,
            PlayProfileSnapshot {
                frame: 101,
                elapsed_secs: 11.0,
                ..default()
            },
        );

        profile
            .write_summary()
            .expect("profile report should be written");
        let report: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&output_path).expect("profile report should exist"),
        )
        .expect("profile report should be valid json");
        let hitches = report["hitch_events"]
            .as_array()
            .expect("hitch events should be an array");

        assert_eq!(hitches.len(), PROFILE_MAX_HITCH_EVENTS);
        assert_eq!(hitches[0]["frame"], 2);
        assert_eq!(
            hitches.last().expect("late hitch should be retained")["frame"],
            100
        );
        assert!(
            hitches.iter().any(|hitch| hitch["frame_time_ms"] == 200.0),
            "the late severe hitch should replace the least severe retained event"
        );
        assert!(
            hitches.iter().all(|hitch| hitch["frame"] != 101),
            "a later hitch no worse than the retained set should be discarded"
        );

        fs::remove_file(output_path).expect("profile report should be removable");
    }

    #[test]
    fn play_profile_records_25ms_hitches_with_window_context() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}",
            std::process::id(),
            "minor_hitch.json"
        ));
        let _ = fs::remove_file(&output_path);
        let mut profile = PlayProfileRun::new(output_path.clone(), None, None)
            .expect("profile output should initialize");

        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::ZERO),
                ..default()
            },
        );
        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(2.0, 0.0, 0.0)),
                ..default()
            },
        );
        profile.observe_hitch_event(25.0, PlayProfileSnapshot::default());
        assert!(profile.hitch_events.is_empty());
        profile.observe_frame(
            25.1,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(3.0, 0.0, 0.0)),
                window: PlayProfileWindowSnapshot {
                    focused: Some(true),
                    present_mode: Some(PresentMode::Fifo),
                    monitor_count: 2,
                    min_monitor_refresh_rate_hz: Some(100.0),
                    max_monitor_refresh_rate_hz: Some(120.0),
                    primary_monitor_refresh_rate_hz: Some(120.0),
                },
                ..default()
            },
        );
        profile
            .write_summary()
            .expect("profile report should be written");

        let report: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&output_path).expect("profile report should exist"),
        )
        .expect("profile report should be valid json");
        let hitch = &report["hitch_events"][0];
        assert_eq!(hitch["severity"], "over_25ms");
        assert_eq!(hitch["frame_time_ms"], 25.1);
        assert_eq!(hitch["snapshot"]["window"]["focused"], true);
        assert_eq!(hitch["snapshot"]["window"]["present_mode"], "fifo");
        assert_eq!(hitch["snapshot"]["window"]["monitor_count"], 2);
        assert_eq!(
            hitch["snapshot"]["window"]["min_monitor_refresh_rate_hz"],
            100.0
        );
        assert_eq!(
            hitch["snapshot"]["window"]["max_monitor_refresh_rate_hz"],
            120.0
        );
        assert_eq!(
            hitch["snapshot"]["window"]["primary_monitor_refresh_rate_hz"],
            120.0
        );

        fs::remove_file(output_path).expect("profile report should be removable");
    }

    #[test]
    fn play_profile_focus_gate_weights_elapsed_time_not_frame_count() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "time_weighted_focus"
        ));
        let mut profile =
            PlayProfileRun::new(output_path, None, None).expect("profile output should initialize");

        profile.observe_frame(
            10.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::ZERO),
                window: PlayProfileWindowSnapshot {
                    focused: Some(true),
                    ..default()
                },
                ..default()
            },
        );
        for frame in 0..19 {
            profile.observe_frame(
                10.0,
                PlayProfileSnapshot {
                    player_position: Some(Vec3::new(2.0 + frame as f32, 0.0, 0.0)),
                    window: PlayProfileWindowSnapshot {
                        focused: Some(true),
                        ..default()
                    },
                    ..default()
                },
            );
        }
        profile.observe_frame(
            100.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(22.0, 0.0, 0.0)),
                window: PlayProfileWindowSnapshot {
                    focused: Some(false),
                    ..default()
                },
                ..default()
            },
        );

        assert_eq!(profile.focused_window_samples, 19);
        assert_eq!(profile.unfocused_window_samples, 1);
        assert_eq!(
            focused_window_ratio(
                profile.focused_window_samples as f64,
                profile.unfocused_window_samples as f64,
            ),
            0.95
        );
        assert!(
            !play_profile_window_focus_check(
                profile.focused_window_secs,
                profile.unfocused_window_secs,
            )
            .passed
        );
    }

    #[test]
    fn play_profile_window_focus_check_rejects_unfocused_runs() {
        let check = play_profile_window_focus_check(94.0, 6.0);

        assert!(!check.passed);
        assert_eq!(check.value, 0.94);
        assert_eq!(check.threshold, PROFILE_MIN_FOCUSED_WINDOW_RATIO);

        let near_threshold = play_profile_window_focus_check(94.99, 5.01);
        let near_threshold_json = near_threshold.to_json();
        assert!(!near_threshold.passed);
        assert_eq!(near_threshold_json["value"], 0.9499);
        assert_eq!(near_threshold_json["threshold"], 0.95);
    }

    #[test]
    fn play_profile_arming_waits_for_foreground_focus() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "focus_arming"
        ));
        let mut profile = PlayProfileRun::new(output_path, Some(30.0), None)
            .expect("profile output should initialize");

        for position_x in [0.0, 2.0] {
            profile.observe_frame(
                16.0,
                PlayProfileSnapshot {
                    player_position: Some(Vec3::new(position_x, 0.0, 0.0)),
                    window: PlayProfileWindowSnapshot {
                        focused: Some(false),
                        ..default()
                    },
                    ..default()
                },
            );
        }
        assert!(!profile.armed);

        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(3.0, 0.0, 0.0)),
                window: PlayProfileWindowSnapshot {
                    focused: Some(true),
                    ..default()
                },
                ..default()
            },
        );
        assert!(!profile.armed);

        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(4.1, 0.0, 0.0)),
                window: PlayProfileWindowSnapshot {
                    focused: Some(true),
                    ..default()
                },
                ..default()
            },
        );
        assert!(profile.armed);
        assert_eq!(profile.focused_window_samples, 1);
        assert_eq!(profile.unfocused_window_samples, 0);
    }

    #[test]
    fn play_profile_run_reports_target_duration_reached() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "timed"
        ));
        let _ = fs::remove_file(&output_path);
        let mut profile = PlayProfileRun::new(output_path.clone(), Some(30.0), None)
            .expect("profile output should initialize");

        assert!(!profile.target_duration_reached());

        profile.observe_frame(29_999.0, PlayProfileSnapshot::default());
        assert!(!profile.target_duration_reached());

        profile.observe_frame(1.0, PlayProfileSnapshot::default());
        assert!(profile.target_duration_reached());

        let mut profile = PlayProfileRun::new(output_path.clone(), Some(30.0), None)
            .expect("profile output should initialize");
        profile.observe_frame(
            250.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::ZERO),
                ..default()
            },
        );
        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(2.0, 0.0, 0.0)),
                ..default()
            },
        );
        assert!(profile.armed);
        assert!((profile.elapsed_secs - 0.016).abs() < f64::EPSILON);
        assert!(!profile.target_duration_reached());

        profile.observe_frame(29_984.0, PlayProfileSnapshot::default());
        assert!(profile.target_duration_reached());
        profile
            .write_summary()
            .expect("timed profile report should be written");

        let report: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&output_path).expect("timed profile report should exist"),
        )
        .expect("timed profile report should be valid json");
        assert_eq!(report["target_duration_secs"], 30.0);

        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn scripted_profile_reports_scripted_source_and_inputs() {
        let output_path = std::env::temp_dir().join(format!(
            "nau_play_profile_report_{}_{}.json",
            std::process::id(),
            "scripted"
        ));
        let _ = fs::remove_file(&output_path);
        let mut profile = PlayProfileRun::new(
            output_path.clone(),
            Some(30.0),
            Some(PlayProfileScript::Freeflight),
        )
        .expect("profile output should initialize");

        let launch_input = profile
            .scripted_flight_input()
            .expect("scripted profile should provide input");
        assert!(launch_input.launch);
        assert!(!launch_input.glide);

        profile.observe_frame(
            500.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::ZERO),
                ..default()
            },
        );
        profile.observe_frame(
            16.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(2.0, 0.0, 0.0)),
                ..default()
            },
        );
        assert!(profile.armed);
        profile.observe_frame(
            500.0,
            PlayProfileSnapshot {
                player_position: Some(Vec3::new(4.0, 0.0, 0.0)),
                ..default()
            },
        );

        let active_input = profile
            .scripted_flight_input()
            .expect("scripted profile should keep providing input");
        assert!(active_input.forward);
        assert!(active_input.glide);

        profile
            .write_summary()
            .expect("scripted profile report should be written");
        let report: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&output_path).expect("scripted profile report should exist"),
        )
        .expect("scripted profile report should be valid json");
        assert_eq!(report["profile_kind"], "scripted_play_foreground");
        assert_eq!(report["control_source"], "scripted");
        assert_eq!(report["script"], "freeflight");

        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn ground_traversal_profile_starts_grounded_and_never_launches() {
        let route = SkyRoute::default();
        let profile = PlayProfileRun::new(
            std::env::temp_dir().join("nau_ground_traversal_profile.json"),
            Some(30.0),
            Some(PlayProfileScript::GroundTraversal),
        )
        .expect("profile output should initialize");
        let start = profile
            .scripted_start_position(&route)
            .expect("ground traversal should define a start");
        let input = profile
            .scripted_flight_input()
            .expect("ground traversal should provide input");

        assert_eq!(start.y, route.ground_at(start).floor_y);
        assert_eq!(route.ground_at(start).island_name, None);
        assert!(input.forward);
        assert!(!input.launch);
        assert!(!input.glide);
        assert!(!input.dive);
    }

    fn named_check<'a>(checks: &'a [PlayProfileCheck], name: &str) -> &'a PlayProfileCheck {
        checks
            .iter()
            .find(|check| check.name == name)
            .expect("named profile check should exist")
    }

    fn active_play_activity() -> PlayProfileActivity {
        let mut activity = PlayProfileActivity::default();
        activity.observe_position(Some(Vec3::ZERO));
        activity.observe_position(Some(Vec3::new(60.0, 0.0, 0.0)));
        activity
    }
}
