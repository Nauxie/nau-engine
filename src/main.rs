mod authored_assets;
mod camera_runtime;
mod content_diagnostics;
mod content_export;
mod debug_readout_runtime;
mod debug_visuals;
mod environment_visuals;
mod eval_app_runtime;
mod eval_runtime;
mod game_ui_runtime;
mod generated_content;
mod island_visuals;
mod play_profile_runtime;
mod player_runtime;
mod power_up_runtime;
mod scene_setup_runtime;
mod world_collision_runtime;
mod world_floor_runtime;
use authored_assets::*;
use bevy::app::AnimationSystems;
use bevy::light::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy::window::{CompositeAlphaMode, PresentMode};
use camera_runtime::*;
#[cfg(test)]
use content_export::mesh_uv0;
use content_export::{
    export_terrain_inspection, export_visual_content_inspection, export_wind_visual_inspection,
};
use debug_readout_runtime::*;
use debug_visuals::*;
use environment_visuals::*;
use eval_app_runtime::*;
use eval_runtime::{
    CliAction, EvalMovementBasis, EvalOptions, EvalRun, ISLAND_HERO_GALLERY, path_string, usage,
};
#[cfg(test)]
use eval_runtime::{RunMode, parse_cli_args, remove_existing_dir};
use game_ui_runtime::*;
#[cfg(test)]
use generated_content::*;
use island_visuals::*;
use nau_engine::camera::{CameraControlState, CameraControlTuning};
#[cfg(test)]
use nau_engine::eval::scenario_named;
#[cfg(test)]
use nau_engine::movement::FlightMode;
use nau_engine::movement::FlightTuning;
#[cfg(test)]
use nau_engine::world::SkyIsland;
use nau_engine::world::SkyRoute;
use play_profile_runtime::{PlayProfileRun, collect_play_profile_sample};
pub(crate) use player_runtime::{
    Player, PlayerDisplacementDiagnostics, RouteObjectiveTracker, WindForceDiagnostics,
    grounded_visual_foot_gap_m, keyboard_flight_input, movement_facing,
};
use player_runtime::{
    animate_character, eval_fly_player, eval_reset_player_to_playtest_position, fly_player,
    reset_player_to_playtest_position, scripted_play_profile_fly_player, update_route_objectives,
};
use player_runtime::{
    apply_authored_glider_pose, apply_authored_player_pose_nodes, reapply_authored_glider_pose,
    reapply_authored_player_pose_nodes,
};
use power_up_runtime::*;
use scene_setup_runtime::{
    INITIAL_SKY_CLEAR_COLOR, WORLD_RADIUS, apply_authored_island_material_parity,
    fix_island_hero_gallery_player, setup,
};
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::PathBuf;
use world_collision_runtime::*;
use world_floor_runtime::*;

fn main() -> AppExit {
    let cli = match CliAction::from_env() {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            return AppExit::from_code(2);
        }
    };

    let (eval, run_mode, play_profile) = match cli {
        CliAction::Run {
            eval,
            mode,
            play_profile,
        } => (eval, mode, play_profile),
        CliAction::ExportTerrain { output_dir } => {
            return match export_terrain_inspection(&output_dir) {
                Ok(report) => {
                    println!(
                        "exported {} islands / {} meshes to {}",
                        report.island_count,
                        report.mesh_count,
                        path_string(&report.manifest_path)
                    );
                    AppExit::Success
                }
                Err(error) => {
                    eprintln!("terrain export failed: {error}");
                    AppExit::from_code(1)
                }
            };
        }
        CliAction::ExportVisualContent { output_dir } => {
            return match export_visual_content_inspection(&output_dir) {
                Ok(report) => {
                    println!(
                        "exported {} visual-content meshes to {}",
                        report.mesh_count,
                        path_string(&report.manifest_path)
                    );
                    AppExit::Success
                }
                Err(error) => {
                    eprintln!("visual content export failed: {error}");
                    AppExit::from_code(1)
                }
            };
        }
        CliAction::ExportWindVisuals { output_dir } => {
            return match export_wind_visual_inspection(&output_dir) {
                Ok(report) => {
                    println!(
                        "exported {} wind visual tracks to {}",
                        report.track_count,
                        path_string(&report.manifest_path)
                    );
                    AppExit::Success
                }
                Err(error) => {
                    eprintln!("wind visual export failed: {error}");
                    AppExit::from_code(1)
                }
            };
        }
        CliAction::Help => {
            println!("{}", usage());
            return AppExit::Success;
        }
    };
    let screenshot_eval = eval
        .as_deref()
        .is_some_and(|options| options.capture_screenshot);
    let scripted_play_profile = play_profile
        .as_ref()
        .is_some_and(|options| options.script.is_some());
    let game_ui_enabled = eval.is_none() && play_profile.is_none();

    if play_profile.is_some() && cfg!(debug_assertions) {
        eprintln!(
            "--play-profile requires a release build; run cargo run --release -- --play --play-profile <file>"
        );
        return AppExit::from_code(2);
    }

    let mut app = App::new();
    app.insert_resource(ClearColor(INITIAL_SKY_CLEAR_COLOR))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.62, 0.68, 0.78),
            brightness: 360.0,
            ..default()
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(FlightTuning::default())
        .insert_resource(CameraControlTuning::default())
        .insert_resource(CameraControlState::default())
        .insert_resource(run_mode)
        .insert_resource(CameraDiagnostics::default())
        .insert_resource(CinematicWeather::new(WORLD_RADIUS))
        .insert_resource(VisualAssetDiagnostics::default())
        .insert_resource(AuthoredAnimationDiagnostics::default())
        .insert_resource(IslandStreamDiagnostics::default())
        .insert_resource(RouteObjectiveTracker::default())
        .insert_resource(PowerUpCollectionState::default())
        .insert_resource(PlayerDisplacementDiagnostics::default())
        .insert_resource(WorldCollisionDiagnostics::default())
        .insert_resource(WorldFloorDiagnostics::default())
        .insert_resource(WindForceDiagnostics::default())
        .insert_resource(MouseLookState::default())
        .insert_resource(GameUiState::new(game_ui_enabled))
        .insert_resource(DebugVisuals::for_run_mode(run_mode, screenshot_eval))
        .insert_resource(SkyRoute::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window(eval.as_deref())),
            ..default()
        }))
        .configure_sets(
            Update,
            (
                GameSet::Ui,
                GameSet::CameraInput,
                GameSet::Movement.run_if(gameplay_input_active),
                GameSet::Camera.run_if(gameplay_input_active),
                GameSet::Diagnostics.run_if(gameplay_input_active),
                GameSet::Eval,
            )
                .chain(),
        )
        .add_systems(
            Startup,
            (setup, apply_authored_island_material_parity).chain(),
        )
        .add_systems(
            Update,
            (toggle_game_menu, handle_game_menu_buttons, sync_game_ui)
                .chain()
                .in_set(GameSet::Ui),
        )
        .add_systems(
            Update,
            (
                update_mouse_look_capture.run_if(gameplay_input_active),
                update_camera_control,
            )
                .chain()
                .in_set(GameSet::CameraInput),
        )
        .add_systems(
            Update,
            (
                animate_character,
                link_ready_authored_animations,
                tag_authored_player_pose_nodes,
                update_authored_player_animation,
                apply_authored_player_pose_nodes,
                apply_authored_glider_pose,
                update_player_airflow_visuals,
                follow_camera,
                direct_plateau_vista_camera,
            )
                .chain()
                .in_set(GameSet::Camera),
        )
        .add_systems(
            PostUpdate,
            (
                reapply_authored_player_pose_nodes,
                reapply_authored_glider_pose,
            )
                .after(AnimationSystems)
                .before(TransformSystems::Propagate),
        )
        .add_systems(
            Update,
            (
                update_island_stream_visibility,
                update_world_floor_streaming,
                update_cinematic_weather,
                update_weather_drift,
                update_wind_responsive_visuals,
                update_updraft_columns,
                update_updraft_guides,
                update_updraft_ribbons,
                update_crosswind_guides,
                update_crosswind_ribbons,
                update_power_up_guides,
                update_route_objectives,
                update_visual_asset_diagnostics,
                update_debug_readout,
                draw_debug_gizmos,
            )
                .chain()
                .in_set(GameSet::Diagnostics),
        );

    if let Some(play_profile_options) = play_profile {
        let play_profile_run = match PlayProfileRun::new(
            play_profile_options.output_path,
            play_profile_options.duration_secs,
            play_profile_options.script,
        ) {
            Ok(play_profile_run) => play_profile_run,
            Err(error) => {
                eprintln!("failed to prepare play profile output: {error}");
                return AppExit::from_code(2);
            }
        };

        app.insert_resource(play_profile_run)
            .add_systems(Update, collect_play_profile_sample.in_set(GameSet::Eval));
    }

    if let Some(eval_options) = eval {
        let eval_run = match EvalRun::new(*eval_options) {
            Ok(eval_run) => eval_run,
            Err(error) => {
                eprintln!("failed to prepare eval output: {error}");
                return AppExit::from_code(2);
            }
        };

        app.insert_resource(eval_run)
            .insert_resource(EvalMovementBasis::default())
            .insert_resource(VisiblePoseTemporalState::default())
            .insert_resource(ObservedWindVisualMotionState::default())
            .insert_resource(RuntimeAssetCostState::default())
            .add_systems(
                Update,
                (
                    (eval_reset_player_to_playtest_position, eval_fly_player)
                        .chain()
                        .run_if(eval_gameplay_movement_active),
                    fix_island_hero_gallery_player,
                )
                    .chain()
                    .in_set(GameSet::Movement),
            )
            .add_systems(
                Update,
                (
                    apply_authored_player_pose_nodes,
                    apply_authored_glider_pose,
                    collect_eval_frame_time,
                    collect_eval_metrics,
                    finish_eval_frame,
                )
                    .chain()
                    .in_set(GameSet::Eval),
            );
    } else {
        if scripted_play_profile {
            app.add_systems(
                Update,
                scripted_play_profile_fly_player.in_set(GameSet::Movement),
            );
        } else if run_mode.debug_visual_toggle_enabled() {
            app.add_systems(
                Update,
                (
                    toggle_debug_visuals,
                    reset_player_to_playtest_position,
                    fly_player,
                )
                    .chain()
                    .in_set(GameSet::Movement),
            );
        } else {
            app.add_systems(
                Update,
                (reset_player_to_playtest_position, fly_player)
                    .chain()
                    .in_set(GameSet::Movement),
            );
        }
    }

    app.run()
}

fn eval_gameplay_movement_active(run: Res<EvalRun>) -> bool {
    run.scenario.name != ISLAND_HERO_GALLERY
}

fn primary_window(eval: Option<&EvalOptions>) -> Window {
    let hidden_metric_eval =
        eval.is_some_and(|options| !options.capture_screenshot && !options.visible_window);

    Window {
        title: "The NAU Engine Flight Sandbox".into(),
        resolution: (1280, 720).into(),
        present_mode: PresentMode::AutoVsync,
        composite_alpha_mode: CompositeAlphaMode::Opaque,
        transparent: false,
        visible: !hidden_metric_eval,
        focused: !hidden_metric_eval,
        ..default()
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameSet {
    Ui,
    CameraInput,
    Movement,
    Camera,
    Diagnostics,
    Eval,
}

#[cfg(test)]
mod app_tests;
