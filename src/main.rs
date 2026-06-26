mod authored_assets;
mod camera_runtime;
mod content_diagnostics;
mod content_export;
mod debug_readout_runtime;
mod debug_visuals;
mod environment_visuals;
mod eval_app_runtime;
mod eval_runtime;
mod generated_content;
mod island_visuals;
mod player_runtime;
mod power_up_runtime;
mod scene_setup_runtime;
mod world_collision_runtime;
use authored_assets::*;
use bevy::app::AnimationSystems;
use bevy::light::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy::window::CompositeAlphaMode;
use camera_runtime::*;
#[cfg(test)]
use content_export::mesh_uv0;
use content_export::{export_terrain_inspection, export_visual_content_inspection};
use debug_readout_runtime::*;
use debug_visuals::*;
use environment_visuals::*;
use eval_app_runtime::*;
use eval_runtime::{CliAction, EvalMovementBasis, EvalOptions, EvalRun, path_string, usage};
#[cfg(test)]
use eval_runtime::{parse_cli_args, remove_existing_dir};
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
use player_runtime::apply_authored_player_pose_nodes;
pub(crate) use player_runtime::{
    Player, RouteObjectiveTracker, grounded_visual_foot_gap_m, keyboard_flight_input,
    movement_facing,
};
use player_runtime::{animate_character, eval_fly_player, fly_player, update_route_objectives};
use power_up_runtime::*;
use scene_setup_runtime::{INITIAL_SKY_CLEAR_COLOR, WORLD_RADIUS, setup};
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::PathBuf;
use world_collision_runtime::*;

fn main() -> AppExit {
    let cli = match CliAction::from_env() {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            return AppExit::from_code(2);
        }
    };

    let eval = match cli {
        CliAction::Run { eval } => eval,
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
        CliAction::Help => {
            println!("{}", usage());
            return AppExit::Success;
        }
    };
    let screenshot_eval = eval
        .as_deref()
        .is_some_and(|options| options.capture_screenshot);

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
        .insert_resource(CameraDiagnostics::default())
        .insert_resource(CinematicWeather::new(WORLD_RADIUS))
        .insert_resource(VisualAssetDiagnostics::default())
        .insert_resource(IslandStreamDiagnostics::default())
        .insert_resource(RouteObjectiveTracker::default())
        .insert_resource(PowerUpCollectionState::default())
        .insert_resource(WorldCollisionDiagnostics::default())
        .insert_resource(MouseLookState::default())
        .insert_resource(DebugVisuals {
            enabled: !screenshot_eval,
        })
        .insert_resource(SkyRoute::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window(eval.as_deref())),
            ..default()
        }))
        .configure_sets(
            Update,
            (
                GameSet::Movement,
                GameSet::Camera,
                GameSet::Diagnostics,
                GameSet::Eval,
            )
                .chain(),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_mouse_look_capture,
                update_camera_control,
                animate_character,
                link_ready_authored_animations,
                tag_authored_player_pose_nodes,
                update_authored_player_animation,
                apply_authored_player_pose_nodes,
                update_glider_airflow_trails,
                follow_camera,
            )
                .chain()
                .in_set(GameSet::Camera),
        )
        .add_systems(
            PostUpdate,
            apply_authored_player_pose_nodes
                .after(AnimationSystems)
                .before(TransformSystems::Propagate),
        )
        .add_systems(
            Update,
            (
                update_island_stream_visibility,
                update_cinematic_weather,
                update_weather_drift,
                update_wind_responsive_visuals,
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
            .add_systems(Update, eval_fly_player.in_set(GameSet::Movement))
            .add_systems(
                Update,
                (
                    apply_authored_player_pose_nodes,
                    collect_eval_frame_time,
                    collect_eval_metrics,
                    finish_eval_frame,
                )
                    .chain()
                    .in_set(GameSet::Eval),
            );
    } else {
        app.add_systems(
            Update,
            (toggle_debug_visuals, fly_player).in_set(GameSet::Movement),
        );
    }

    app.run()
}

fn primary_window(eval: Option<&EvalOptions>) -> Window {
    let hidden_metric_eval = eval.is_some_and(|options| !options.capture_screenshot);

    Window {
        title: "The NAU Engine Flight Sandbox".into(),
        resolution: (1280, 720).into(),
        composite_alpha_mode: CompositeAlphaMode::Opaque,
        transparent: false,
        visible: !hidden_metric_eval,
        focused: !hidden_metric_eval,
        ..default()
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameSet {
    Movement,
    Camera,
    Diagnostics,
    Eval,
}

#[cfg(test)]
mod app_tests;
