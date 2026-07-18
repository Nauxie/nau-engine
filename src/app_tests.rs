use super::*;
use bevy::gltf::Gltf;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::pbr::ScatteringMedium;
use nau_engine::animation::{AnimationState, PlayerPoseIntent};
use nau_engine::environment::{AERIAL_POWER_UP_ROUTE, LiftField, WindField};
use nau_engine::movement::{FlightController, FlightInput, Velocity};
use nau_engine::world::{
    IslandPlateauRegion, IslandReviewView, IslandScaleClass, IslandTerrainArchetype,
    IslandWaterFeature,
};

fn test_island() -> SkyIsland {
    SkyIsland::new(
        "test island",
        Vec3::new(12.0, 40.0, -8.0),
        Vec2::new(22.0, 15.0),
        12.0,
        false,
    )
}

#[test]
fn aerial_power_up_gates_clear_the_world_surface() {
    let route = SkyRoute::default();

    for power_up in AERIAL_POWER_UP_ROUTE {
        let samples = std::iter::once(power_up.center).chain((0..8).map(|sample_index| {
            let angle = sample_index as f32 / 8.0 * std::f32::consts::TAU;
            power_up.center
                + Vec3::new(
                    angle.cos() * power_up.radius_m,
                    0.0,
                    angle.sin() * power_up.radius_m,
                )
        }));
        for (sample_index, sample) in samples.enumerate() {
            let ground = route.ground_at(sample);
            assert!(
                power_up.center.y - power_up.radius_m > ground.floor_y + 2.0,
                "{} should clear the world surface at sample {}, center_y={}, floor_y={}, radius={}",
                power_up.name,
                sample_index,
                power_up.center.y,
                ground.floor_y,
                power_up.radius_m
            );
        }
    }
}

fn positions(mesh: &Mesh) -> &[[f32; 3]] {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => panic!("mesh should expose Float32x3 positions"),
    }
}

fn u32_indices(mesh: &Mesh) -> &[u32] {
    match mesh.indices() {
        Some(Indices::U32(values)) => values,
        _ => panic!("mesh should expose U32 indices"),
    }
}

fn colors(mesh: &Mesh) -> &[[f32; 4]] {
    match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(values)) => values,
        _ => panic!("mesh should expose Float32x4 vertex colors"),
    }
}

fn triangle_normal_y(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let a = Vec3::from_array(positions[indices[0] as usize]);
    let b = Vec3::from_array(positions[indices[1] as usize]);
    let c = Vec3::from_array(positions[indices[2] as usize]);
    (b - a).cross(c - a).y
}

fn normalized_radius(island: SkyIsland, position: [f32; 3]) -> f32 {
    Vec2::new(
        (position[0] - island.center.x) / island.half_extents.x,
        (position[2] - island.center.z) / island.half_extents.y,
    )
    .length()
}

fn normalized_visual_radius(island: SkyIsland, position: [f32; 3]) -> f32 {
    let normalized = Vec2::new(
        (position[0] - island.center.x) / island.half_extents.x,
        (position[2] - island.center.z) / island.half_extents.y,
    );
    normalized.length() / island.visual_silhouette_scale(normalized.y.atan2(normalized.x))
}

fn radial_range(positions: &[[f32; 3]]) -> f32 {
    let mut min_radius = f32::INFINITY;
    let mut max_radius = f32::NEG_INFINITY;
    for position in positions {
        let radius = Vec2::new(position[0], position[2]).length();
        min_radius = min_radius.min(radius);
        max_radius = max_radius.max(radius);
    }
    max_radius - min_radius
}

const EXPECTED_ISLAND_IMPOSTOR_MESH_VERTICES: usize = 1
    + ISLAND_IMPOSTOR_TERRAIN_RINGS * ISLAND_IMPOSTOR_SEGMENTS
    + (ISLAND_IMPOSTOR_CLIFF_RINGS + 1) * ISLAND_IMPOSTOR_SEGMENTS
    + (ISLAND_IMPOSTOR_UNDERSIDE_RINGS + 1) * ISLAND_IMPOSTOR_SEGMENTS
    + 1;

#[test]
fn marker_occlusion_detects_island_between_camera_and_marker() {
    let island = SkyIsland::new(
        "blocking island",
        Vec3::new(0.0, 40.0, -40.0),
        Vec2::new(22.0, 16.0),
        14.0,
        false,
    );
    let occlusion = marker_occlusion_between(
        Vec3::new(0.0, 40.0, 0.0),
        Vec3::new(0.0, 40.0, -96.0),
        &[island],
    )
    .expect("island should block the marker ray");

    assert_eq!(occlusion.island_name, "blocking island");
    assert!(occlusion.distance_m > 20.0);
    assert!(occlusion.distance_m < 70.0);
}

#[test]
fn marker_occlusion_ignores_clear_high_ray() {
    let island = SkyIsland::new(
        "low island",
        Vec3::new(0.0, 40.0, -40.0),
        Vec2::new(22.0, 16.0),
        14.0,
        false,
    );

    assert!(
        marker_occlusion_between(
            Vec3::new(0.0, 72.0, 0.0),
            Vec3::new(0.0, 72.0, -96.0),
            &[island],
        )
        .is_none()
    );
}

#[test]
fn authored_player_clip_selection_tracks_flight_state() {
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Grounded, 0.2),
        AuthoredPlayerClip::Idle
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Grounded, 4.0),
        AuthoredPlayerClip::Walk
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Grounded, 7.0),
        AuthoredPlayerClip::Run
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Launching, 18.0),
        AuthoredPlayerClip::Launch
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Gliding, 40.0),
        AuthoredPlayerClip::Glide
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Airborne, 16.0),
        AuthoredPlayerClip::Fall
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Airborne, 4.0),
        AuthoredPlayerClip::Fall
    );
}

#[test]
fn authored_player_clip_selection_tracks_pose_intent() {
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedIdle, 0.2),
        AuthoredPlayerClip::Idle
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedWalk, 4.0),
        AuthoredPlayerClip::Walk
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedStride, 4.0),
        AuthoredPlayerClip::Run
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedRun, 4.0),
        AuthoredPlayerClip::Run
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::Diving, 34.0),
        AuthoredPlayerClip::Dive
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::AirTurn, 28.0),
        AuthoredPlayerClip::Glide
    );
    assert_eq!(
        authored_player_clip_for_pose_intent_with_input(
            PlayerPoseIntent::AirTurn,
            28.0,
            FlightInput {
                left: true,
                ..default()
            },
        ),
        AuthoredPlayerClip::BankLeft
    );
    assert_eq!(
        authored_player_clip_for_pose_intent_with_input(
            PlayerPoseIntent::AirTurn,
            28.0,
            FlightInput {
                right: true,
                ..default()
            },
        ),
        AuthoredPlayerClip::BankRight
    );
    assert_eq!(
        authored_player_clip_for_pose_intent_with_input(
            PlayerPoseIntent::Diving,
            34.0,
            FlightInput {
                right: true,
                dive: true,
                ..default()
            },
        ),
        AuthoredPlayerClip::Dive
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::AirBrake, 28.0),
        AuthoredPlayerClip::AirBrake
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::LandingAnticipation, 12.0),
        AuthoredPlayerClip::Land
    );
}

#[test]
fn island_hero_gallery_forces_matching_controller_and_animation_pose_for_every_view() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-gallery-pose-parity-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    let scenario = scenario_named(ISLAND_HERO_GALLERY).expect("gallery scenario should exist");
    let run = EvalRun::new(EvalOptions {
        scenario,
        output_dir: output_dir.clone(),
        capture_screenshot: false,
        visible_window: false,
    })
    .expect("gallery run should initialize");
    let frames_per_view = run
        .scenario
        .island_hero_gallery_timing()
        .expect("gallery timing should exist")
        .frames_per_view;
    let mut app = App::new();
    app.insert_resource(run)
        .insert_resource(PlayerDisplacementDiagnostics::default())
        .add_systems(Update, fix_island_hero_gallery_player);
    app.world_mut().spawn((
        Player,
        Transform::default(),
        Velocity(Vec3::splat(99.0)),
        FlightController::default(),
        AnimationState::default(),
    ));

    let expected_views = [
        (
            IslandReviewView::Near,
            FlightMode::Grounded,
            PlayerPoseIntent::GroundedIdle,
        ),
        (
            IslandReviewView::Mid,
            FlightMode::Gliding,
            PlayerPoseIntent::Gliding,
        ),
        (
            IslandReviewView::Traversal,
            FlightMode::Gliding,
            PlayerPoseIntent::Gliding,
        ),
    ];

    for (view_index, (expected_view, expected_mode, expected_intent)) in
        expected_views.into_iter().enumerate()
    {
        app.world_mut().resource_mut::<EvalRun>().frame = view_index as u32 * frames_per_view;
        {
            let world = app.world_mut();
            let mut query = world
                .query_filtered::<(&mut FlightController, &mut AnimationState), With<Player>>();
            let (mut controller, mut animation) = query
                .single_mut(world)
                .expect("gallery player should exist");
            controller.mode = if expected_mode == FlightMode::Grounded {
                FlightMode::Gliding
            } else {
                FlightMode::Grounded
            };
            animation.pose_intent = if expected_intent == PlayerPoseIntent::GroundedIdle {
                PlayerPoseIntent::Gliding
            } else {
                PlayerPoseIntent::GroundedIdle
            };
        }

        app.update();

        let pose = app
            .world()
            .resource::<EvalRun>()
            .island_review_pose()
            .expect("gallery pose should exist");
        assert_eq!(pose.view, expected_view);
        let world = app.world_mut();
        let mut query = world.query_filtered::<
            (&Transform, &Velocity, &FlightController, &AnimationState),
            With<Player>,
        >();
        let (transform, velocity, controller, animation) =
            query.single(world).expect("gallery player should exist");
        assert_eq!(transform.translation, pose.player_position);
        assert_eq!(velocity.0, Vec3::ZERO);
        assert_eq!(controller.mode, expected_mode);
        assert_eq!(animation.pose_intent, expected_intent);
        assert!(
            controller.mode != FlightMode::Gliding
                || animation.pose_intent != PlayerPoseIntent::GroundedIdle,
            "{expected_view:?} must not deploy the glider with a grounded-idle body"
        );
    }

    drop(app);
    remove_existing_dir(&output_dir).expect("gallery pose test dir should be removable");
}

#[test]
fn authored_player_clip_indices_match_declared_gltf_order() {
    assert_eq!(AuthoredPlayerClip::Idle.index(), 0);
    assert_eq!(AuthoredPlayerClip::Walk.index(), 1);
    assert_eq!(AuthoredPlayerClip::Run.index(), 2);
    assert_eq!(AuthoredPlayerClip::Launch.index(), 3);
    assert_eq!(AuthoredPlayerClip::Fall.index(), 4);
    assert_eq!(AuthoredPlayerClip::Glide.index(), 5);
    assert_eq!(AuthoredPlayerClip::BankLeft.index(), 6);
    assert_eq!(AuthoredPlayerClip::BankRight.index(), 7);
    assert_eq!(AuthoredPlayerClip::Dive.index(), 8);
    assert_eq!(AuthoredPlayerClip::AirBrake.index(), 9);
    assert_eq!(AuthoredPlayerClip::Land.index(), 10);
}

#[test]
fn named_animation_clip_resolution_reports_missing_clips() {
    let mut named_animations = HashMap::new();
    named_animations.insert("Idle_Loop".to_string(), Handle::<AnimationClip>::default());
    named_animations.insert("Glide_Loop".to_string(), Handle::<AnimationClip>::default());

    let resolution = resolve_named_animation_clip_handles(
        &["Idle_Loop", "Walk_Fwd_Loop", "Glide_Loop"],
        &named_animations,
    );

    assert_eq!(resolution.ready_clip_count(), 2);
    assert_eq!(resolution.expected_clip_count, 3);
    assert_eq!(resolution.missing_clip_names, vec!["Walk_Fwd_Loop"]);
    assert!(!resolution.is_complete());
}

#[test]
fn parse_cli_args_defaults_to_clean_play_run_mode() {
    let action = parse_cli_args(std::iter::empty::<String>())
        .expect("empty args should run the clean play sandbox");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            assert!(eval.is_none());
            assert_eq!(mode, RunMode::Play);
            assert!(play_profile.is_none());
            assert!(!mode.debug_readout_enabled());
            assert!(!mode.debug_visuals_enabled());
            assert!(!mode.debug_visual_toggle_enabled());
        }
        _ => panic!("expected run action"),
    }
}

#[test]
fn parse_cli_args_accepts_play_mode() {
    let action =
        parse_cli_args(["--play"].into_iter().map(str::to_string)).expect("play args should parse");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            assert!(eval.is_none());
            assert_eq!(mode, RunMode::Play);
            assert!(play_profile.is_none());
            assert!(!mode.debug_readout_enabled());
            assert!(!mode.debug_visuals_enabled());
            assert!(!mode.debug_visual_toggle_enabled());
        }
        _ => panic!("expected play run action"),
    }
}

#[test]
fn parse_cli_args_accepts_debug_mode() {
    let action = parse_cli_args(["--debug"].into_iter().map(str::to_string))
        .expect("debug args should parse");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            assert!(eval.is_none());
            assert_eq!(mode, RunMode::Debug);
            assert!(play_profile.is_none());
            assert!(mode.debug_readout_enabled());
            assert!(mode.debug_visuals_enabled());
            assert!(mode.debug_visual_toggle_enabled());
        }
        _ => panic!("expected debug run action"),
    }
}

#[test]
fn parse_cli_args_accepts_manual_play_profile() {
    let action = parse_cli_args(
        [
            "--play",
            "--play-profile",
            "target/eval/manual_play/profile.json",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect("play profile args should parse");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            let play_profile = play_profile.expect("play profile options");
            assert!(eval.is_none());
            assert_eq!(mode, RunMode::Play);
            assert_eq!(
                play_profile.output_path,
                PathBuf::from("target/eval/manual_play/profile.json")
            );
            assert_eq!(play_profile.duration_secs, None);
            assert_eq!(play_profile.script, None);
        }
        _ => panic!("expected play profile run action"),
    }
}

#[test]
fn parse_cli_args_accepts_timed_manual_play_profile() {
    let action = parse_cli_args(
        [
            "--play",
            "--play-profile",
            "target/eval/manual_play/profile.json",
            "--play-profile-duration",
            "45",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect("timed play profile args should parse");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            let play_profile = play_profile.expect("play profile options");
            assert!(eval.is_none());
            assert_eq!(mode, RunMode::Play);
            assert_eq!(
                play_profile.output_path,
                PathBuf::from("target/eval/manual_play/profile.json")
            );
            assert_eq!(play_profile.duration_secs, Some(45.0));
            assert_eq!(play_profile.script, None);
        }
        _ => panic!("expected play profile run action"),
    }
}

#[test]
fn parse_cli_args_accepts_scripted_play_profile() {
    let action = parse_cli_args(
        [
            "--play",
            "--play-profile",
            "target/eval/play_profile/candidate_freeflight.json",
            "--play-profile-duration",
            "45",
            "--play-profile-script",
            "freeflight",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect("scripted play profile args should parse");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            let play_profile = play_profile.expect("play profile options");
            assert!(eval.is_none());
            assert_eq!(mode, RunMode::Play);
            assert_eq!(
                play_profile.output_path,
                PathBuf::from("target/eval/play_profile/candidate_freeflight.json")
            );
            assert_eq!(play_profile.duration_secs, Some(45.0));
            assert_eq!(
                play_profile.script,
                Some(crate::play_profile_runtime::PlayProfileScript::Freeflight)
            );
        }
        _ => panic!("expected scripted play profile run action"),
    }
}

#[test]
fn parse_cli_args_rejects_profile_options_without_output() {
    let error = parse_cli_args(
        ["--play", "--play-profile-duration", "45"]
            .into_iter()
            .map(str::to_string),
    )
    .expect_err("timed manual profile without output should be rejected");

    assert!(error.contains("--play-profile-duration requires --play-profile"));

    let error = parse_cli_args(
        ["--play", "--play-profile-script", "freeflight"]
            .into_iter()
            .map(str::to_string),
    )
    .expect_err("scripted profile without output should be rejected");

    assert!(error.contains("--play-profile-script requires --play-profile"));
}

#[test]
fn parse_cli_args_rejects_invalid_manual_play_profile_duration() {
    for duration in ["0", "-1", "nan", "inf", "abc"] {
        let error = parse_cli_args(
            [
                "--play",
                "--play-profile",
                "target/eval/manual_play/profile.json",
                "--play-profile-duration",
                duration,
            ]
            .into_iter()
            .map(str::to_string),
        )
        .expect_err("invalid timed manual profile duration should be rejected");

        assert!(error.contains("--play-profile-duration requires a positive finite number"));
    }
}

#[test]
fn parse_cli_args_rejects_unknown_play_profile_script() {
    let error = parse_cli_args(
        [
            "--play",
            "--play-profile",
            "target/eval/play_profile/profile.json",
            "--play-profile-script",
            "unknown",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("unknown scripted profile should be rejected");

    assert!(error.contains("unknown play profile script"));
}

#[test]
fn parse_cli_args_rejects_debug_manual_play_profile() {
    let error = parse_cli_args(
        [
            "--debug",
            "--play-profile",
            "target/eval/manual_play/profile.json",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("debug manual profile should be rejected");

    assert!(error.contains("--play-profile requires --play mode"));
}

#[test]
fn parse_cli_args_eval_defaults_to_clean_play_mode() {
    let action = parse_cli_args(["--eval", "baseline_route"].into_iter().map(str::to_string))
        .expect("eval args should parse");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            assert!(eval.is_some());
            assert_eq!(mode, RunMode::Play);
            assert!(play_profile.is_none());
            assert!(!mode.debug_readout_enabled());
            assert!(!mode.debug_visuals_enabled());
        }
        _ => panic!("expected eval run action"),
    }
}

#[test]
fn parse_cli_args_accepts_visible_metric_eval_window() {
    let action = parse_cli_args(
        [
            "--eval",
            "baseline_route",
            "--eval-no-screenshot",
            "--eval-visible-window",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect("visible metric eval args should parse");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            let eval = eval.expect("eval options");
            assert_eq!(mode, RunMode::Play);
            assert!(play_profile.is_none());
            assert!(!eval.capture_screenshot);
            assert!(eval.visible_window);
        }
        _ => panic!("expected eval run action"),
    }
}

#[test]
fn parse_cli_args_rejects_eval_only_flags_without_eval() {
    for args in [
        vec!["--eval-output", "target/eval/baseline_route"],
        vec!["--eval-no-screenshot"],
        vec!["--eval-visible-window"],
    ] {
        let error = parse_cli_args(args.into_iter().map(str::to_string))
            .expect_err("eval-only flag should require --eval");

        assert!(error.contains("eval options require --eval"));
    }
}

#[test]
fn parse_cli_args_accepts_debug_eval_mode() {
    let action = parse_cli_args(
        ["--eval", "baseline_route", "--debug"]
            .into_iter()
            .map(str::to_string),
    )
    .expect("debug eval args should parse");

    match action {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => {
            assert!(eval.is_some());
            assert_eq!(mode, RunMode::Debug);
            assert!(play_profile.is_none());
            assert!(mode.debug_readout_enabled());
            assert!(mode.debug_visuals_enabled());
        }
        _ => panic!("expected debug eval run action"),
    }
}

#[test]
fn parse_cli_args_accepts_terrain_export() {
    let action = parse_cli_args(
        ["--export-terrain", "target/terrain_export"]
            .into_iter()
            .map(str::to_string),
    )
    .expect("terrain export args should parse");

    match action {
        CliAction::ExportTerrain { output_dir } => {
            assert_eq!(output_dir, PathBuf::from("target/terrain_export"));
        }
        _ => panic!("expected terrain export action"),
    }
}

#[test]
fn parse_cli_args_accepts_visual_content_export() {
    let action = parse_cli_args(
        ["--export-visual-content", "target/visual_content_export"]
            .into_iter()
            .map(str::to_string),
    )
    .expect("visual content export args should parse");

    match action {
        CliAction::ExportVisualContent { output_dir } => {
            assert_eq!(output_dir, PathBuf::from("target/visual_content_export"));
        }
        _ => panic!("expected visual content export action"),
    }
}

#[test]
fn parse_cli_args_accepts_wind_visual_export() {
    let action = parse_cli_args(
        ["--export-wind-visuals", "target/wind_visual_export"]
            .into_iter()
            .map(str::to_string),
    )
    .expect("wind visual export args should parse");

    match action {
        CliAction::ExportWindVisuals { output_dir } => {
            assert_eq!(output_dir, PathBuf::from("target/wind_visual_export"));
        }
        _ => panic!("expected wind visual export action"),
    }
}

#[test]
fn parse_cli_args_rejects_eval_and_terrain_export_together() {
    let error = parse_cli_args(
        [
            "--eval",
            "baseline_route",
            "--export-terrain",
            "target/terrain_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("eval and terrain export should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_eval_and_visual_content_export_together() {
    let error = parse_cli_args(
        [
            "--eval",
            "baseline_route",
            "--export-visual-content",
            "target/visual_content_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("eval and visual content export should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_eval_and_wind_visual_export_together() {
    let error = parse_cli_args(
        [
            "--eval",
            "baseline_route",
            "--export-wind-visuals",
            "target/wind_visual_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("eval and wind visual export should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_eval_and_play_profile_together() {
    let error = parse_cli_args(
        [
            "--eval",
            "baseline_route",
            "--play-profile",
            "target/eval/manual_play/profile.json",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("eval and play profile should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_export_and_play_profile_together() {
    let error = parse_cli_args(
        [
            "--export-terrain",
            "target/terrain_export",
            "--play-profile",
            "target/eval/manual_play/profile.json",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("export and play profile should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_both_export_paths_together() {
    let error = parse_cli_args(
        [
            "--export-terrain",
            "target/terrain_export",
            "--export-visual-content",
            "target/visual_content_export",
            "--export-wind-visuals",
            "target/wind_visual_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("export paths should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_play_and_debug_together() {
    let error = parse_cli_args(["--play", "--debug"].into_iter().map(str::to_string))
        .expect_err("play and debug should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_run_mode_and_export_together() {
    let error = parse_cli_args(
        ["--debug", "--export-terrain", "target/terrain_export"]
            .into_iter()
            .map(str::to_string),
    )
    .expect_err("run mode flags and export should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn clean_play_mode_starts_world_without_debug_readout_or_gizmos() {
    let mut app = setup_headless_runtime_app(RunMode::Play, false, true);
    let world = app.world_mut();
    let (route_island_count, expected_pond_surface_count) = {
        let route = world.resource::<SkyRoute>();
        (
            route.islands().len(),
            route
                .islands()
                .iter()
                .copied()
                .enumerate()
                .flat_map(|(index, island)| island_water_visual_specs(index, island))
                .filter(|feature| feature.kind == IslandWaterVisualKind::PondSurface)
                .count(),
        )
    };
    let content_metrics = *world.resource::<crate::content_diagnostics::IslandContentDiagnostics>();

    assert_eq!(component_count::<DebugReadout>(world), 0);
    assert_eq!(component_count::<GameHudRoot>(world), 1);
    assert_eq!(component_count::<GameMenuOverlay>(world), 0);
    assert!(!world.resource::<DebugVisuals>().enabled);
    assert_eq!(component_count::<Player>(world), 1);
    assert_eq!(component_count::<Camera3d>(world), 1);
    assert!(
        world
            .resource::<IslandVisualCatalog>()
            .prebuilt_mesh_count()
            > route_island_count
    );
    assert!(content_metrics.generated_launch_beacon_count >= 1);
    assert!(content_metrics.generated_route_cairn_count >= route_island_count);
    assert!(content_metrics.generated_ground_cover_patch_count >= 2_400);
    assert!(content_metrics.generated_tree_trunk_count >= 160);
    assert!(content_metrics.generated_tree_canopy_count >= 160);
    assert!(content_metrics.generated_rock_count >= 230);
    assert!(content_metrics.generated_ruin_cluster_count >= 6);
    assert!(
        expected_pond_surface_count >= 5,
        "expected at least five generated pond surfaces"
    );
    assert_eq!(
        content_metrics.generated_pond_surface_count,
        expected_pond_surface_count
    );
    assert!(component_count::<WindField>(world) > 0);
    assert!(component_count::<LiftField>(world) > 0);
    assert!(component_count::<UpdraftGuide>(world) > 0);
    assert!(component_count::<UpdraftRibbon>(world) > 0);
    assert!(component_count::<CrosswindGuide>(world) > 0);
}

#[test]
fn debug_mode_starts_with_debug_readout_and_toggleable_gizmos() {
    let mut app = setup_headless_runtime_app(RunMode::Debug, false, true);
    let world = app.world_mut();

    assert_eq!(component_count::<DebugReadout>(world), 1);
    assert_eq!(component_count::<GameHudRoot>(world), 1);
    assert!(world.resource::<DebugVisuals>().enabled);
    assert_eq!(component_count::<Player>(world), 1);
    assert_eq!(component_count::<Camera3d>(world), 1);
}

#[test]
fn screenshot_eval_suppresses_debug_gizmos_even_in_debug_mode() {
    let debug_visuals = DebugVisuals::for_run_mode(RunMode::Debug, true);

    assert!(!debug_visuals.enabled);
}

#[test]
fn eval_and_profile_style_runs_can_suppress_game_ui() {
    let mut app = setup_headless_runtime_app(RunMode::Play, false, false);
    let world = app.world_mut();

    assert_eq!(component_count::<GameHudRoot>(world), 0);
    assert_eq!(component_count::<GameMenuOverlay>(world), 0);
}

#[test]
fn clean_play_ui_keeps_only_the_compact_hud_resident() {
    let mut app_with_ui = setup_headless_runtime_app(RunMode::Play, false, true);
    let mut app_without_ui = setup_headless_runtime_app(RunMode::Play, false, false);

    assert_eq!(
        live_entity_count(app_with_ui.world_mut()) - live_entity_count(app_without_ui.world_mut()),
        3
    );
}

fn setup_headless_runtime_app(
    run_mode: RunMode,
    suppress_debug_visuals_for_screenshot: bool,
    game_ui_enabled: bool,
) -> App {
    let mut app = App::new();
    app.insert_resource(run_mode)
        .insert_resource(DebugVisuals::for_run_mode(
            run_mode,
            suppress_debug_visuals_for_screenshot,
        ))
        .insert_resource(SkyRoute::default())
        .insert_resource(GameUiState::new(game_ui_enabled))
        .add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Assets<SurfaceMaterial>>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<ScatteringMedium>>()
        .init_asset::<Gltf>()
        .init_asset::<Scene>()
        .add_systems(
            Startup,
            (setup, apply_authored_island_material_parity).chain(),
        );
    app.update();
    app
}

fn setup_headless_gallery_runtime_app(island_index: usize) -> (App, PathBuf) {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-gallery-material-parity-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    let scenario = scenario_named(ISLAND_HERO_GALLERY).expect("gallery scenario should exist");
    let mut run = EvalRun::new(EvalOptions {
        scenario,
        output_dir: output_dir.clone(),
        capture_screenshot: false,
        visible_window: false,
    })
    .expect("gallery run should initialize");
    let frames_per_view = run
        .scenario
        .island_hero_gallery_timing()
        .expect("gallery timing should exist")
        .frames_per_view;
    run.frame = island_index as u32
        * nau_engine::world::ISLAND_REVIEW_VIEWS_PER_ISLAND as u32
        * frames_per_view;

    let mut app = App::new();
    app.insert_resource(RunMode::Play)
        .insert_resource(DebugVisuals::for_run_mode(RunMode::Play, false))
        .insert_resource(SkyRoute::default())
        .insert_resource(GameUiState::new(false))
        .insert_resource(run)
        .add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Assets<SurfaceMaterial>>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<ScatteringMedium>>()
        .init_asset::<Gltf>()
        .init_asset::<Scene>()
        .add_systems(
            Startup,
            (setup, apply_authored_island_material_parity).chain(),
        );
    app.update();
    (app, output_dir)
}

fn material_texture_contains_color(
    world: &World,
    handle: &Handle<StandardMaterial>,
    expected: [u8; 4],
) -> bool {
    let texture = world
        .resource::<Assets<StandardMaterial>>()
        .get(handle)
        .and_then(|material| material.base_color_texture.clone())
        .expect("runtime detail material should have a base-color texture");
    world
        .resource::<Assets<Image>>()
        .get(&texture)
        .and_then(|image| image.data.as_deref())
        .is_some_and(|data| data.chunks_exact(4).any(|pixel| pixel == expected))
}

fn component_count<T: Component>(world: &mut World) -> usize {
    let mut query = world.query_filtered::<Entity, With<T>>();
    query.iter(world).count()
}

fn live_entity_count(world: &mut World) -> usize {
    let mut query = world.query::<Entity>();
    query.iter(world).count()
}

#[test]
fn play_window_uses_display_synchronized_presentation() {
    assert_eq!(primary_window(None).present_mode, PresentMode::AutoVsync);
}

#[test]
fn metric_only_eval_window_is_hidden_and_unfocused() {
    let scenario = scenario_named("baseline_route").expect("baseline scenario should exist");
    let options = EvalOptions {
        scenario,
        output_dir: PathBuf::from("target/eval/test_hidden_window"),
        capture_screenshot: false,
        visible_window: false,
    };

    let window = primary_window(Some(&options));

    assert!(!window.visible);
    assert!(!window.focused);
    assert!(!window.transparent);
    assert_eq!(window.composite_alpha_mode, CompositeAlphaMode::Opaque);
}

#[test]
fn metric_only_profile_eval_window_can_remain_visible_and_focused() {
    let scenario = scenario_named("baseline_route").expect("baseline scenario should exist");
    let options = EvalOptions {
        scenario,
        output_dir: PathBuf::from("target/eval/test_visible_metric_window"),
        capture_screenshot: false,
        visible_window: true,
    };

    let window = primary_window(Some(&options));

    assert!(window.visible);
    assert!(window.focused);
    assert!(!window.transparent);
    assert_eq!(window.composite_alpha_mode, CompositeAlphaMode::Opaque);
}

#[test]
fn screenshot_eval_window_remains_visible_for_capture() {
    let scenario = scenario_named("baseline_route").expect("baseline scenario should exist");
    let options = EvalOptions {
        scenario,
        output_dir: PathBuf::from("target/eval/test_visible_window"),
        capture_screenshot: true,
        visible_window: false,
    };

    let window = primary_window(Some(&options));

    assert!(window.visible);
    assert!(window.focused);
    assert!(!window.transparent);
    assert_eq!(window.composite_alpha_mode, CompositeAlphaMode::Opaque);
}

#[test]
fn terrain_export_writes_manifest_meshes_and_weight_sidecars() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-terrain-export-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    remove_existing_dir(&output_dir).expect("stale terrain export dir should be removable");

    let report = export_terrain_inspection(&output_dir).expect("terrain export should succeed");
    let manifest = fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
    let launch_terrain = output_dir.join("islands/00_launch_mesa_terrain.obj");
    let launch_impostor = output_dir.join("islands/00_launch_mesa_impostor.obj");
    let launch_weights = output_dir.join("islands/00_launch_mesa_terrain_material_weights.csv");
    let weights =
        fs::read_to_string(&launch_weights).expect("material weights csv should be readable");

    assert_eq!(report.island_count, SkyRoute::default().islands().len());
    assert_eq!(report.mesh_count, report.island_count * 4);
    assert!(report.total_vertex_count > report.island_count * (2305 + 140));
    assert!(report.total_triangle_count > report.island_count * 4000);
    assert!(report.min_terrain_mesh_vertices >= 2305);
    assert!(report.min_terrain_color_bands >= ISLAND_TERRAIN_COLOR_BANDS);
    assert!(report.min_terrain_material_weight_bands >= ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS);
    assert!(report.min_terrain_material_channels >= ISLAND_TERRAIN_MATERIAL_CHANNELS);
    assert!(report.min_terrain_material_regions >= ISLAND_TERRAIN_MATERIAL_REGIONS);
    assert!(report.min_terrain_height_bands >= ISLAND_TERRAIN_HEIGHT_BANDS);
    assert!(report.min_terrain_normal_slope_bands >= ISLAND_TERRAIN_NORMAL_SLOPE_BANDS);
    assert!(report.min_terrain_texture_detail_bands >= ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS);
    assert!(report.min_terrain_texture_edge_promille >= ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE);
    assert!(
        report.min_terrain_texture_near_detail_energy_promille
            >= ISLAND_TERRAIN_TEXTURE_NEAR_DETAIL_ENERGY_PROMILLE
    );
    assert!(
        report.min_terrain_texture_mid_detail_energy_promille
            >= ISLAND_TERRAIN_TEXTURE_MID_DETAIL_ENERGY_PROMILLE
    );
    assert!(
        report.min_terrain_texture_macro_detail_energy_promille
            >= ISLAND_TERRAIN_TEXTURE_MACRO_DETAIL_ENERGY_PROMILLE
    );
    assert!(
        report.min_terrain_texture_near_to_mid_ratio_promille
            >= ISLAND_TERRAIN_TEXTURE_MIN_NEAR_TO_MID_RATIO_PROMILLE
    );
    assert!(
        report.max_terrain_texture_near_to_mid_ratio_promille
            <= ISLAND_TERRAIN_TEXTURE_MAX_NEAR_TO_MID_RATIO_PROMILLE
    );
    assert!(
        report.max_terrain_texture_isolated_edge_promille
            <= ISLAND_TERRAIN_TEXTURE_MAX_ISOLATED_EDGE_PROMILLE
    );
    assert!(report.min_terrain_relief_range_m >= 0.8);
    assert!(report.min_cliff_color_bands >= ISLAND_CLIFF_STRATA_BANDS / 2);
    assert!(report.min_impostor_mesh_vertices >= EXPECTED_ISLAND_IMPOSTOR_MESH_VERTICES);
    assert!(report.min_impostor_color_bands >= ISLAND_IMPOSTOR_COLOR_BANDS);
    assert!(launch_terrain.exists());
    assert!(launch_impostor.exists());
    assert!(launch_weights.exists());
    assert!(weights.starts_with("vertex,lush_highland,exposed_edge\n"));
    assert!(weights.lines().count() > 2000);
    assert!(manifest.contains("\"schema\": \"nau_terrain_export.v1\""));
    assert!(manifest.contains(
        "\"material_weights_csv\": \"islands/00_launch_mesa_terrain_material_weights.csv\""
    ));
    assert!(manifest.contains(&format!(
        "\"terrain_material_weight_bands\": {}",
        report.min_terrain_material_weight_bands
    )));
    assert!(manifest.contains(&format!(
        "\"terrain_material_regions\": {}",
        report.min_terrain_material_regions
    )));
    assert!(manifest.contains("\"terrain_height_bands\""));
    assert!(manifest.contains("\"terrain_normal_slope_bands\""));
    assert!(manifest.contains(&format!(
        "\"terrain_texture_detail_bands\": {}",
        report.min_terrain_texture_detail_bands
    )));
    assert!(manifest.contains(&format!(
        "\"terrain_texture_edge_promille\": {}",
        report.min_terrain_texture_edge_promille
    )));
    assert!(manifest.contains(&format!(
        "\"impostor_mesh_vertices\": {}",
        report.min_impostor_mesh_vertices
    )));
    assert!(manifest.contains(&format!(
        "\"impostor_color_bands\": {}",
        report.min_impostor_color_bands
    )));
    assert!(manifest.contains("\"impostor\": {\"obj\": \"islands/00_launch_mesa_impostor.obj\""));

    remove_existing_dir(&output_dir).expect("terrain export test dir should be removable");
}

#[test]
fn terrain_export_includes_great_sky_plateau_scale_and_region_evidence() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-terrain-export-plateau-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    remove_existing_dir(&output_dir).expect("stale terrain export dir should be removable");

    let report = export_terrain_inspection(&output_dir).expect("terrain export should succeed");
    let manifest = fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
    let plateau = report
        .islands
        .iter()
        .find(|island| island.island.name == "great sky plateau")
        .expect("terrain export should include the Great Sky Plateau island");
    let terrain_obj = output_dir.join(&plateau.terrain.obj_path);
    let regions = [
        IslandPlateauRegion::MeadowPlateau,
        IslandPlateauRegion::CliffRim,
        IslandPlateauRegion::HighShelf,
        IslandPlateauRegion::LowBasin,
        IslandPlateauRegion::BrokenEdge,
        IslandPlateauRegion::UnderhangEntry,
    ];

    assert_eq!(
        plateau.island.terrain_archetype,
        IslandTerrainArchetype::SkyPlateau
    );
    assert!(plateau.island.is_great_plateau_anchor());
    assert!(plateau.island.base_area_m2() >= 32_000.0);
    assert!(plateau.island.longest_span_m() >= 450.0);
    assert!(plateau.terrain.relief_range_m >= 0.8);
    assert!(plateau.terrain.height_bands >= ISLAND_TERRAIN_HEIGHT_BANDS);
    assert!(plateau.terrain.normal_slope_bands >= ISLAND_TERRAIN_NORMAL_SLOPE_BANDS);
    assert!(plateau.slug.contains("great_sky_plateau"));
    assert!(terrain_obj.exists());
    assert!(manifest.contains("\"name\": \"great sky plateau\""));
    assert!(manifest.contains("\"terrain_archetype\": \"sky_plateau\""));
    assert!(manifest.contains("great_sky_plateau_terrain.obj"));

    for region in regions {
        assert!(
            plateau.island.plateau_region_position(region).is_some(),
            "{region:?} should export from a playable plateau surface"
        );
    }

    remove_existing_dir(&output_dir).expect("terrain export test dir should be removable");
}

#[test]
fn great_sky_plateau_water_specs_span_basin_cliffs_and_falls() {
    let route = SkyRoute::default();
    let (plateau_index, plateau) = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, island)| island.is_great_plateau_anchor())
        .expect("route should include great sky plateau");
    let water_features = island_water_visual_specs(plateau_index, plateau);

    assert_eq!(
        water_features
            .iter()
            .filter(|feature| feature.kind == IslandWaterVisualKind::PondSurface)
            .count(),
        0
    );
    assert_eq!(
        water_features
            .iter()
            .filter(|feature| feature.kind == IslandWaterVisualKind::PlateauLakeSurface)
            .count(),
        2
    );
    assert_eq!(
        water_features
            .iter()
            .filter(|feature| feature.kind == IslandWaterVisualKind::PlateauWaterfallRibbon)
            .count(),
        2
    );
    assert_eq!(
        water_features
            .iter()
            .filter(|feature| feature.kind == IslandWaterVisualKind::PlateauWaterfallMist)
            .count(),
        2
    );

    let low_basin = plateau
        .plateau_region_position(IslandPlateauRegion::LowBasin)
        .expect("low basin should be playable");
    let high_shelf = plateau
        .plateau_region_position(IslandPlateauRegion::HighShelf)
        .expect("high shelf should be playable");
    let lake = water_features
        .iter()
        .find(|feature| feature.label == "low basin lake")
        .expect("plateau should place a low basin lake");
    let high_pool = water_features
        .iter()
        .find(|feature| feature.label == "high shelf pool")
        .expect("plateau should place a high shelf pool");
    assert!(lake.translation.xz().distance(low_basin.xz()) < 0.2);
    assert!(high_pool.translation.xz().distance(high_shelf.xz()) < 0.2);
    assert!(high_pool.translation.y > lake.translation.y);

    let rim = plateau
        .plateau_region_position(IslandPlateauRegion::CliffRim)
        .expect("rim should be playable");
    let waterfall = water_features
        .iter()
        .find(|feature| feature.label == "north rim waterfall")
        .expect("plateau should place a rim waterfall");
    let mist = water_features
        .iter()
        .find(|feature| feature.label == "north rim waterfall mist")
        .expect("plateau should place waterfall mist below the rim");
    assert!(rim.y - waterfall.translation.y >= 25.0);
    assert!(waterfall.translation.y - mist.translation.y >= 25.0);

    let lake_mesh = lake.build_mesh();
    let waterfall_mesh = waterfall.build_mesh();
    let mist_mesh = mist.build_mesh();
    assert!(lake_mesh.count_vertices() > LAKE_SURFACE_SEGMENTS * 3);
    assert!(mesh_y_range(&lake_mesh) >= 0.05);
    assert!(mesh_y_range(&waterfall_mesh) >= 58.0);
    assert!(waterfall_mesh.count_vertices() >= WATERFALL_RIBBON_COLUMNS * WATERFALL_RIBBON_ROWS);
    assert!(mist_mesh.count_vertices() >= WATERFALL_MIST_LOBES * 40);
}

#[test]
fn great_sky_plateau_under_route_visual_specs_mark_cave_and_shelf() {
    let route = SkyRoute::default();
    let (plateau_index, plateau) = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, island)| island.is_great_plateau_anchor())
        .expect("route should include great sky plateau");
    let under_route = plateau
        .under_route_segment()
        .expect("plateau should define an under-route");
    let cave_features = island_under_route_visual_specs(plateau_index, plateau);

    assert_eq!(cave_features.len(), 4);
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::CaveMouthArch)
            .count(),
        2
    );
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::UnderhangShelf)
            .count(),
        1
    );
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::HangingRoots)
            .count(),
        1
    );

    let entry_arch = cave_features
        .iter()
        .find(|feature| feature.label == "underhang entry arch")
        .expect("entry arch should be generated");
    let shelf = cave_features
        .iter()
        .find(|feature| feature.label == "underside glide shelf")
        .expect("glide shelf should be generated");
    let roots = cave_features
        .iter()
        .find(|feature| feature.label == "hanging root curtain")
        .expect("hanging roots should be generated");
    let exit_arch = cave_features
        .iter()
        .find(|feature| feature.label == "updraft skylight exit arch")
        .expect("exit arch should be generated");

    assert!(entry_arch.translation.distance(under_route.entry) < 0.1);
    assert!(exit_arch.translation.distance(under_route.exit) < 0.1);
    assert!(shelf.translation.y < under_route.midpoint.y);
    assert!(roots.translation.y > under_route.midpoint.y);
    assert!(entry_arch.camera_half_extents.x >= under_route.clearance_radius_m);
    assert!(shelf.camera_half_extents.x > entry_arch.camera_half_extents.x);
    assert!(roots.camera_half_extents.y >= under_route.clearance_radius_m * 0.4);

    let arch_mesh = entry_arch.build_mesh();
    let shelf_mesh = shelf.build_mesh();
    let roots_mesh = roots.build_mesh();
    assert!(arch_mesh.count_vertices() >= CAVE_MOUTH_ARCH_STONES * 40);
    assert!(mesh_y_range(&arch_mesh) >= under_route.clearance_radius_m);
    assert_eq!(shelf_mesh.count_vertices(), UNDERHANG_SHELF_SEGMENTS * 2);
    assert!(mesh_y_range(&shelf_mesh) > 3.0);
    assert_eq!(
        roots_mesh.count_vertices(),
        HANGING_ROOT_STRANDS * (HANGING_ROOT_SEGMENTS + 1) * 4
    );
    assert!(mesh_y_range(&roots_mesh) >= under_route.clearance_radius_m * 0.7);
}

#[test]
fn underbridge_cay_under_route_visual_specs_mark_cave_and_shelf() {
    let route = SkyRoute::default();
    let (underbridge_index, underbridge) = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, island)| island.name == "underbridge cay")
        .expect("route should include underbridge cay");
    let under_route = underbridge
        .under_route_segment()
        .expect("underbridge cay should define an under-route");
    let cave_features = island_under_route_visual_specs(underbridge_index, underbridge);

    assert_eq!(cave_features.len(), 4);
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::CaveMouthArch)
            .count(),
        2
    );
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::UnderhangShelf)
            .count(),
        1
    );
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::HangingRoots)
            .count(),
        1
    );

    let entry_arch = cave_features
        .iter()
        .find(|feature| feature.label == "underhang entry arch")
        .expect("entry arch should be generated");
    let shelf = cave_features
        .iter()
        .find(|feature| feature.label == "underside glide shelf")
        .expect("glide shelf should be generated");
    let roots = cave_features
        .iter()
        .find(|feature| feature.label == "hanging root curtain")
        .expect("hanging roots should be generated");
    let exit_arch = cave_features
        .iter()
        .find(|feature| feature.label == "updraft skylight exit arch")
        .expect("exit arch should be generated");

    assert!(entry_arch.translation.distance(under_route.entry) < 0.1);
    assert!(exit_arch.translation.distance(under_route.exit) < 0.1);
    assert!(shelf.translation.y < under_route.midpoint.y);
    assert!(roots.translation.y > under_route.midpoint.y);
    assert!(entry_arch.camera_half_extents.x >= under_route.clearance_radius_m);
    assert!(shelf.camera_half_extents.x > entry_arch.camera_half_extents.x);
    assert!(roots.camera_half_extents.y >= under_route.clearance_radius_m * 0.4);

    let arch_mesh = entry_arch.build_mesh();
    let shelf_mesh = shelf.build_mesh();
    let roots_mesh = roots.build_mesh();
    assert!(arch_mesh.count_vertices() >= CAVE_MOUTH_ARCH_STONES * 40);
    assert!(mesh_y_range(&arch_mesh) >= under_route.clearance_radius_m);
    assert_eq!(shelf_mesh.count_vertices(), UNDERHANG_SHELF_SEGMENTS * 2);
    assert!(mesh_y_range(&shelf_mesh) >= under_route.clearance_radius_m * 0.30);
    assert_eq!(
        roots_mesh.count_vertices(),
        HANGING_ROOT_STRANDS * (HANGING_ROOT_SEGMENTS + 1) * 4
    );
    assert!(mesh_y_range(&roots_mesh) >= under_route.clearance_radius_m * 0.7);
}

#[test]
fn visual_content_export_writes_manifest_meshes_and_shape_metrics() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-visual-content-export-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    remove_existing_dir(&output_dir).expect("stale visual content export dir should be removable");

    let report = export_visual_content_inspection(&output_dir)
        .expect("visual content export should succeed");
    let manifest = fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
    let launch_ground_cover = output_dir.join("visuals/00_launch_mesa_ground_cover.obj");
    let launch_tree_trunk =
        output_dir.join("visuals/00_launch_mesa_launch_tree_broad_canopy_trunk.obj");
    let launch_cloud = output_dir.join("visuals/00_launch_mesa_bank_0.obj");
    let launch_beacon = output_dir.join("visuals/00_launch_mesa_launch_beacon.obj");
    let midpoint_cairn = output_dir.join("visuals/01_midpoint_shelf_route_cairn.obj");
    let landing_pond = output_dir.join("visuals/02_landing_garden_spring_pond.obj");
    let launch_spire = output_dir.join("visuals/00_launch_mesa_obstruction_spire.obj");
    let landing_marker = output_dir.join("visuals/02_landing_garden_landing_garden_marker_0.obj");
    let plateau_roots = output_dir.join("visuals/38_great_sky_plateau_hanging_root_curtain.obj");
    let plateau_arrival_shelf =
        output_dir.join("visuals/38_great_sky_plateau_meadow_landing_shelf.obj");
    let plateau_arrival_ruin =
        output_dir.join("visuals/38_great_sky_plateau_arrival_ruin_marker.obj");
    let plateau_high_shelf_hint =
        output_dir.join("visuals/38_great_sky_plateau_high_shelf_onward_hint.obj");
    let plateau_cave_hint =
        output_dir.join("visuals/38_great_sky_plateau_cave_mouth_onward_hint.obj");
    let north_ruin_spire = output_dir.join("visuals/13_mist_arch_north_ruin_spire.obj");
    let south_ruin_spire = output_dir.join("visuals/14_cloud_gate_south_ruin_spire.obj");
    let waterfall_cliff =
        output_dir.join("visuals/28_cloudfall_meadow_waterfall_cliff_silhouette.obj");
    let cave_arch = output_dir.join("visuals/20_underbridge_cay_cave_mouth_silhouette.obj");
    let ring_garden = output_dir.join("visuals/36_sunspire_garden_ring_garden_silhouette.obj");
    let broken_stair_silhouette =
        output_dir.join("visuals/12_broken_stair_broken_stair_silhouette.obj");
    let high_crown = output_dir.join("visuals/40_upper_crown_high_crown_silhouette.obj");
    let route = SkyRoute::default();
    let island_count = route.islands().len();
    let pond_surface_count = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .flat_map(|(index, island)| island_water_visual_specs(index, island))
        .filter(|feature| feature.kind == IslandWaterVisualKind::PondSurface)
        .count();
    assert!(pond_surface_count >= 5);
    let small_island_count = route
        .islands()
        .iter()
        .filter(|island| {
            matches!(
                island.world_tags.scale_class,
                IslandScaleClass::Tiny | IslandScaleClass::Small
            )
        })
        .count();
    let ground_cover_patch_total = route
        .islands()
        .iter()
        .copied()
        .map(island_detail_budget)
        .map(|budget| budget.ground_cover_patch_count)
        .sum::<usize>();
    let generated_rock_count = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .map(|(index, island)| island_rock_specs(index, island).len())
        .sum::<usize>();
    let ruin_cluster_count = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, island)| !island_ruin_specs(*index, *island).is_empty())
        .count();
    let runtime_ruin_specs = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .flat_map(|(island_index, island)| {
            island_ruin_specs(island_index, island)
                .into_iter()
                .enumerate()
                .map(move |(ruin_index, spec)| (island_index, island.name, ruin_index, spec))
        })
        .collect::<Vec<_>>();
    let ruin_arch_count = runtime_ruin_specs.len();
    let cliff_teeth_count = route
        .islands()
        .iter()
        .filter(|island| {
            matches!(
                island.terrain_archetype,
                IslandTerrainArchetype::StormRavine | IslandTerrainArchetype::StormShard
            )
        })
        .count();
    let garden_ring_count = route
        .islands()
        .iter()
        .filter(|island| {
            matches!(
                island.terrain_archetype,
                IslandTerrainArchetype::GardenBasin
                    | IslandTerrainArchetype::GardenApron
                    | IslandTerrainArchetype::OrchardBasin
                    | IslandTerrainArchetype::OrchardSpur
            )
        })
        .count();
    let lake_basin_count: usize = route
        .islands()
        .iter()
        .enumerate()
        .map(|(index, island)| island_lake_basin_visual_specs(index, *island).len())
        .sum();
    let first_expedition_silhouette_count: usize = route
        .islands()
        .iter()
        .enumerate()
        .map(|(index, island)| first_expedition_silhouette_specs(index, *island).len())
        .sum();
    let artifact_detail_count: usize = route
        .islands()
        .iter()
        .enumerate()
        .map(|(index, island)| island_artifact_visual_specs(index, *island).len())
        .sum();
    let flora_cluster_count: usize = route
        .islands()
        .iter()
        .enumerate()
        .map(|(index, island)| island_flora_visual_specs(index, *island).len())
        .sum();
    let ruin_complex_count: usize = route
        .islands()
        .iter()
        .enumerate()
        .map(|(index, island)| island_ruin_complex_specs(index, *island).len())
        .sum();
    let rock_formation_count: usize = route
        .islands()
        .iter()
        .enumerate()
        .map(|(index, island)| island_rock_formation_specs(index, *island).len())
        .sum();
    let water_detail_count: usize = route
        .islands()
        .iter()
        .enumerate()
        .map(|(index, island)| {
            let water_features = island_water_visual_specs(index, *island);
            island_water_detail_specs(index, *island, &water_features).len()
        })
        .sum();
    let surface_feature_count =
        flora_cluster_count + ruin_complex_count + rock_formation_count + water_detail_count;
    let route_lake_surface_count = route
        .islands()
        .iter()
        .enumerate()
        .flat_map(|(index, island)| island_water_visual_specs(index, *island))
        .filter(|feature| feature.kind == IslandWaterVisualKind::RouteLakeSurface)
        .count();
    let route_waterfall_source_count = route
        .islands()
        .iter()
        .filter(|island| island.world_tags.water_feature == IslandWaterFeature::WaterfallSource)
        .count();
    let route_waterfall_visual_count = route_waterfall_source_count * 2;
    let river_channel_count = route
        .islands()
        .iter()
        .enumerate()
        .flat_map(|(index, island)| island_water_visual_specs(index, *island))
        .filter(|feature| feature.kind == IslandWaterVisualKind::RiverChannel)
        .count();
    let generated_tree_count = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .map(|(index, island)| island_tree_specs(index, island).len())
        .sum::<usize>()
        + 1;
    let weather_veil_count = island_count.div_ceil(2) * 3;
    let route_cairn_count = island_count - 2;
    let hero_landmark_count = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, island)| island_hero_landmark_spec(index, island))
        .count();
    let plateau_extra_water_count = 6;
    let plateau_arrival_landmark_count = 4;
    let under_route_visual_specs = route
        .islands()
        .iter()
        .enumerate()
        .flat_map(|(index, island)| island_under_route_visual_specs(index, *island))
        .collect::<Vec<_>>();
    let under_route_visual_count = under_route_visual_specs.len();
    let under_route_cave_mouth_count = under_route_visual_specs
        .iter()
        .filter(|feature| feature.kind == IslandUnderRouteVisualKind::CaveMouthArch)
        .count();
    let under_route_shelf_count = under_route_visual_specs
        .iter()
        .filter(|feature| feature.kind == IslandUnderRouteVisualKind::UnderhangShelf)
        .count();
    let under_route_root_count = under_route_visual_specs
        .iter()
        .filter(|feature| feature.kind == IslandUnderRouteVisualKind::HangingRoots)
        .count();
    assert_eq!(under_route_visual_count, 8);
    assert_eq!(under_route_cave_mouth_count, 4);
    assert_eq!(under_route_shelf_count, 2);
    assert_eq!(under_route_root_count, 2);
    assert_eq!(first_expedition_silhouette_count, 7);
    assert_eq!(hero_landmark_count, island_count);
    let landmark_count = island_count
        + hero_landmark_count
        + pond_surface_count
        + route_cairn_count
        + 1
        + 4
        + plateau_extra_water_count
        + plateau_arrival_landmark_count
        + under_route_visual_count
        + first_expedition_silhouette_count
        + ruin_arch_count
        + cliff_teeth_count
        + garden_ring_count
        + lake_basin_count
        + route_lake_surface_count
        + route_waterfall_visual_count
        + river_channel_count
        + artifact_detail_count
        + surface_feature_count;

    assert_eq!(report.ground_cover_count, island_count);
    assert_eq!(report.ground_cover_patch_total, ground_cover_patch_total);
    assert_eq!(
        report.ground_cover_blade_total,
        ground_cover_patch_total * GROUND_COVER_BLADES_PER_PATCH
    );
    assert_eq!(report.tree_trunk_count, generated_tree_count);
    assert_eq!(report.tree_canopy_count, generated_tree_count);
    assert_eq!(report.rock_count, generated_rock_count);
    assert!(report.rock_count >= 230);
    assert_eq!(
        report.weather_cloud_count,
        island_count + weather_veil_count
    );
    assert_eq!(report.weather_cloud_bank_count, island_count);
    assert_eq!(report.weather_cloud_veil_count, weather_veil_count);
    assert_eq!(report.landmark_count, landmark_count);
    assert_eq!(report.flora_cluster_count, flora_cluster_count);
    assert_eq!(report.ruin_complex_count, ruin_complex_count);
    assert_eq!(report.rock_formation_count, rock_formation_count);
    assert_eq!(report.water_detail_count, water_detail_count);
    assert!(report.landmark_kind_count >= 28);
    assert_eq!(report.small_island_count, small_island_count);
    assert!(report.small_island_count >= 10);
    let plateau_landmark_count = report
        .landmarks
        .iter()
        .filter(|summary| summary.island_name == "great sky plateau")
        .count();
    assert_eq!(report.plateau_landmark_count, plateau_landmark_count);
    assert_eq!(report.plateau_waterfall_ribbon_count, 2);
    assert_eq!(report.plateau_waterfall_mist_count, 2);
    assert_eq!(
        report.route_waterfall_ribbon_count,
        route_waterfall_source_count
    );
    assert_eq!(
        report.route_waterfall_mist_count,
        route_waterfall_source_count
    );
    assert_eq!(report.route_lake_surface_count, route_lake_surface_count);
    assert_eq!(report.river_channel_count, river_channel_count);
    assert_eq!(report.under_route_visual_count, under_route_visual_count);
    assert_eq!(
        report.under_route_cave_mouth_count,
        under_route_cave_mouth_count
    );
    assert_eq!(report.ruin_cluster_count, ruin_cluster_count);
    assert!(report.ruin_cluster_count >= 6);
    assert_eq!(report.ruin_arch_count, ruin_arch_count);
    let exported_ruin_arches = report
        .landmarks
        .iter()
        .filter(|summary| summary.kind == "ruin_arch")
        .collect::<Vec<_>>();
    assert_eq!(exported_ruin_arches.len(), runtime_ruin_specs.len());
    for ((_, island_name, ruin_index, spec), summary) in
        runtime_ruin_specs.iter().zip(&exported_ruin_arches)
    {
        assert_eq!(summary.island_name, *island_name);
        assert_eq!(summary.label, format!("ruin arch {ruin_index}"));

        let expected_mesh = ruin_arch_mesh(spec.width_m, spec.height_m, spec.depth_m, spec.seed);
        let mut expected_min = Vec3::splat(f32::INFINITY);
        let mut expected_max = Vec3::splat(f32::NEG_INFINITY);
        for position in positions(&expected_mesh) {
            let position = Vec3::from_array(*position);
            expected_min = expected_min.min(position);
            expected_max = expected_max.max(position);
        }
        let expected_span = expected_max - expected_min;

        assert_eq!(summary.mesh.vertex_count, expected_mesh.count_vertices());
        assert_eq!(
            summary.mesh.triangle_count,
            u32_indices(&expected_mesh).len() / 3
        );
        assert_eq!(summary.mesh.horizontal_span_m, expected_span.x);
        assert_eq!(summary.mesh.vertical_span_m, expected_span.y);
        assert_eq!(summary.mesh.depth_span_m, expected_span.z);
        assert!(output_dir.join(&summary.mesh.obj_path).exists());
    }
    assert_eq!(report.route_cairn_count, route_cairn_count);
    assert_eq!(report.launch_beacon_count, 1);
    assert_eq!(report.landing_garden_marker_count, 4);
    assert_eq!(report.pond_surface_count, pond_surface_count);
    assert_eq!(report.obstruction_spire_count, island_count);
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_arrival_shelf")
            .count(),
        1
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_arrival_ruin")
            .count(),
        1
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_onward_hint")
            .count(),
        2
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind.starts_with("first_expedition_"))
            .count(),
        first_expedition_silhouette_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_lake_surface")
            .count(),
        2
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_waterfall_ribbon")
            .count(),
        2
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_waterfall_mist")
            .count(),
        2
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "route_waterfall_ribbon")
            .count(),
        route_waterfall_source_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "route_waterfall_mist")
            .count(),
        route_waterfall_source_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "route_lake_surface")
            .count(),
        route_lake_surface_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "under_route_cave_mouth")
            .count(),
        under_route_cave_mouth_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "under_route_hanging_shelf")
            .count(),
        under_route_shelf_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "under_route_hanging_roots")
            .count(),
        under_route_root_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "ruin_arch")
            .count(),
        ruin_arch_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "cliff_teeth")
            .count(),
        cliff_teeth_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "garden_ring")
            .count(),
        garden_ring_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "lake_basin")
            .count(),
        lake_basin_count
    );
    assert_eq!(
        report.mesh_count,
        report.ground_cover_count
            + report.tree_trunk_count * 2
            + report.rock_count
            + report.weather_cloud_count
            + report.landmark_count
    );
    assert!(report.total_vertex_count > 70_000);
    assert!(report.total_triangle_count > 75_000);
    assert!(report.min_ground_cover_mesh_vertices >= 720);
    assert!(report.min_ground_cover_patch_count >= 24);
    assert!(report.min_ground_cover_blade_count >= 120);
    assert!(report.min_ground_cover_blade_height_range_m >= 0.65);
    assert!(report.min_tree_trunk_mesh_vertices >= 190);
    assert!(report.min_tree_trunk_taper_ratio >= 1.35);
    assert!(report.min_tree_branch_reach_ratio >= 1.8);
    assert!(report.min_tree_branch_count >= 4);
    assert!(report.min_tree_root_flare_count >= 5);
    assert!(report.min_tree_trunk_ring_count >= 5);
    assert!(report.tree_trunk_height_range_m >= 1.5);
    assert!(report.min_tree_canopy_mesh_vertices >= 450);
    assert!(report.min_tree_canopy_lobe_count >= 6);
    assert!(report.min_tree_canopy_detail_card_count >= 18);
    assert!(report.min_tree_canopy_vertical_to_horizontal_ratio >= 0.45);
    assert!(report.tree_canopy_radius_range_m >= 0.35);
    assert!(report.min_rock_mesh_vertices >= 70);
    assert!(report.min_rock_vertical_span_m > 0.25);
    assert!(report.min_weather_cloud_mesh_vertices >= 2500);
    assert!(report.min_weather_cloud_lobe_count >= 12);
    assert!(report.min_weather_cloud_wisp_card_count >= 60);
    assert!(report.min_weather_cloud_filament_ribbon_detail_count >= 48);
    assert!(report.min_weather_cloud_bank_depth_m >= 6.4);
    assert!(report.min_weather_cloud_bank_lobe_count >= 22);
    assert!(report.min_weather_cloud_scaled_depth_span_m >= 14.0);
    assert!(report.min_route_cairn_mesh_vertices >= 240);
    assert!(report.min_route_cairn_vertical_span_m >= 3.0);
    assert!(report.min_launch_beacon_mesh_vertices >= 300);
    assert!(report.min_launch_beacon_vertical_span_m >= 2.8);
    assert!(report.min_landing_garden_marker_mesh_vertices >= 39);
    assert!(report.min_landing_garden_marker_vertical_span_m >= 0.12);
    assert!(report.min_pond_surface_mesh_vertices >= 65);
    assert!(report.min_pond_surface_vertical_span_m >= 0.015);
    assert!(report.plateau_landmark_vertex_total >= 2_500);
    assert!(report.max_plateau_landmark_mesh_vertices >= 600);
    assert!(report.min_plateau_waterfall_vertical_span_m >= 58.0);
    assert!(report.min_route_waterfall_vertical_span_m >= 24.0);
    assert!(report.min_route_lake_surface_horizontal_span_m >= 18.0);
    assert!(report.min_under_route_visual_vertical_span_m >= 4.0);
    assert!(report.min_ruin_arch_mesh_vertices >= 500);
    assert!(report.min_ruin_arch_vertical_span_m >= 4.5);
    assert!(report.min_ruin_arch_radius_band_count >= 8);
    assert!(report.min_ruin_arch_normal_slope_band_count >= 5);
    assert!(report.min_obstruction_spire_mesh_vertices >= 300);
    assert!(report.min_obstruction_spire_triangle_count >= 500);
    assert!(report.min_obstruction_spire_vertical_span_m >= 3.0);
    assert!(report.min_obstruction_spire_height_band_count >= 6);
    assert!(report.min_obstruction_spire_radius_band_count >= 5);
    assert!(report.min_obstruction_spire_normal_slope_band_count >= 5);
    assert_eq!(report.terrain_biome_palette_count, island_count);
    assert_eq!(report.foliage_palette_count, island_count);
    assert_eq!(report.stone_palette_count, island_count);
    assert!(launch_ground_cover.exists());
    assert!(launch_tree_trunk.exists());
    assert!(launch_cloud.exists());
    assert!(launch_beacon.exists());
    assert!(midpoint_cairn.exists());
    assert!(landing_pond.exists());
    assert!(launch_spire.exists());
    assert!(landing_marker.exists());
    assert!(plateau_roots.exists());
    assert!(plateau_arrival_shelf.exists());
    assert!(plateau_arrival_ruin.exists());
    assert!(plateau_high_shelf_hint.exists());
    assert!(plateau_cave_hint.exists());
    assert!(north_ruin_spire.exists());
    assert!(south_ruin_spire.exists());
    assert!(waterfall_cliff.exists());
    assert!(cave_arch.exists());
    assert!(ring_garden.exists());
    assert!(broken_stair_silhouette.exists());
    assert!(high_crown.exists());
    let low_basin_lake = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "low basin lake"
        })
        .expect("great sky plateau should export a low basin lake");
    let waterfall = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.kind == "plateau_waterfall_ribbon"
        })
        .expect("great sky plateau should export waterfall ribbons");
    let mist = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.kind == "plateau_waterfall_mist"
        })
        .expect("great sky plateau should export waterfall mist");
    let route_waterfall = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "cloudfall meadow" && summary.kind == "route_waterfall_ribbon"
        })
        .expect("waterfall-source islands should export route waterfall ribbons");
    let route_mist = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "cloudfall meadow" && summary.kind == "route_waterfall_mist"
        })
        .expect("waterfall-source islands should export route waterfall mist");
    let route_lake = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "sapphire basin" && summary.kind == "route_lake_surface"
        })
        .expect("lake-basin islands should export route lake surfaces");
    let bluevault_basin = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "bluevault basin" && summary.label == "route lake basin"
        })
        .expect("bluevault basin should export a terrain lake basin rim");
    let cave_arch = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "underhang entry arch"
        })
        .expect("great sky plateau should export an underhang entry arch");
    let underhang_shelf = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "underside glide shelf"
        })
        .expect("great sky plateau should export an underside glide shelf");
    let hanging_roots = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "hanging root curtain"
        })
        .expect("great sky plateau should export hanging roots");
    let arrival_shelf = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "meadow landing shelf"
        })
        .expect("great sky plateau should export a broad meadow landing shelf");
    let arrival_ruin = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "arrival ruin marker"
        })
        .expect("great sky plateau should export an arrival ruin marker");
    let high_shelf_hint = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "high shelf onward hint"
        })
        .expect("great sky plateau should export a high shelf onward hint");
    let cave_hint = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "cave mouth onward hint"
        })
        .expect("great sky plateau should export a cave mouth onward hint");
    let ruin_arch = report
        .landmarks
        .iter()
        .find(|summary| summary.kind == "ruin_arch")
        .expect("ruin-tagged islands should export stacked stone arches");
    let cliff_teeth = report
        .landmarks
        .iter()
        .find(|summary| summary.kind == "cliff_teeth")
        .expect("storm islands should export jagged cliff teeth");
    let garden_ring = report
        .landmarks
        .iter()
        .find(|summary| summary.kind == "garden_ring")
        .expect("garden and orchard islands should export organic garden rings");
    let lake_basin = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "low basin lake basin"
        })
        .expect("great sky plateau should export a terrain lake basin rim");

    assert!(low_basin_lake.mesh.horizontal_span_m >= 100.0);
    assert!(low_basin_lake.mesh.depth_span_m >= 45.0);
    assert!(waterfall.mesh.vertical_span_m >= 58.0);
    assert!(waterfall.normal_slope_band_count >= 4);
    assert!(mist.mesh.horizontal_span_m >= 20.0);
    assert!(route_waterfall.mesh.vertical_span_m >= 24.0);
    assert!(route_waterfall.normal_slope_band_count >= 4);
    assert!(route_mist.mesh.vertical_span_m >= 1.8);
    assert!(route_lake.mesh.horizontal_span_m >= 18.0);
    assert!(route_lake.mesh.depth_span_m >= 9.0);
    assert!(bluevault_basin.mesh.horizontal_span_m >= 32.0);
    assert!(bluevault_basin.normal_slope_band_count >= 4);
    assert!(cave_arch.mesh.horizontal_span_m >= 20.0);
    assert!(cave_arch.mesh.vertical_span_m >= 14.0);
    assert!(underhang_shelf.mesh.horizontal_span_m >= 45.0);
    assert!(underhang_shelf.mesh.depth_span_m >= 24.0);
    assert!(hanging_roots.mesh.vertex_count >= 350);
    assert!(hanging_roots.mesh.vertical_span_m >= 8.0);
    assert!(hanging_roots.mesh.horizontal_span_m >= 20.0);
    assert!(arrival_shelf.mesh.horizontal_span_m >= 54.0);
    assert!(arrival_shelf.mesh.depth_span_m >= 54.0);
    assert!(arrival_shelf.mesh.vertical_span_m >= 0.35);
    assert!(arrival_ruin.mesh.horizontal_span_m >= 20.0);
    assert!(arrival_ruin.mesh.vertical_span_m >= 17.0);
    assert!(arrival_ruin.mesh.depth_span_m >= 4.5);
    assert!(arrival_ruin.normal_slope_band_count >= 5);
    assert!(high_shelf_hint.mesh.vertical_span_m >= 5.8);
    assert!(cave_hint.mesh.vertical_span_m >= 6.2);
    assert!(high_shelf_hint.radius_band_count >= 6);
    assert!(cave_hint.radius_band_count >= 6);
    assert!(ruin_arch.mesh.vertex_count >= 500);
    assert!(ruin_arch.mesh.vertical_span_m >= 4.5);
    assert!(ruin_arch.radius_band_count >= 8);
    assert!(
        cliff_teeth.mesh.vertex_count >= CLIFF_TOOTH_COUNT * CLIFF_TOOTH_TRIANGLES_PER_TOOTH * 3
    );
    assert!(cliff_teeth.mesh.vertical_span_m >= 4.0);
    assert!(cliff_teeth.mesh.horizontal_span_m >= 10.0);
    assert!(cliff_teeth.normal_slope_band_count >= 4);
    assert!(garden_ring.mesh.vertex_count >= (GARDEN_RING_SEGMENTS + 1) * GARDEN_RING_BANDS);
    assert!(garden_ring.mesh.horizontal_span_m >= 5.0);
    assert!(garden_ring.mesh.depth_span_m >= 5.0);
    assert!(garden_ring.mesh.vertical_span_m >= 0.16);
    assert!(garden_ring.normal_slope_band_count >= 3);
    assert!(lake_basin.mesh.vertex_count >= (LAKE_BASIN_RIM_SEGMENTS + 1) * LAKE_BASIN_RIM_BANDS);
    assert!(lake_basin.mesh.horizontal_span_m >= 120.0);
    assert!(lake_basin.mesh.depth_span_m >= 65.0);
    assert!(lake_basin.mesh.vertical_span_m >= 1.0);
    assert!(lake_basin.normal_slope_band_count >= 4);
    assert!(output_dir.join(&low_basin_lake.mesh.obj_path).exists());
    assert!(output_dir.join(&waterfall.mesh.obj_path).exists());
    assert!(output_dir.join(&mist.mesh.obj_path).exists());
    assert!(output_dir.join(&route_waterfall.mesh.obj_path).exists());
    assert!(output_dir.join(&route_mist.mesh.obj_path).exists());
    assert!(output_dir.join(&route_lake.mesh.obj_path).exists());
    assert!(output_dir.join(&bluevault_basin.mesh.obj_path).exists());
    assert!(output_dir.join(&cave_arch.mesh.obj_path).exists());
    assert!(output_dir.join(&underhang_shelf.mesh.obj_path).exists());
    assert!(output_dir.join(&hanging_roots.mesh.obj_path).exists());
    assert!(output_dir.join(&arrival_shelf.mesh.obj_path).exists());
    assert!(output_dir.join(&arrival_ruin.mesh.obj_path).exists());
    assert!(output_dir.join(&high_shelf_hint.mesh.obj_path).exists());
    assert!(output_dir.join(&cave_hint.mesh.obj_path).exists());
    assert!(output_dir.join(&ruin_arch.mesh.obj_path).exists());
    assert!(output_dir.join(&cliff_teeth.mesh.obj_path).exists());
    assert!(output_dir.join(&garden_ring.mesh.obj_path).exists());
    assert!(output_dir.join(&lake_basin.mesh.obj_path).exists());
    assert!(manifest.contains("\"schema\": \"nau_visual_content_export.v2\""));
    assert!(manifest.contains("\"ground_cover_blade_height_range_m\""));
    assert!(manifest.contains("\"tree_branch_reach_ratio\""));
    assert!(manifest.contains("\"tree_root_flare_count\": 5"));
    assert!(manifest.contains("\"tree_trunk_ring_count\": 5"));
    assert!(manifest.contains("\"tree_trunk_height_range_m\""));
    assert!(manifest.contains("\"surface_feature_family\": \"flora_cluster\""));
    assert!(manifest.contains("\"surface_feature_family\": \"ruin_complex\""));
    assert!(manifest.contains("\"surface_feature_family\": \"rock_formation\""));
    assert!(manifest.contains("\"surface_feature_family\": \"water_detail\""));
    assert!(manifest.contains("\"tree_canopy_radius_range_m\""));
    assert!(manifest.contains(&format!(
        "\"weather_cloud_veil_count\": {weather_veil_count}"
    )));
    assert!(manifest.contains("\"weather_cloud_scaled_depth_span_m\""));
    assert!(manifest.contains("\"weather_cloud_wisp_card_count\""));
    assert!(manifest.contains("\"weather_cloud_filament_ribbon_detail_count\""));
    assert!(manifest.contains(&format!("\"landmark_count\": {landmark_count}")));
    assert!(manifest.contains("\"landmark_kind_count\""));
    assert!(manifest.contains(&format!("\"small_island_count\": {small_island_count}")));
    assert!(manifest.contains(&format!(
        "\"plateau_landmark_count\": {plateau_landmark_count}"
    )));
    assert!(manifest.contains("\"plateau_waterfall_ribbon_count\": 2"));
    assert!(manifest.contains("\"plateau_waterfall_mist_count\": 2"));
    assert!(manifest.contains(&format!(
        "\"route_waterfall_ribbon_count\": {route_waterfall_source_count}"
    )));
    assert!(manifest.contains(&format!(
        "\"route_waterfall_mist_count\": {route_waterfall_source_count}"
    )));
    assert!(manifest.contains(&format!(
        "\"route_lake_surface_count\": {route_lake_surface_count}"
    )));
    assert!(manifest.contains(&format!("\"river_channel_count\": {river_channel_count}")));
    assert!(manifest.contains(&format!(
        "\"under_route_visual_count\": {under_route_visual_count}"
    )));
    assert!(manifest.contains(&format!(
        "\"under_route_cave_mouth_count\": {under_route_cave_mouth_count}"
    )));
    assert!(manifest.contains(&format!("\"ruin_cluster_count\": {ruin_cluster_count}")));
    assert!(manifest.contains(&format!("\"ruin_arch_count\": {ruin_arch_count}")));
    assert!(manifest.contains(&format!("\"rock_count\": {generated_rock_count}")));
    assert!(manifest.contains(&format!("\"route_cairn_count\": {route_cairn_count}")));
    assert!(manifest.contains("\"launch_beacon_count\": 1"));
    assert!(manifest.contains("\"landing_garden_marker_count\": 4"));
    assert!(manifest.contains(&format!("\"pond_surface_count\": {pond_surface_count}")));
    assert!(manifest.contains(&format!("\"obstruction_spire_count\": {island_count}")));
    assert!(manifest.contains("\"route_cairn_vertical_span_m\""));
    assert!(manifest.contains("\"launch_beacon_vertical_span_m\""));
    assert!(manifest.contains("\"landing_garden_marker_vertical_span_m\""));
    assert!(manifest.contains("\"pond_surface_vertical_span_m\""));
    assert!(manifest.contains("\"plateau_landmark_vertex_total\""));
    assert!(manifest.contains("\"max_plateau_landmark_mesh_vertices\""));
    assert!(manifest.contains("\"plateau_waterfall_vertical_span_m\""));
    assert!(manifest.contains("\"route_waterfall_vertical_span_m\""));
    assert!(manifest.contains("\"route_lake_surface_horizontal_span_m\""));
    assert!(manifest.contains("\"under_route_visual_vertical_span_m\""));
    assert!(manifest.contains("\"ruin_arch_mesh_vertices\""));
    assert!(manifest.contains("\"ruin_arch_vertical_span_m\""));
    assert!(manifest.contains("\"ruin_arch_radius_band_count\""));
    assert!(manifest.contains("\"ruin_arch_normal_slope_band_count\""));
    assert!(manifest.contains("\"kind\": \"plateau_lake_surface\""));
    assert!(manifest.contains("\"kind\": \"plateau_waterfall_ribbon\""));
    assert!(manifest.contains("\"kind\": \"plateau_waterfall_mist\""));
    assert!(manifest.contains("\"kind\": \"plateau_arrival_shelf\""));
    assert!(manifest.contains("\"kind\": \"plateau_arrival_ruin\""));
    assert!(manifest.contains("\"kind\": \"plateau_onward_hint\""));
    assert!(manifest.contains("\"kind\": \"route_waterfall_ribbon\""));
    assert!(manifest.contains("\"kind\": \"route_waterfall_mist\""));
    assert!(manifest.contains("\"kind\": \"route_lake_surface\""));
    assert!(manifest.contains("\"kind\": \"under_route_cave_mouth\""));
    assert!(manifest.contains("\"kind\": \"under_route_hanging_shelf\""));
    assert!(manifest.contains("\"kind\": \"under_route_hanging_roots\""));
    assert!(manifest.contains("\"kind\": \"ruin_arch\""));
    assert!(manifest.contains("\"kind\": \"cliff_teeth\""));
    assert!(manifest.contains("\"kind\": \"garden_ring\""));
    assert!(manifest.contains("\"kind\": \"lake_basin\""));
    assert!(manifest.contains("\"kind\": \"first_expedition_north_ruin_spire\""));
    assert!(manifest.contains("\"kind\": \"first_expedition_south_ruin_spire\""));
    assert!(manifest.contains("\"kind\": \"first_expedition_waterfall_cliff\""));
    assert!(manifest.contains("\"kind\": \"first_expedition_cave_arch\""));
    assert!(manifest.contains("\"kind\": \"first_expedition_ring_garden\""));
    assert!(manifest.contains("\"kind\": \"first_expedition_broken_stair\""));
    assert!(manifest.contains("\"kind\": \"first_expedition_high_crown\""));
    assert!(manifest.contains("great_sky_plateau_low_basin_lake.obj"));
    assert!(manifest.contains("great_sky_plateau_north_rim_waterfall.obj"));
    assert!(manifest.contains("great_sky_plateau_meadow_landing_shelf.obj"));
    assert!(manifest.contains("great_sky_plateau_arrival_ruin_marker.obj"));
    assert!(manifest.contains("great_sky_plateau_high_shelf_onward_hint.obj"));
    assert!(manifest.contains("great_sky_plateau_cave_mouth_onward_hint.obj"));
    assert!(manifest.contains("cloudfall_meadow_route_edge_waterfall.obj"));
    assert!(manifest.contains("cloudfall_meadow_route_edge_mist.obj"));
    assert!(manifest.contains("sapphire_basin_route_lake_surface.obj"));
    assert!(manifest.contains("bluevault_basin_route_lake_basin.obj"));
    assert!(manifest.contains("great_sky_plateau_underhang_entry_arch.obj"));
    assert!(manifest.contains("great_sky_plateau_underside_glide_shelf.obj"));
    assert!(manifest.contains("great_sky_plateau_hanging_root_curtain.obj"));
    assert!(manifest.contains("underbridge_cay_underhang_entry_arch.obj"));
    assert!(manifest.contains("underbridge_cay_underside_glide_shelf.obj"));
    assert!(manifest.contains("underbridge_cay_hanging_root_curtain.obj"));
    assert!(manifest.contains("storm_porch_cliff_teeth.obj"));
    assert!(manifest.contains("landing_garden_garden_ring.obj"));
    assert!(manifest.contains("great_sky_plateau_low_basin_lake_basin.obj"));
    assert!(manifest.contains("mist_arch_north_ruin_spire.obj"));
    assert!(manifest.contains("cloud_gate_south_ruin_spire.obj"));
    assert!(manifest.contains("cloudfall_meadow_waterfall_cliff_silhouette.obj"));
    assert!(manifest.contains("underbridge_cay_cave_mouth_silhouette.obj"));
    assert!(manifest.contains("sunspire_garden_ring_garden_silhouette.obj"));
    assert!(manifest.contains("broken_stair_broken_stair_silhouette.obj"));
    assert!(manifest.contains("upper_crown_high_crown_silhouette.obj"));
    assert!(manifest.contains("\"obstruction_spire_height_band_count\""));
    assert!(manifest.contains("\"obstruction_spire_radius_band_count\""));
    assert!(manifest.contains("\"obstruction_spire_normal_slope_band_count\""));
    assert!(manifest.contains(&format!(
        "\"terrain_biome_palette_count\": {}",
        report.terrain_biome_palette_count
    )));

    remove_existing_dir(&output_dir).expect("visual content export test dir should be removable");
}

#[test]
fn wind_visual_export_writes_motion_tracks_and_manifest() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-wind-visual-export-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    remove_existing_dir(&output_dir).expect("stale wind visual export dir should be removable");

    let report =
        export_wind_visual_inspection(&output_dir).expect("wind visual export should succeed");
    let manifest = fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
    let manifest_json: serde_json::Value =
        serde_json::from_str(&manifest).expect("manifest should be valid json");
    let track_obj = output_dir.join("wind_tracks/wind_visual_tracks.obj");
    let track_ndjson = output_dir.join("wind_tracks/wind_visual_tracks.ndjson");
    let track_lines = fs::read_to_string(&track_ndjson).expect("track ndjson should be readable");
    let updraft_field_count = nau_engine::environment::GAMEPLAY_LIFT_ROUTE.len();
    let crosswind_field_count = nau_engine::environment::VISUAL_CROSSWIND_FIELD_COUNT;
    let updraft_guide_count =
        updraft_field_count * UPDRAFT_GUIDE_RING_LEVELS.len() * UPDRAFT_GUIDES_PER_RING;
    let updraft_ribbon_count = updraft_field_count * UPDRAFT_RIBBONS_PER_FIELD;
    let crosswind_guide_count = crosswind_field_count * CROSSWIND_GUIDES_PER_FIELD;
    let crosswind_ribbon_count = crosswind_field_count * CROSSWIND_RIBBONS_PER_FIELD;
    let sample_window_count = manifest_json["sample_windows_secs"]
        .as_array()
        .expect("sample windows should be present")
        .len();
    let expected_tracks = (updraft_guide_count
        + updraft_ribbon_count * 4
        + crosswind_guide_count
        + crosswind_ribbon_count * 3)
        * sample_window_count;

    assert_eq!(report.track_count, expected_tracks);
    assert!(track_obj.exists());
    assert!(track_ndjson.exists());
    assert_eq!(track_lines.lines().count(), expected_tracks);
    assert!(track_lines.contains("\"family\":\"updraft_guide\""));
    assert!(track_lines.contains("\"family\":\"updraft_ribbon\""));
    assert!(track_lines.contains("\"family\":\"crosswind_guide\""));
    assert!(track_lines.contains("\"family\":\"crosswind_ribbon\""));
    assert!(manifest.contains("\"schema\": \"nau_wind_visual_export.v1\""));
    assert_eq!(
        manifest_json["counts"]["updraft_field_count"].as_u64(),
        Some(updraft_field_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["crosswind_field_count"].as_u64(),
        Some(crosswind_field_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["updraft_guide_count"].as_u64(),
        Some(updraft_guide_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["updraft_ribbon_count"].as_u64(),
        Some(updraft_ribbon_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["crosswind_guide_count"].as_u64(),
        Some(crosswind_guide_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["crosswind_ribbon_count"].as_u64(),
        Some(crosswind_ribbon_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["track_count"].as_u64(),
        Some(expected_tracks as u64)
    );
    let total_motion = &manifest_json["motion"]["total"];
    let static_track_count = total_motion["static_track_count"]
        .as_u64()
        .expect("static track count should be present");
    let off_field_track_count = total_motion["off_field_track_count"]
        .as_u64()
        .expect("off-field track count should be present");
    let low_alignment_track_count = total_motion["low_alignment_track_count"]
        .as_u64()
        .expect("low-alignment track count should be present");
    assert!(static_track_count * 50 <= expected_tracks as u64);
    assert!(off_field_track_count * 1000 <= expected_tracks as u64);
    assert!(low_alignment_track_count * 100 <= expected_tracks as u64 * 9);
    assert!(
        total_motion["coherent_track_count"]
            .as_u64()
            .expect("coherent track count should be present")
            >= expected_tracks as u64 * 17 / 20
    );

    remove_existing_dir(&output_dir).expect("wind visual export test dir should be removable");
}

#[test]
fn spawned_island_visuals_attach_world_collision_proxies() {
    let route = SkyRoute::default();
    let mut catalog = IslandVisualCatalog::default();
    let mut diagnostics = content_diagnostics::IslandContentDiagnostics::default();
    let mut meshes = Assets::<Mesh>::default();
    let material = Handle::<StandardMaterial>::default();
    let surface_material = Handle::<SurfaceMaterial>::default();
    let water_materials = WaterSurfaceMaterials {
        body: surface_material.clone(),
        foam: surface_material.clone(),
        mist: material.clone(),
    };
    let detail_materials = IslandDetailMaterials {
        trunk: material.clone(),
        foliage: material.clone(),
        ground_cover: material.clone(),
        stone: material.clone(),
    };

    for (index, island) in route.islands().iter().copied().enumerate() {
        queue_sky_island(
            &mut catalog,
            &mut diagnostics,
            &mut meshes,
            surface_material.clone(),
            material.clone(),
            material.clone(),
            material.clone(),
            material.clone(),
            detail_materials.clone(),
            material.clone(),
            water_materials.clone(),
            index,
            island,
        );
    }

    let coverage = audit_island_collision_coverage(&catalog, &route);
    assert!(
        coverage.passed,
        "island visual collision coverage should pass:\n{}",
        coverage.failures.join("\n")
    );
    assert!(coverage.checked_visual_count > 0);
    assert!(coverage.solid_visual_count >= 60);
    assert_eq!(
        coverage.terrain_rim_proxy_count,
        route.islands().len() * nau_engine::world::TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND
    );
    assert_eq!(
        coverage.terrain_body_proxy_count,
        route.islands().len() * nau_engine::world::TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND
    );
    assert!(coverage.camera_only_allowance_count >= route.islands().len());
    assert_eq!(
        catalog.named_obstacle_count("great sky plateau", "under-route cave mouth arch"),
        2
    );
    assert_eq!(
        catalog.named_obstacle_count("great sky plateau", "under-route hanging shelf"),
        1
    );
    assert_eq!(
        catalog.named_obstacle_count("great sky plateau", "plateau arrival ruin marker"),
        1
    );
    assert_eq!(
        catalog.named_obstacle_count("great sky plateau", "plateau high shelf route hint"),
        1
    );
    assert_eq!(
        catalog.named_obstacle_count("great sky plateau", "plateau cave route hint"),
        1
    );
    assert_eq!(
        catalog.named_obstacle_count("underbridge cay", "under-route cave mouth arch"),
        2
    );
    assert_eq!(
        catalog.named_obstacle_count("underbridge cay", "under-route hanging shelf"),
        1
    );
    assert_eq!(catalog.deferred_mesh_count(), route.islands().len() * 3);
    assert!(catalog.prebuilt_mesh_count() > catalog.deferred_mesh_count());

    let mut world = World::new();
    let stream_state;
    {
        let mut commands = world.commands();
        stream_state = spawn_initial_island_visuals(
            &mut commands,
            &mut meshes,
            &catalog,
            nau_engine::world::START_POSITION,
        );
    }
    world.flush();
    assert!(stream_state.loaded_mesh_count() > 0);
    assert!(stream_state.loaded_mesh_count() < catalog.deferred_mesh_count());

    let mut query = world.query::<&WorldCollisionProxy>();
    let proxies = query.iter(&world).copied().collect::<Vec<_>>();

    let terrain_rim_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::TerrainRim)
        .count();
    let terrain_body_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::TerrainBody)
        .count();
    let expected_spawned_terrain_rim_proxy_count = catalog.resident_collision_proxy_count(
        nau_engine::world::START_POSITION,
        WorldCollisionProxyKind::TerrainRim,
    );
    let expected_spawned_terrain_body_proxy_count = catalog.resident_collision_proxy_count(
        nau_engine::world::START_POSITION,
        WorldCollisionProxyKind::TerrainBody,
    );
    let expected_spawned_landmark_proxy_count = catalog.resident_collision_proxy_count(
        nau_engine::world::START_POSITION,
        WorldCollisionProxyKind::Landmark,
    );
    let expected_spawned_tree_proxy_count = catalog.resident_collision_proxy_count(
        nau_engine::world::START_POSITION,
        WorldCollisionProxyKind::Tree,
    );
    let expected_spawned_rock_proxy_count = catalog.resident_collision_proxy_count(
        nau_engine::world::START_POSITION,
        WorldCollisionProxyKind::Rock,
    );
    let tree_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::Tree)
        .count();
    let rock_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::Rock)
        .count();
    let landmark_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::Landmark)
        .count();
    assert!(proxies.len() >= 24);
    assert!(tree_proxy_count > 0);
    assert!(rock_proxy_count > 0);
    assert!(landmark_proxy_count >= 24);
    assert_eq!(tree_proxy_count, expected_spawned_tree_proxy_count);
    assert_eq!(rock_proxy_count, expected_spawned_rock_proxy_count);
    assert_eq!(landmark_proxy_count, expected_spawned_landmark_proxy_count);
    assert_eq!(
        terrain_rim_proxy_count,
        expected_spawned_terrain_rim_proxy_count
    );
    assert_eq!(
        terrain_body_proxy_count,
        expected_spawned_terrain_body_proxy_count
    );
    assert!(
        proxies
            .iter()
            .any(|proxy| proxy.kind == WorldCollisionProxyKind::Tree)
    );
    assert!(
        proxies
            .iter()
            .any(|proxy| proxy.kind == WorldCollisionProxyKind::Rock)
    );
    assert!(
        proxies
            .iter()
            .any(|proxy| proxy.kind == WorldCollisionProxyKind::Landmark)
    );
    assert!(
        proxies
            .iter()
            .filter(|proxy| {
                proxy.kind == WorldCollisionProxyKind::Landmark
                    && (proxy.half_extents.x > 3.0 || proxy.half_extents.z > 3.0)
            })
            .count()
            >= 4
    );
}

#[test]
fn tree_trunk_mesh_is_tapered_instead_of_a_straight_cylinder() {
    let mesh = tree_trunk_mesh(0.3, 4.0, 123);
    let positions = positions(&mesh);
    let bottom_ring = &positions[..TREE_TRUNK_SEGMENTS];
    let top_ring_start = TREE_TRUNK_SEGMENTS * (TREE_TRUNK_RING_COUNT - 1);
    let top_ring = &positions[top_ring_start..top_ring_start + TREE_TRUNK_SEGMENTS];
    let branch_vertices_start = TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT + 2;
    let root_vertices_start = branch_vertices_start + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2;
    let top_center = top_ring
        .iter()
        .map(|position| Vec2::new(position[0], position[2]))
        .sum::<Vec2>()
        / TREE_TRUNK_SEGMENTS as f32;
    let average_bottom_radius = bottom_ring
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .sum::<f32>()
        / TREE_TRUNK_SEGMENTS as f32;
    let average_top_radius = top_ring
        .iter()
        .map(|position| (Vec2::new(position[0], position[2]) - top_center).length())
        .sum::<f32>()
        / TREE_TRUNK_SEGMENTS as f32;
    let max_branch_reach = positions[branch_vertices_start..root_vertices_start]
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);
    let max_root_reach = positions[root_vertices_start..]
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT
            + 2
            + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2
            + TREE_ROOT_FLARE_COUNT * TREE_ROOT_FLARE_SEGMENTS * 2
    );
    assert!(
        average_bottom_radius > average_top_radius * 1.45,
        "tree trunks should taper enough to stop reading as plain cylinders"
    );
    assert!(
        max_branch_reach > average_bottom_radius * 1.8,
        "tree trunks should include visible branch mass instead of only a tapered stick"
    );
    assert!(
        max_root_reach > average_bottom_radius * 1.35,
        "tree trunks should include root flares that break the pole silhouette near the ground"
    );
}

#[test]
fn tree_trunk_cap_winding_matches_declared_normals() {
    let mesh = tree_trunk_mesh(0.3, 4.0, 123);
    let positions = positions(&mesh);
    let indices = u32_indices(&mesh);
    let cap_start = TREE_TRUNK_SEGMENTS * 6 * (TREE_TRUNK_RING_COUNT - 1);

    for segment in 0..TREE_TRUNK_SEGMENTS {
        let bottom = &indices[cap_start + segment * 6..cap_start + segment * 6 + 3];
        let top = &indices[cap_start + segment * 6 + 3..cap_start + segment * 6 + 6];

        assert!(
            triangle_normal_y(positions, bottom) < 0.0,
            "bottom cap triangles should face downward"
        );
        assert!(
            triangle_normal_y(positions, top) > 0.0,
            "top cap triangles should face upward"
        );
    }
}

#[test]
fn tree_canopy_mesh_uses_overlapping_lobes_instead_of_one_sphere() {
    let mesh = tree_canopy_mesh(1.4, 42);
    let positions = positions(&mesh);
    let single_lobe_vertices =
        (TREE_CANOPY_LATITUDE_SEGMENTS + 1) * (TREE_CANOPY_LONGITUDE_SEGMENTS + 1);
    let secondary_lobe_vertices = 5 * ((4 + 1) * (8 + 1));
    let lobe_vertices = single_lobe_vertices + secondary_lobe_vertices;
    let expected_card_vertices = TREE_CANOPY_CARD_COUNT * DETAIL_CARD_VERTICES;
    let skirt_card_vertices = 6 * DETAIL_CARD_VERTICES;
    let skirt_start = lobe_vertices + expected_card_vertices - skirt_card_vertices;
    let skirt_positions = &positions[skirt_start..skirt_start + skirt_card_vertices];
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let horizontal_span = positions
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);
    let avg_skirt_y = skirt_positions
        .iter()
        .map(|position| position[1])
        .sum::<f32>()
        / skirt_positions.len() as f32;
    let skirt_horizontal_span = skirt_positions
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);

    assert!(mesh.count_vertices() > single_lobe_vertices * 3);
    assert!(mesh.count_vertices() >= single_lobe_vertices + expected_card_vertices);
    assert!(mesh.count_vertices() >= 460);
    assert!(max_y - min_y > 1.9);
    assert!(horizontal_span > 1.45);
    assert!(avg_skirt_y < -0.18);
    assert!(skirt_horizontal_span > 1.0);
}

#[test]
fn cloud_cluster_mesh_uses_multiple_lobes_for_depth() {
    let mesh = cloud_cluster_mesh(99, CLOUD_BANK_LOBES);
    let positions = positions(&mesh);
    let colors = colors(&mesh);
    let lobe_vertices = (5 + 1) * (10 + 1);
    let card_vertices = CLOUD_WISP_CARDS_PER_LOBE * DETAIL_CARD_VERTICES;
    let filament_vertices = CLOUD_FILAMENT_RIBBONS_PER_LOBE * CLOUD_FILAMENT_RIBBON_VERTICES;
    let per_lobe_vertices = lobe_vertices + card_vertices + filament_vertices;
    let mut lower_depth_wisp_count = 0usize;
    let min_x = positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_z = positions
        .iter()
        .map(|position| position[2])
        .fold(f32::INFINITY, f32::min);
    let max_z = positions
        .iter()
        .map(|position| position[2])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(mesh.count_vertices(), CLOUD_BANK_LOBES * per_lobe_vertices);
    assert_eq!(colors.len(), positions.len());
    assert_eq!(
        cloud_filament_ribbon_detail_count(CLOUD_BANK_LOBES),
        CLOUD_BANK_LOBES * CLOUD_FILAMENT_RIBBONS_PER_LOBE
    );
    assert!(
        max_x - min_x > 1.2,
        "cloud clusters should have lateral lobe structure"
    );
    assert!(
        max_y - min_y > 1.0,
        "cloud clusters should stack lobes vertically instead of staying wafer-flat"
    );
    assert!(
        max_z - min_z > 0.8,
        "cloud clusters should have visible depth, not one flat blob"
    );
    for lobe in 0..CLOUD_BANK_LOBES {
        let lobe_start = lobe * per_lobe_vertices + lobe_vertices;
        let lower_depth_start = lobe_start + (CLOUD_WISP_CARDS_PER_LOBE - 1) * DETAIL_CARD_VERTICES;
        let lower_depth_wisp =
            &positions[lower_depth_start..lower_depth_start + DETAIL_CARD_VERTICES];
        let upper_wisp = &positions[lobe_start..lower_depth_start];
        let lower_avg_y = lower_depth_wisp
            .iter()
            .map(|position| position[1])
            .sum::<f32>()
            / lower_depth_wisp.len() as f32;
        let upper_avg_y =
            upper_wisp.iter().map(|position| position[1]).sum::<f32>() / upper_wisp.len() as f32;

        if lower_avg_y + 0.05 < upper_avg_y {
            lower_depth_wisp_count += 1;
        }
    }
    assert!(
        lower_depth_wisp_count > CLOUD_BANK_LOBES * 3 / 4,
        "cloud clusters should add lower depth wisps under most lobes"
    );
}

#[test]
fn cloud_cluster_mesh_has_painterly_warm_and_cool_color_wash() {
    let mesh = cloud_cluster_mesh(123, CLOUD_BANK_LOBES);
    let colors = colors(&mesh);
    let min_alpha = colors
        .iter()
        .map(|color| color[3])
        .fold(f32::INFINITY, f32::min);
    let max_alpha = colors
        .iter()
        .map(|color| color[3])
        .fold(f32::NEG_INFINITY, f32::max);
    let warm_vertices = colors
        .iter()
        .filter(|color| color[0] > color[2] && color[1] > 0.72)
        .count();
    let cool_shadow_vertices = colors
        .iter()
        .filter(|color| color[2] >= color[0] && color[1] < 0.84)
        .count();

    assert!(
        mesh_vertex_color_band_count(&mesh) >= 8,
        "clouds should carry watercolor-like color variation instead of one flat material color"
    );
    assert!(
        max_alpha - min_alpha > 0.12,
        "cloud core and wisp geometry should have layered opacity"
    );
    assert!(
        warm_vertices > colors.len() / 5,
        "clouds should include warm sunlit wash vertices"
    );
    assert!(
        cool_shadow_vertices > colors.len() / 8,
        "clouds should include cooler shadowed vapor vertices"
    );
}

#[test]
fn water_materials_preserve_readable_physical_contract() {
    let mut images = Assets::<Image>::default();
    let mut materials = Assets::<SurfaceMaterial>::default();
    let mist = Handle::<StandardMaterial>::default();
    let handles = water_surface_materials(&mut images, &mut materials, mist);
    let body = materials
        .get(&handles.body)
        .expect("water body material exists");
    let foam = materials
        .get(&handles.foam)
        .expect("water foam material exists");

    assert_eq!(body.base.cull_mode, None);
    assert!(body.base.double_sided);
    assert_eq!(body.base.alpha_mode, AlphaMode::Blend);
    assert!(body.base.reflectance <= 0.4);
    assert!(body.base.clearcoat <= 0.1);
    assert_eq!(foam.base.alpha_mode, AlphaMode::Add);
    assert!(foam.base.unlit);
}

#[test]
fn ground_cover_mesh_uses_dense_curved_blades() {
    let island = test_island();
    let patch_count = island_detail_budget(island).ground_cover_patch_count;
    let mesh = island_ground_cover_mesh(2, island, patch_count);
    let positions = positions(&mesh);
    let indices = u32_indices(&mesh);
    let blade_count = patch_count * GROUND_COVER_BLADES_PER_PATCH;
    let mut side_leaf_count = 0usize;
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        blade_count * VERTICES_PER_GROUND_BLADE
    );
    assert_eq!(indices.len(), blade_count * INDICES_PER_GROUND_BLADE);
    assert!(
        max_y - min_y > 1.0,
        "ground cover should have enough varied height to read as dense vegetation"
    );
    for (blade_index, blade) in positions
        .chunks_exact(VERTICES_PER_GROUND_BLADE)
        .enumerate()
    {
        let base_center = (Vec3::from_array(blade[0]) + Vec3::from_array(blade[1])) * 0.5;
        let mid_center = (Vec3::from_array(blade[2]) + Vec3::from_array(blade[3])) * 0.5;
        let tip = Vec3::from_array(blade[4]);
        let leaflet = Vec3::from_array(blade[5]);
        let main_axis = Vec2::new(tip.x - base_center.x, tip.z - base_center.z).normalize();
        let side_axis = Vec2::new(-main_axis.y, main_axis.x);
        let side_offset = Vec2::new(leaflet.x - mid_center.x, leaflet.z - mid_center.z)
            .dot(side_axis)
            .abs();

        if side_offset > 0.08 && leaflet.y > mid_center.y && leaflet.y < tip.y {
            side_leaf_count += 1;
        }

        let blade_vertex_start = blade_index * VERTICES_PER_GROUND_BLADE;
        let blade_vertex_end = blade_vertex_start + VERTICES_PER_GROUND_BLADE;
        let blade_index_start = blade_index * INDICES_PER_GROUND_BLADE;
        for index in &indices[blade_index_start..blade_index_start + INDICES_PER_GROUND_BLADE] {
            let index = *index as usize;
            assert!(
                index >= blade_vertex_start && index < blade_vertex_end,
                "ground cover indices should stay inside their blade chunk"
            );
        }
    }
    assert!(
        side_leaf_count > blade_count * 9 / 10,
        "ground cover should add side leaflets to most blades"
    );
}

#[test]
fn island_detail_surface_offsets_clamp_to_playable_silhouette() {
    let island = SkyIsland::new(
        "storm porch",
        Vec3::ZERO,
        Vec2::new(42.0, 28.0),
        15.0,
        false,
    );
    let mut narrow_angle = 0.0;
    let mut narrow_scale = f32::INFINITY;
    for step in 0..128 {
        let angle = step as f32 / 128.0 * std::f32::consts::TAU;
        let scale = island.playable_silhouette_scale(angle);
        if scale < narrow_scale {
            narrow_angle = angle;
            narrow_scale = scale;
        }
    }

    let outside_offset = Vec2::new(narrow_angle.cos(), narrow_angle.sin()) * 0.92;
    let clamped_offset = island_playable_normalized_offset(island, outside_offset);
    let surface = island_visual_surface_position(island, outside_offset);

    assert!(clamped_offset.length() < outside_offset.length());
    assert!(clamped_offset.length() <= narrow_scale * 0.941);
    assert!(island.contains_horizontal(surface));
}

#[test]
fn rock_scatter_mesh_has_flattened_irregular_silhouette() {
    let mesh = rock_scatter_mesh(0.7, 1234);
    let positions = positions(&mesh);
    let indices = u32_indices(&mesh);
    let radial_lengths: Vec<f32> = positions[1..positions.len() - 1]
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .collect();
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_radius = radial_lengths.iter().copied().fold(f32::INFINITY, f32::min);
    let max_radius = radial_lengths
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let top_cap_start = ROCK_MESH_SEGMENTS * 3 + (ROCK_MESH_RINGS - 1) * ROCK_MESH_SEGMENTS * 6;

    assert_eq!(
        mesh.count_vertices(),
        ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2
    );
    assert!(
        max_radius - min_radius > 0.5,
        "rock scatter should have a jagged profile instead of one repeated radius"
    );
    assert!(
        max_radius > (max_y - min_y) * 0.95,
        "rock scatter should be squat and grounded rather than a sphere"
    );
    assert!(
        triangle_normal_y(positions, &indices[0..3]) < 0.0,
        "rock bottom cap should face downward"
    );
    assert!(
        triangle_normal_y(positions, &indices[top_cap_start..top_cap_start + 3]) > 0.0,
        "rock top cap should face upward"
    );
}

#[test]
fn landmark_meshes_replace_basic_cylinders_and_boxes() {
    let cairn = route_cairn_mesh(0.44, 4.2, 12_345);
    let cairn_positions = positions(&cairn);
    let cairn_y_range = mesh_y_range(&cairn);
    let cairn_radius_range = radial_range(cairn_positions);

    assert!(
        cairn.count_vertices() > ROUTE_CAIRN_STONE_COUNT * 40,
        "route cairns should be stacked stone meshes, not one cylinder"
    );
    assert!(cairn_y_range > 3.0);
    assert!(
        cairn_radius_range > 0.18,
        "route cairn stones should vary their silhouette radius"
    );

    let launch_beacon = launch_beacon_mesh(0.78, 3.2, 14_321);
    let launch_positions = positions(&launch_beacon);
    assert!(
        launch_beacon.count_vertices() > cairn.count_vertices() + LAUNCH_BEACON_CRYSTAL_COUNT * 10,
        "launch beacon should add shard geometry on top of its stone base"
    );
    assert!(
        mesh_y_range(&launch_beacon) > 2.8,
        "launch beacon should read as a vertical landmark"
    );
    assert!(
        launch_positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max)
            > 2.0
    );

    let marker = landing_garden_marker_mesh(8.0, 0.62, 13_579);
    let marker_positions = positions(&marker);
    let marker_indices = u32_indices(&marker);
    let min_x = marker_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = marker_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        marker.count_vertices(),
        (LANDING_GARDEN_MARKER_SEGMENTS + 1) * 3
    );
    assert_eq!(marker_indices.len(), LANDING_GARDEN_MARKER_SEGMENTS * 12);
    assert!(max_x - min_x > 7.8);
    assert!(
        mesh_y_range(&marker) > 0.12,
        "landing garden markers should be low organic mounds, not flat boxes"
    );

    let ruin_arch = ruin_arch_mesh(9.0, 6.2, 2.2, 15_789);
    let ruin_positions = positions(&ruin_arch);
    assert!(
        ruin_arch.count_vertices() >= RUIN_ARCH_STONE_COUNT * 50,
        "ruin arches should be made from separate stacked stones"
    );
    assert!(
        mesh_y_range(&ruin_arch) > 6.0,
        "ruin arches should form a readable vertical opening"
    );
    assert!(
        radial_range(ruin_positions) > 3.0,
        "ruin arches should have a broad broken-stone silhouette"
    );
    assert!(
        mesh_normal_slope_band_count(&ruin_arch) >= 5,
        "ruin arches should not collapse into one flat slab"
    );

    let pond = pond_surface_mesh(3.2, 1.4, 11_789);
    let pond_positions = positions(&pond);
    let pond_indices = u32_indices(&pond);
    let pond_radius_range = radial_range(pond_positions);

    assert_eq!(pond.count_vertices(), 1 + POND_SURFACE_SEGMENTS * 2);
    assert_eq!(pond_indices.len(), POND_SURFACE_SEGMENTS * 9);
    assert!(
        pond_radius_range > 2.1,
        "pond perimeter should be an irregular surface mesh, not a scaled cylinder"
    );
    assert!(
        mesh_y_range(&pond) > 0.015,
        "pond surface should carry subtle ripple variation"
    );

    let lake = lake_surface_mesh(18.0, 9.0, 31_789);
    let lake_positions = positions(&lake);
    let lake_indices = u32_indices(&lake);
    assert_eq!(lake.count_vertices(), 1 + LAKE_SURFACE_SEGMENTS * 3);
    assert_eq!(lake_indices.len(), LAKE_SURFACE_SEGMENTS * 15);
    assert!(
        radial_range(lake_positions) > 14.0,
        "large lakes should read as broad irregular basins"
    );
    assert!(
        mesh_y_range(&lake) > 0.05,
        "large lakes should carry stronger watercolor ripple relief than small ponds"
    );

    let waterfall = waterfall_ribbon_mesh(16.0, 60.0, 1.4, 33_789);
    let waterfall_indices = u32_indices(&waterfall);
    assert_eq!(
        waterfall.count_vertices(),
        WATERFALL_RIBBON_COLUMNS * WATERFALL_RIBBON_ROWS
    );
    assert_eq!(
        waterfall_indices.len(),
        (WATERFALL_RIBBON_COLUMNS - 1) * (WATERFALL_RIBBON_ROWS - 1) * 6
    );
    assert!(
        mesh_y_range(&waterfall) > 59.0,
        "waterfalls should be vertical traversal-scale ribbons"
    );
    assert!(
        mesh_normal_slope_band_count(&waterfall) >= 4,
        "waterfall ribbons should not be a single flat quad"
    );

    let mist = waterfall_mist_mesh(7.0, 5.0, 34_789);
    assert!(
        mist.count_vertices() > WATERFALL_MIST_LOBES * 40,
        "waterfall mist should be built from multiple soft lobes"
    );
    assert!(
        mesh_y_range(&mist) > 4.0,
        "mist should have visible depth instead of a flat decal"
    );

    let cave_arch = cave_mouth_arch_mesh(24.0, 18.0, 6.0, 41_789);
    assert!(
        cave_arch.count_vertices() >= CAVE_MOUTH_ARCH_STONES * 40,
        "cave mouths should be built from stacked arch stones, not one flat decal"
    );
    assert!(
        mesh_y_range(&cave_arch) > 14.0,
        "cave mouth arches should frame a readable flight opening"
    );

    let underhang_shelf = underhang_shelf_mesh(54.0, 30.0, 4.0, 42_789);
    let shelf_indices = u32_indices(&underhang_shelf);
    assert_eq!(
        underhang_shelf.count_vertices(),
        UNDERHANG_SHELF_SEGMENTS * 2
    );
    assert_eq!(
        shelf_indices.len(),
        (UNDERHANG_SHELF_SEGMENTS - 2) * 6 + UNDERHANG_SHELF_SEGMENTS * 6
    );
    assert!(
        radial_range(positions(&underhang_shelf)) > 10.0,
        "underhang shelves should have broad irregular ledge silhouettes"
    );

    let hanging_roots = hanging_root_curtain_mesh(32.0, 12.0, 8.0, 44_789);
    let hanging_root_indices = u32_indices(&hanging_roots);
    assert_eq!(
        hanging_roots.count_vertices(),
        HANGING_ROOT_STRANDS * (HANGING_ROOT_SEGMENTS + 1) * 4
    );
    assert_eq!(
        hanging_root_indices.len(),
        HANGING_ROOT_STRANDS * HANGING_ROOT_SEGMENTS * 24
    );
    assert!(
        mesh_y_range(&hanging_roots) > 9.0,
        "hanging roots should visibly descend from the cave ceiling"
    );
    assert!(
        radial_range(positions(&hanging_roots)) > 12.0,
        "hanging roots should form a broad organic curtain, not one strand"
    );

    let cliff_teeth = cliff_tooth_ridge_mesh(24.0, 8.0, 4.0, 45_789);
    let cliff_teeth_indices = u32_indices(&cliff_teeth);
    assert_eq!(
        cliff_teeth.count_vertices(),
        CLIFF_TOOTH_COUNT * CLIFF_TOOTH_TRIANGLES_PER_TOOTH * 3
    );
    assert_eq!(
        cliff_teeth_indices.len(),
        CLIFF_TOOTH_COUNT * CLIFF_TOOTH_TRIANGLES_PER_TOOTH * 3
    );
    assert!(
        mesh_y_range(&cliff_teeth) > 7.0,
        "cliff teeth should create sharp vertical silhouette peaks"
    );
    assert!(
        radial_range(positions(&cliff_teeth)) > 10.0,
        "cliff teeth should form a broad broken ridge, not one spike"
    );
    assert!(
        mesh_normal_slope_band_count(&cliff_teeth) >= 4,
        "cliff teeth should keep faceted fracture planes"
    );

    let garden_ring = garden_ring_mesh(6.0, 1.2, 0.6, 46_789);
    let garden_ring_positions = positions(&garden_ring);
    let garden_ring_indices = u32_indices(&garden_ring);
    let garden_min_x = garden_ring_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let garden_max_x = garden_ring_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        garden_ring.count_vertices(),
        (GARDEN_RING_SEGMENTS + 1) * GARDEN_RING_BANDS
    );
    assert_eq!(
        garden_ring_indices.len(),
        GARDEN_RING_SEGMENTS * (GARDEN_RING_BANDS - 1) * 6
    );
    assert!(
        garden_max_x - garden_min_x > 11.0,
        "garden rings should be broad circular landmarks, not short strips"
    );
    assert!(
        radial_range(garden_ring_positions) > 0.7,
        "garden rings should have measurable annular width"
    );
    assert!(
        mesh_y_range(&garden_ring) > 0.45,
        "garden rings should have low organic mound relief instead of a flat decal"
    );
    assert!(
        mesh_normal_slope_band_count(&garden_ring) >= 3,
        "garden rings should vary their soft ridge slopes"
    );

    let lake_basin = lake_basin_rim_mesh(24.0, 14.0, 2.4, 1.2, 47_789);
    let lake_basin_positions = positions(&lake_basin);
    let lake_basin_indices = u32_indices(&lake_basin);
    let lake_basin_min_x = lake_basin_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let lake_basin_max_x = lake_basin_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let lake_basin_min_z = lake_basin_positions
        .iter()
        .map(|position| position[2])
        .fold(f32::INFINITY, f32::min);
    let lake_basin_max_z = lake_basin_positions
        .iter()
        .map(|position| position[2])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        lake_basin.count_vertices(),
        (LAKE_BASIN_RIM_SEGMENTS + 1) * LAKE_BASIN_RIM_BANDS
    );
    assert_eq!(
        lake_basin_indices.len(),
        LAKE_BASIN_RIM_SEGMENTS * (LAKE_BASIN_RIM_BANDS - 1) * 6
    );
    assert!(
        lake_basin_max_x - lake_basin_min_x > 45.0,
        "lake basin rims should read as broad terrain-scale shorelines"
    );
    assert!(
        lake_basin_max_z - lake_basin_min_z > 25.0,
        "lake basin rims should preserve elliptical basin depth"
    );
    assert!(
        mesh_y_range(&lake_basin) > 0.9,
        "lake basin rims should have terraced shoreline relief"
    );
    assert!(
        mesh_normal_slope_band_count(&lake_basin) >= 4,
        "lake basin rims should include varied inner and outer slopes"
    );

    let spire = obstruction_spire_mesh(1.0, 5.2, 18_123);
    let spire_positions = positions(&spire);
    let spire_indices = u32_indices(&spire);
    assert_eq!(
        spire.count_vertices(),
        OBSTRUCTION_SPIRE_RINGS * OBSTRUCTION_SPIRE_SEGMENTS + 2 + OBSTRUCTION_SPIRE_RIB_COUNT * 8
    );
    assert_eq!(
        spire_indices.len(),
        OBSTRUCTION_SPIRE_SEGMENTS * 6
            + (OBSTRUCTION_SPIRE_RINGS - 1) * OBSTRUCTION_SPIRE_SEGMENTS * 6
            + OBSTRUCTION_SPIRE_RIB_COUNT * 12
    );
    assert!(
        mesh_y_range(&spire) > 5.0,
        "obstruction spires should read as tall terrain-integrated blockers"
    );
    assert!(
        radial_range(spire_positions) > 1.0,
        "obstruction spires should have roots/ribs instead of a box footprint"
    );
    assert!(mesh_vertical_band_count(&spire) >= 6);
    assert!(mesh_normal_slope_band_count(&spire) >= 5);
}

#[test]
fn terrain_mesh_uses_high_resolution_irregular_silhouette() {
    let island = test_island();
    let mesh = island_terrain_mesh(2, island);
    let positions = positions(&mesh);
    let colors = colors(&mesh);
    let outer_ring_start = 1 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS;
    let outer_ring = &positions[outer_ring_start..outer_ring_start + ISLAND_BODY_SEGMENTS];
    let min_radius = outer_ring
        .iter()
        .map(|position| normalized_radius(island, *position))
        .fold(f32::INFINITY, f32::min);
    let max_radius = outer_ring
        .iter()
        .map(|position| normalized_radius(island, *position))
        .fold(f32::NEG_INFINITY, f32::max);
    let min_visual_radius = outer_ring
        .iter()
        .map(|position| normalized_visual_radius(island, *position))
        .fold(f32::INFINITY, f32::min);
    let max_visual_radius = outer_ring
        .iter()
        .map(|position| normalized_visual_radius(island, *position))
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        1 + (ISLAND_TERRAIN_RINGS + 1) * ISLAND_BODY_SEGMENTS
    );
    assert!(
        min_visual_radius >= 0.999 && max_visual_radius <= 1.001,
        "terrain top ring must meet the visual cliff contour without a see-through rim gap"
    );
    assert!(
        max_radius - min_radius > 0.10,
        "outer ring should not read as a perfect cylinder"
    );
    assert_eq!(colors.len(), positions.len());
    let skirt_start = 1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS;
    for segment in 0..ISLAND_BODY_SEGMENTS {
        let terrain = Vec3::from_array(outer_ring[segment]);
        let skirt = Vec3::from_array(positions[skirt_start + segment]);
        assert!(
            Vec2::new(terrain.x, terrain.z).distance(Vec2::new(skirt.x, skirt.z)) < 0.001,
            "terrain edge skirt should drop vertically instead of opening a horizontal rim gap"
        );
        assert!(
            terrain.y - skirt.y >= ISLAND_TERRAIN_EDGE_SKIRT_DEPTH_M - 0.001,
            "terrain edge skirt should be deep enough to hide the terrain/cliff seam"
        );
    }
    assert!(
        mesh_vertex_color_band_count(&mesh) >= ISLAND_TERRAIN_COLOR_BANDS,
        "terrain mesh should carry vertex-color biome/detail variation"
    );
    assert!(
        mesh_terrain_material_weight_band_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS,
        "terrain mesh should carry material-weight variation for future PBR blends"
    );
    assert!(
        mesh_terrain_material_channel_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_CHANNELS,
        "terrain mesh should expose base, lush, and edge material channels"
    );
    assert!(
        mesh_terrain_material_region_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_REGIONS,
        "terrain mesh should expose distinct meadow, transition, highland, and edge regions"
    );
    assert!(
        mesh_vertical_band_count(&mesh) >= ISLAND_TERRAIN_HEIGHT_BANDS,
        "terrain mesh should expose enough vertical bands for ravines and terrace relief"
    );
    assert!(
        mesh_normal_slope_band_count(&mesh) >= ISLAND_TERRAIN_NORMAL_SLOPE_BANDS,
        "terrain mesh normals should expose slope variety rather than one smooth plate"
    );
    let uvs = mesh_uv0(&mesh).expect("terrain mesh should expose material uvs");
    let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
    let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
    let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);
    let min_x = positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_z = positions
        .iter()
        .map(|position| position[2])
        .fold(f32::INFINITY, f32::min);
    let max_z = positions
        .iter()
        .map(|position| position[2])
        .fold(f32::NEG_INFINITY, f32::max);
    let u_span = max_u - min_u;
    let v_span = max_v - min_v;
    let expected_u_span = (max_x - min_x) * TERRAIN_UV_TILES_PER_METER;
    let expected_v_span = (max_z - min_z) * TERRAIN_UV_TILES_PER_METER;
    assert!(
        (u_span - expected_u_span).abs() < 0.001 && (v_span - expected_v_span).abs() < 0.001,
        "terrain UVs should preserve the authored world-space texel scale"
    );
    assert!(
        u_span >= 2.0 && v_span >= 1.5,
        "terrain albedo should repeat across large islands without obvious stretching"
    );
    assert!(
        mesh_y_range(&mesh) >= 0.8,
        "terrain mesh should have enough relief range to avoid flat plateaus"
    );
}

#[test]
fn terrain_and_cliff_top_rings_share_a_closed_visual_seam() {
    let island = test_island();
    let terrain_mesh = island_terrain_mesh(2, island);
    let cliff_mesh = island_cliff_mesh(2, island);
    let terrain_positions = positions(&terrain_mesh);
    let cliff_positions = positions(&cliff_mesh);
    let terrain_outer_start = 1 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS;

    for segment in 0..ISLAND_BODY_SEGMENTS {
        let terrain = Vec3::from_array(terrain_positions[terrain_outer_start + segment]);
        let cliff = Vec3::from_array(cliff_positions[segment]);
        assert!(
            terrain.distance(cliff) < 0.001,
            "terrain and cliff top rings should meet without a hollow rim seam"
        );
    }
}

#[test]
fn island_impostor_mesh_uses_layered_color_and_silhouette() {
    let island = test_island();
    let mesh = island_impostor_mesh(4, island);
    let terrain_mesh = island_terrain_mesh(4, island);
    let underside_mesh = island_underside_mesh(4, island);
    let impostor_positions = positions(&mesh);
    let terrain_positions = positions(&terrain_mesh);
    let underside_positions = positions(&underside_mesh);
    let colors = colors(&mesh);
    let top_ring_start = 1 + (ISLAND_IMPOSTOR_TERRAIN_RINGS - 1) * ISLAND_IMPOSTOR_SEGMENTS;
    let top_ring = &impostor_positions[top_ring_start..top_ring_start + ISLAND_IMPOSTOR_SEGMENTS];
    let terrain_outer_start = 1 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS;
    let terrain_step = ISLAND_BODY_SEGMENTS / ISLAND_IMPOSTOR_SEGMENTS;
    let min_radius = top_ring
        .iter()
        .map(|position| normalized_radius(island, *position))
        .fold(f32::INFINITY, f32::min);
    let max_radius = top_ring
        .iter()
        .map(|position| normalized_radius(island, *position))
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        EXPECTED_ISLAND_IMPOSTOR_MESH_VERTICES
    );
    assert_eq!(colors.len(), impostor_positions.len());
    assert!(
        max_radius - min_radius > 0.08,
        "distant impostor should keep an irregular island silhouette"
    );
    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        let terrain =
            Vec3::from_array(terrain_positions[terrain_outer_start + segment * terrain_step]);
        let impostor = Vec3::from_array(top_ring[segment]);
        assert!(
            terrain.distance(impostor) < 0.001,
            "distant impostor top ring should preserve the full terrain footprint"
        );
    }
    let impostor_tip = Vec3::from_array(
        *impostor_positions
            .last()
            .expect("impostor bottom tip exists"),
    );
    let underside_tip =
        Vec3::from_array(*underside_positions.last().expect("underside tip exists"));
    assert!(
        impostor_tip.distance(underside_tip) < 0.001,
        "distant impostor should keep the full underside depth"
    );
    assert!(
        mesh_y_range(&mesh) >= island.thickness * 1.5,
        "distant impostor should include a readable underside mass"
    );
    assert!(
        mesh_vertex_color_band_count(&mesh) >= ISLAND_IMPOSTOR_COLOR_BANDS,
        "distant impostor should carry terrain, cliff, and underside color variation"
    );
}

#[test]
fn cliff_and_underside_meshes_replace_cylinder_body_resolution() {
    let island = test_island();
    let cliff_mesh = island_cliff_mesh(3, island);
    let underside_mesh = island_underside_mesh(3, island);
    let underside_positions = positions(&underside_mesh);
    let underside_top_radius = normalized_radius(island, underside_positions[0]);
    let underside_tip = *underside_positions.last().expect("bottom tip exists");

    assert_eq!(
        cliff_mesh.count_vertices(),
        (ISLAND_CLIFF_RINGS + 1) * ISLAND_BODY_SEGMENTS
    );
    assert_eq!(
        underside_mesh.count_vertices(),
        (ISLAND_UNDERSIDE_RINGS + 1) * ISLAND_BODY_SEGMENTS + 1
    );
    assert!(underside_top_radius > 0.55);
    assert!(normalized_radius(island, underside_tip) < 0.01);
    assert!(underside_tip[1] < island.mesh_top_y() - island.thickness * 1.5);
    assert!(
        mesh_vertex_color_band_count(&cliff_mesh) >= ISLAND_CLIFF_STRATA_BANDS,
        "cliff mesh should carry visible strata color bands"
    );
    assert!(
        mesh_vertex_color_band_count(&underside_mesh) >= ISLAND_CLIFF_STRATA_BANDS / 2,
        "underside mesh should not be one flat rock color"
    );
}

#[test]
fn cliff_and_underside_share_their_transition_ring() {
    let island = test_island();
    let cliff_mesh = island_cliff_mesh(3, island);
    let underside_mesh = island_underside_mesh(3, island);
    let cliff_positions = positions(&cliff_mesh);
    let underside_positions = positions(&underside_mesh);
    let cliff_bottom_start = ISLAND_CLIFF_RINGS * ISLAND_BODY_SEGMENTS;

    for segment in 0..ISLAND_BODY_SEGMENTS {
        let cliff = Vec3::from_array(cliff_positions[cliff_bottom_start + segment]);
        let underside = Vec3::from_array(underside_positions[segment]);
        assert!(
            cliff.distance(underside) < 0.001,
            "cliff and underside should not leave a visible body seam"
        );
    }
}

#[test]
fn terrain_surface_texture_has_coherent_multiscale_detail() {
    let data = procedural_terrain_surface_texture_data(
        [54, 128, 70, 255],
        [28, 92, 48, 255],
        [128, 174, 78, 255],
        17,
        TERRAIN_TEXTURE_SIZE,
    );

    assert_eq!(
        data.len(),
        (TERRAIN_TEXTURE_SIZE * TERRAIN_TEXTURE_SIZE * 4) as usize
    );
    assert!(
        texture_detail_band_count(&data) >= ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS,
        "terrain texture should retain enough palette range to avoid flat fills"
    );
    let near_detail = texture_detail_energy_promille(&data, TERRAIN_TEXTURE_SIZE, 1);
    let mid_detail = texture_detail_energy_promille(&data, TERRAIN_TEXTURE_SIZE, 4);
    let macro_detail = texture_detail_energy_promille(&data, TERRAIN_TEXTURE_SIZE, 16);

    assert!(
        near_detail >= ISLAND_TERRAIN_TEXTURE_NEAR_DETAIL_ENERGY_PROMILLE,
        "terrain texture should retain subtle near detail"
    );
    assert!(
        mid_detail >= ISLAND_TERRAIN_TEXTURE_MID_DETAIL_ENERGY_PROMILLE,
        "terrain texture should retain readable mid-scale variation"
    );
    assert!(
        macro_detail >= ISLAND_TERRAIN_TEXTURE_MACRO_DETAIL_ENERGY_PROMILLE,
        "terrain texture should retain broad authored variation"
    );
    assert!(
        near_detail < mid_detail && mid_detail < macro_detail,
        "terrain detail should remain hierarchical rather than white-noise dominated: \
         near={near_detail}, mid={mid_detail}, macro={macro_detail}"
    );
    assert!(
        texture_isolated_edge_promille(&data, TERRAIN_TEXTURE_SIZE)
            <= ISLAND_TERRAIN_TEXTURE_MAX_ISOLATED_EDGE_PROMILLE,
        "terrain texture should reject isolated salt-and-pepper detail"
    );
}

#[test]
fn terrain_biome_palettes_cover_unique_authored_signatures() {
    let profile_count = nau_engine::world::island_art_directions().len();
    let palette_keys = (0..profile_count)
        .map(|index| {
            let palette = terrain_biome_palette(index);
            [
                palette.grass,
                palette.moss,
                palette.meadow,
                palette.clay,
                palette.rock,
            ]
            .map(|color| {
                [
                    (color.x * 255.0).round() as u8,
                    (color.y * 255.0).round() as u8,
                    (color.z * 255.0).round() as u8,
                ]
            })
        })
        .collect::<HashSet<_>>();

    assert_eq!(
        palette_keys.len(),
        profile_count,
        "every authored island should retain a distinct terrain palette signature"
    );
}

#[test]
fn terrain_vertex_colors_use_biome_palette_variation() {
    let profile_count = nau_engine::world::island_art_directions().len();
    let color_keys = (0..profile_count)
        .map(|index| {
            let color = island_terrain_vertex_color(index, 0.56, 1.2, 0.24);
            [
                (color[0] * 255.0).round() as u8,
                (color[1] * 255.0).round() as u8,
                (color[2] * 255.0).round() as u8,
            ]
        })
        .collect::<HashSet<_>>();

    assert_eq!(
        color_keys.len(),
        profile_count,
        "same-region terrain samples should preserve every authored palette signature"
    );
}

#[test]
fn every_terrain_palette_preserves_broad_vertex_value_separation() {
    let profile_count = nau_engine::world::island_art_directions().len();
    let min_luma_span = (0..profile_count)
        .map(|index| {
            let mesh = island_terrain_mesh(index, test_island());
            let mut luma_values = colors(&mesh)
                .iter()
                .map(|color| color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722)
                .collect::<Vec<_>>();
            luma_values.sort_by(f32::total_cmp);
            luma_values[luma_values.len() * 9 / 10] - luma_values[luma_values.len() / 10]
        })
        .fold(f32::INFINITY, f32::min);

    assert!(
        min_luma_span >= 0.10,
        "every terrain palette should retain readable broad value regions: {min_luma_span}"
    );
}

#[test]
fn biome_detail_color_sets_vary_vegetation_and_stone_hues() {
    let profile_count = nau_engine::world::island_art_directions().len();
    let foliage_keys = (0..profile_count)
        .map(|index| biome_detail_color_set(index).foliage_primary)
        .collect::<HashSet<_>>();
    let stone_keys = (0..profile_count)
        .map(|index| biome_detail_color_set(index).stone_primary)
        .collect::<HashSet<_>>();

    assert_eq!(
        foliage_keys.len(),
        profile_count,
        "generated tree canopies should inherit per-island biome identity"
    );
    assert_eq!(
        stone_keys.len(),
        profile_count,
        "stone scatter should vary with the island biome instead of sharing one material"
    );
}

#[test]
fn runtime_spawned_island_materials_preserve_bounded_authored_palette_families() {
    let target_island_index = TERRAIN_BIOME_PALETTE_COUNT;
    let (mut app, output_dir) = setup_headless_gallery_runtime_app(target_island_index);
    let mut foliage_ids = HashSet::new();
    let mut stone_ids = HashSet::new();
    let mut hero_ids = HashSet::new();
    let mut family_material_ids = HashMap::new();

    {
        let world = app.world();
        let route = world.resource::<SkyRoute>();
        let catalog = world.resource::<IslandVisualCatalog>();
        let profiles = nau_engine::world::island_art_directions();
        let palette_family_count = profiles
            .iter()
            .map(|profile| profile.palette_family)
            .collect::<HashSet<_>>()
            .len();
        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let canopy_name = island_tree_specs(island_index, island)
                .first()
                .expect("every island should spawn at least one tree")
                .species
                .canopy_visual_name();
            let hero_name = island_hero_landmark_spec(island_index, island)
                .expect("every island should spawn one hero landmark")
                .label;
            let foliage = catalog
                .entry_material_handle(island.name, canopy_name)
                .expect("queued canopy should retain a runtime material");
            let stone = catalog
                .entry_material_handle(island.name, "island stone scatter")
                .expect("queued stone scatter should retain a runtime material");
            let hero = catalog
                .entry_material_handle(island.name, hero_name)
                .expect("queued hero should retain a runtime material");

            foliage_ids.insert(foliage.id());
            stone_ids.insert(stone.id());
            hero_ids.insert(hero.id());
            let material_ids = (foliage.id(), stone.id(), hero.id());
            if let Some(expected_ids) =
                family_material_ids.insert(profiles[island_index].palette_family, material_ids)
            {
                assert_eq!(
                    material_ids, expected_ids,
                    "{} should reuse its authored palette-family materials",
                    island.name
                );
            }
            assert_eq!(
                hero, stone,
                "{} hero should use its island stone",
                island.name
            );
        }
        assert_eq!(family_material_ids.len(), palette_family_count);
        assert_eq!(foliage_ids.len(), palette_family_count);
        assert_eq!(stone_ids.len(), palette_family_count);
        assert_eq!(hero_ids.len(), palette_family_count);
        let standard_material_count = world.resource::<Assets<StandardMaterial>>().len();
        let surface_material_count = world.resource::<Assets<SurfaceMaterial>>().len();
        assert_eq!(standard_material_count, 51);
        assert_eq!(surface_material_count, 7);
        assert_eq!(
            standard_material_count + surface_material_count,
            58,
            "headless runtime material growth must remain explicit and budgeted"
        );
    }

    let (target_island, canopy_name, hero_name, catalog_foliage, catalog_stone, catalog_hero) = {
        let world = app.world();
        let route = world.resource::<SkyRoute>();
        let island = route.islands()[target_island_index];
        let canopy_name = island_tree_specs(target_island_index, island)
            .first()
            .expect("target island should spawn at least one tree")
            .species
            .canopy_visual_name();
        let hero_name = island_hero_landmark_spec(target_island_index, island)
            .expect("target island should spawn a hero landmark")
            .label;
        let catalog = world.resource::<IslandVisualCatalog>();
        (
            island,
            canopy_name,
            hero_name,
            catalog
                .entry_material_handle(island.name, canopy_name)
                .expect("target canopy material"),
            catalog
                .entry_material_handle(island.name, "island stone scatter")
                .expect("target stone material"),
            catalog
                .entry_material_handle(island.name, hero_name)
                .expect("target hero material"),
        )
    };

    let spawned_foliage = IslandVisualCatalog::spawned_entry_material_handle(
        app.world(),
        target_island.name,
        canopy_name,
    )
    .expect("target canopy should be spawned for the near gallery view");
    let spawned_stone = IslandVisualCatalog::spawned_entry_material_handle(
        app.world(),
        target_island.name,
        "island stone scatter",
    )
    .expect("target stone should be spawned for the near gallery view");
    let spawned_hero = IslandVisualCatalog::spawned_entry_material_handle(
        app.world(),
        target_island.name,
        hero_name,
    )
    .expect("target hero should be spawned for the near gallery view");
    assert_eq!(spawned_foliage, catalog_foliage);
    assert_eq!(spawned_stone, catalog_stone);
    assert_eq!(spawned_hero, catalog_hero);
    assert_eq!(spawned_hero, spawned_stone);
    assert_ne!(spawned_foliage, spawned_stone);

    let obstruction_stone = {
        let expected_name = format!("{} obstruction spire", target_island.name);
        let world = app.world_mut();
        let mut query = world.query::<(&Name, &MeshMaterial3d<StandardMaterial>)>();
        query
            .iter(world)
            .find(|(name, _)| name.as_str() == expected_name)
            .map(|(_, material)| material.0.clone())
            .expect("target obstruction spire should be spawned")
    };
    assert_eq!(obstruction_stone, spawned_stone);

    let expected_colors = biome_detail_color_set(target_island_index);
    assert!(material_texture_contains_color(
        app.world(),
        &spawned_foliage,
        expected_colors.foliage_primary,
    ));
    assert!(material_texture_contains_color(
        app.world(),
        &spawned_stone,
        expected_colors.stone_primary,
    ));

    drop(app);
    remove_existing_dir(&output_dir).expect("gallery material test dir should be removable");
}

#[test]
fn authored_terrain_palettes_stay_in_readable_ranges() {
    for index in 0..nau_engine::world::island_art_directions().len() {
        let palette = terrain_biome_palette(index);
        for color in [
            palette.grass,
            palette.moss,
            palette.meadow,
            palette.clay,
            palette.rock,
        ]
        .into_iter()
        .chain(palette.region_tints)
        {
            assert!(color.is_finite());
            assert!(color.cmpge(Vec3::splat(0.08)).all());
            assert!(color.cmple(Vec3::splat(0.92)).all());
        }

        let luminance = |color: Vec3| color.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        assert!(
            luminance(palette.meadow) > luminance(palette.moss) + 0.04,
            "island {index} should keep meadow and moss values visually separable"
        );
    }
}
