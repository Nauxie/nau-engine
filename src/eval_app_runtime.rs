use crate::authored_assets::VisualAssetDiagnostics;
use crate::camera_runtime::{CameraDiagnostics, CameraFollowFilter};
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::content_export::{
    terrain_export_json_number, terrain_export_json_string, terrain_export_json_vec3,
};
use crate::environment_visuals::{WeatherDrift, WindResponsiveVisual};
use crate::eval_runtime::{EvalCheckpointCapture, EvalRun, path_string};
use crate::generated_content::island_visual_surface_position;
use crate::island_visuals::IslandStreamDiagnostics;
use crate::power_up_runtime::PowerUpCollectionState;
use crate::{Player, RouteObjectiveTracker};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use nau_engine::environment::{AERIAL_POWER_UP_ROUTE, LiftField, WindField};
use nau_engine::eval::EvalScenario;
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::{RouteObjectiveKind, SkyIsland, SkyRoute};
use std::{collections::HashSet, fs, path::Path};

const EVAL_SCREENSHOT_TIMEOUT_FRAMES: u32 = 180;

#[derive(SystemParam)]
pub(crate) struct EvalScene<'w, 's> {
    pub(crate) route: Res<'w, SkyRoute>,
    pub(crate) player: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static Velocity,
            &'static FlightController,
        ),
        With<Player>,
    >,
    pub(crate) camera: Query<'w, 's, &'static Transform, CameraFollowFilter>,
    pub(crate) camera_projection:
        Query<'w, 's, (&'static Camera, &'static GlobalTransform), CameraFollowFilter>,
    pub(crate) camera_diagnostics: Res<'w, CameraDiagnostics>,
    pub(crate) stream_diagnostics: Res<'w, IslandStreamDiagnostics>,
    pub(crate) content_diagnostics: Res<'w, IslandContentDiagnostics>,
    pub(crate) asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    pub(crate) route_objectives: Res<'w, RouteObjectiveTracker>,
    pub(crate) power_ups: Res<'w, PowerUpCollectionState>,
    pub(crate) wind_fields: Query<'w, 's, &'static WindField>,
    pub(crate) lift_fields: Query<'w, 's, &'static LiftField>,
    pub(crate) weather_clouds: Query<'w, 's, &'static Transform, With<WeatherDrift>>,
    pub(crate) wind_responsive_visuals:
        Query<'w, 's, (&'static WindResponsiveVisual, &'static Transform)>,
    pub(crate) all_entities: Query<'w, 's, Entity>,
}

pub(crate) fn finish_eval_frame(
    mut commands: Commands,
    mut run: ResMut<EvalRun>,
    scene: EvalScene,
    mut app_exit: MessageWriter<AppExit>,
) {
    if let Some(error) = run.io_error.clone() {
        eprintln!("{error}");
        run.finalized = true;
        app_exit.write(AppExit::error());
        return;
    }

    if run.finalized {
        if let Some(exit_success) = run.pending_screenshot_exit_success {
            if run
                .screenshot_path
                .as_deref()
                .is_some_and(screenshot_file_ready)
            {
                run.pending_screenshot_exit_success = None;
                let exit = if exit_success {
                    AppExit::Success
                } else {
                    AppExit::error()
                };
                app_exit.write(exit);
                return;
            }

            run.screenshot_wait_frames += 1;
            if run.screenshot_wait_frames > EVAL_SCREENSHOT_TIMEOUT_FRAMES {
                run.pending_screenshot_exit_success = None;
                eprintln!(
                    "eval screenshot did not finish within {} frames",
                    EVAL_SCREENSHOT_TIMEOUT_FRAMES
                );
                app_exit.write(AppExit::error());
            }
        }
        return;
    }

    if let Err(error) = capture_due_checkpoint_screenshots(&mut commands, &mut run, &scene) {
        run.io_error = Some(format!(
            "failed to write checkpoint marker metadata: {error}"
        ));
        return;
    }

    if run.frame < run.scenario.frame_count {
        run.frame += 1;
        return;
    }

    let passed = match run.write_summary() {
        Ok(passed) => passed,
        Err(error) => {
            eprintln!("failed to write eval summary: {error}");
            run.finalized = true;
            app_exit.write(AppExit::error());
            return;
        }
    };

    run.finalized = true;
    eprintln!("eval summary: {}", path_string(&run.summary_path));

    if let Some(screenshot_path) = run.screenshot_path.clone() {
        run.screenshot_wait_frames = 0;
        run.pending_screenshot_exit_success = Some(passed);
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>| {
                save_to_disk(screenshot_path.clone())(captured);
            },
        );
    } else if passed {
        app_exit.write(AppExit::Success);
    } else {
        app_exit.write(AppExit::error());
    }
}

fn screenshot_file_ready(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if metadata.len() == 0 {
        return false;
    }

    image::ImageReader::open(path)
        .and_then(|reader| reader.with_guessed_format())
        .ok()
        .and_then(|reader| reader.decode().ok())
        .is_some_and(|image| image.width() > 0 && image.height() > 0)
}

fn capture_due_checkpoint_screenshots(
    commands: &mut Commands,
    run: &mut EvalRun,
    scene: &EvalScene,
) -> std::io::Result<()> {
    let frame = run.frame;
    let scenario = run.scenario;
    for checkpoint in run
        .checkpoint_captures
        .iter_mut()
        .filter(|checkpoint| !checkpoint.captured && checkpoint.frame == frame)
    {
        if !checkpoint.marker_metadata_written {
            write_checkpoint_marker_metadata(
                &checkpoint.marker_metadata_path,
                scenario,
                checkpoint,
                scene,
            )?;
            checkpoint.marker_metadata_written = true;
        }
        let screenshot_path = checkpoint.path.clone();
        checkpoint.captured = true;
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>| {
                save_to_disk(screenshot_path.clone())(captured);
            },
        );
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct SemanticRouteMarker {
    kind: &'static str,
    label: &'static str,
    world_position: Vec3,
    current_objective: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticMarkerVisibility {
    Visible,
    Occluded,
    Offscreen,
    BehindCamera,
}

impl SemanticMarkerVisibility {
    fn label(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Occluded => "occluded",
            Self::Offscreen => "offscreen",
            Self::BehindCamera => "behind_camera",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SemanticMarkerOcclusion {
    pub(crate) island_name: &'static str,
    pub(crate) distance_m: f32,
}

#[derive(Clone, Copy, Debug)]
struct SemanticSceneSample {
    kind: &'static str,
    label: &'static str,
    expected_material: &'static str,
    world_position: Vec3,
}

fn write_checkpoint_marker_metadata(
    path: &Path,
    scenario: EvalScenario,
    checkpoint: &EvalCheckpointCapture,
    scene: &EvalScene,
) -> std::io::Result<()> {
    let markers = semantic_route_markers(scene);
    let scene_samples = semantic_scene_samples(scene);
    let expected_objective_count = scene
        .route
        .route_objectives(scenario.target_island_name)
        .len();
    let (
        viewport_size,
        marker_json,
        visible_count,
        in_viewport_marker_count,
        occluded_marker_count,
        current_objective_visible,
        scene_sample_json,
        in_viewport_scene_sample_count,
        occluded_scene_sample_count,
        visible_scene_sample_count,
        visible_scene_material_count,
    ) = match scene.camera_projection.single() {
        Ok((camera, camera_transform)) => {
            let (
                viewport_size,
                marker_json,
                visible_count,
                in_viewport_marker_count,
                occluded_marker_count,
                current_objective_visible,
            ) = checkpoint_marker_projection_json(
                camera,
                camera_transform,
                &markers,
                scene.route.islands(),
            );
            let (
                scene_sample_json,
                visible_scene_sample_count,
                in_viewport_scene_sample_count,
                occluded_scene_sample_count,
                visible_scene_material_count,
            ) = checkpoint_scene_sample_projection_json(
                camera,
                camera_transform,
                &scene_samples,
                scene.route.islands(),
            );
            (
                viewport_size,
                marker_json,
                visible_count,
                in_viewport_marker_count,
                occluded_marker_count,
                current_objective_visible,
                scene_sample_json,
                in_viewport_scene_sample_count,
                occluded_scene_sample_count,
                visible_scene_sample_count,
                visible_scene_material_count,
            )
        }
        Err(_) => (None, Vec::new(), 0, 0, 0, false, Vec::new(), 0, 0, 0, 0),
    };
    let viewport_json = viewport_size
        .map(|size| {
            format!(
                "{{\"width\": {}, \"height\": {}}}",
                terrain_export_json_number(size.x),
                terrain_export_json_number(size.y)
            )
        })
        .unwrap_or_else(|| "null".to_string());
    let passed = markers.len() >= expected_objective_count
        && visible_count > 0
        && scene_samples.len() >= 4
        && visible_scene_sample_count > 0
        && viewport_size.is_some();
    let target_island = scenario
        .target_island_name
        .map(terrain_export_json_string)
        .unwrap_or_else(|| "null".to_string());
    let json = format!(
        "{{\n  \"passed\": {},\n  \"scenario\": {},\n  \"target_island\": {},\n  \"frame\": {},\n  \"checkpoint\": {},\n  \"screenshot\": {},\n  \"viewport\": {},\n  \"semantic_marker_count\": {},\n  \"expected_objective_marker_count\": {},\n  \"in_viewport_semantic_marker_count\": {},\n  \"occluded_semantic_marker_count\": {},\n  \"visible_semantic_marker_count\": {},\n  \"current_objective_visible\": {},\n  \"semantic_scene_sample_count\": {},\n  \"in_viewport_semantic_scene_sample_count\": {},\n  \"occluded_semantic_scene_sample_count\": {},\n  \"visible_semantic_scene_sample_count\": {},\n  \"visible_semantic_scene_material_count\": {},\n  \"markers\": [\n{}\n  ],\n  \"scene_samples\": [\n{}\n  ]\n}}\n",
        passed,
        terrain_export_json_string(scenario.name),
        target_island,
        checkpoint.frame,
        terrain_export_json_string(checkpoint.name),
        terrain_export_json_string(&path_string(&checkpoint.path)),
        viewport_json,
        markers.len(),
        expected_objective_count,
        in_viewport_marker_count,
        occluded_marker_count,
        visible_count,
        current_objective_visible,
        scene_samples.len(),
        in_viewport_scene_sample_count,
        occluded_scene_sample_count,
        visible_scene_sample_count,
        visible_scene_material_count,
        marker_json
            .into_iter()
            .map(|entry| format!("    {entry}"))
            .collect::<Vec<_>>()
            .join(",\n"),
        scene_sample_json
            .into_iter()
            .map(|entry| format!("    {entry}"))
            .collect::<Vec<_>>()
            .join(",\n"),
    );

    fs::write(path, json)
}

fn semantic_route_markers(scene: &EvalScene) -> Vec<SemanticRouteMarker> {
    let mut markers = Vec::new();
    let current_label =
        (!scene.route_objectives.complete).then_some(scene.route_objectives.current_label);

    for objective in scene
        .route
        .route_objectives(scene.route_objectives.target_island_name)
    {
        let kind = match objective.kind {
            RouteObjectiveKind::FlyThrough => "objective_updraft",
            RouteObjectiveKind::Land => "objective_landing",
        };
        markers.push(SemanticRouteMarker {
            kind,
            label: objective.label,
            world_position: objective.position,
            current_objective: current_label == Some(objective.label),
        });
    }

    for (island_index, island) in scene.route.islands().iter().copied().enumerate() {
        if island.is_target {
            let ring_size = 8.0;
            for offset in [
                Vec3::new(0.0, 0.05, ring_size * 0.5),
                Vec3::new(0.0, 0.05, -ring_size * 0.5),
                Vec3::new(ring_size * 0.5, 0.05, 0.0),
                Vec3::new(-ring_size * 0.5, 0.05, 0.0),
            ] {
                let surface_y =
                    island.mesh_top_y_at(island.center + Vec3::new(offset.x, 0.0, offset.z));
                markers.push(SemanticRouteMarker {
                    kind: "landing_marker",
                    label: island.name,
                    world_position: Vec3::new(
                        island.center.x + offset.x,
                        surface_y + offset.y,
                        island.center.z + offset.z,
                    ),
                    current_objective: current_label == Some(island.name),
                });
            }
        } else if island.name == "launch mesa" {
            markers.push(SemanticRouteMarker {
                kind: "launch_beacon",
                label: island.name,
                world_position: island_visual_surface_position(island, Vec2::new(-0.42, 0.38))
                    + Vec3::Y * 1.6,
                current_objective: false,
            });
        } else {
            let beacon_height = 3.8 + (island_index % 3) as f32 * 0.7;
            markers.push(SemanticRouteMarker {
                kind: "route_cairn",
                label: island.name,
                world_position: island_visual_surface_position(island, Vec2::new(-0.18, 0.22))
                    + Vec3::Y * (beacon_height * 0.5),
                current_objective: false,
            });
        }
    }

    for power_up in AERIAL_POWER_UP_ROUTE {
        if scene.power_ups.is_collected(power_up) {
            continue;
        }
        markers.push(SemanticRouteMarker {
            kind: "aerial_power_up",
            label: power_up.name,
            world_position: power_up.center,
            current_objective: false,
        });
    }

    markers
}

fn semantic_scene_samples(scene: &EvalScene) -> Vec<SemanticSceneSample> {
    let mut samples = Vec::new();

    for (island_index, island) in scene.route.islands().iter().copied().enumerate() {
        samples.push(SemanticSceneSample {
            kind: "terrain_surface",
            label: island.name,
            expected_material: "terrain",
            world_position: island_visual_surface_position(island, Vec2::new(0.16, -0.14))
                + Vec3::Y * 0.08,
        });
        samples.push(SemanticSceneSample {
            kind: "distant_island",
            label: island.name,
            expected_material: "distant_island",
            world_position: island_visual_surface_position(island, Vec2::new(0.0, 0.0))
                + Vec3::Y * 1.2,
        });

        for (sample_index, canopy_position) in tree_canopy_sample_positions(island_index, island)
            .into_iter()
            .enumerate()
        {
            if sample_index == 1 && island.is_target {
                continue;
            }
            samples.push(SemanticSceneSample {
                kind: "tree_canopy",
                label: island.name,
                expected_material: "foliage",
                world_position: canopy_position,
            });
        }
    }

    for cloud_transform in scene.weather_clouds.iter().take(18) {
        samples.push(SemanticSceneSample {
            kind: "weather_cloud",
            label: "weather cloud",
            expected_material: "cloud",
            world_position: cloud_transform.translation,
        });
    }

    samples
}

fn tree_canopy_sample_positions(island_index: usize, island: SkyIsland) -> Vec<Vec3> {
    if island.name == "launch mesa" {
        let launch_tree_height = 4.4;
        let launch_tree_surface_y =
            island.mesh_top_y_at(Vec3::new(island.center.x, island.center.y, 8.0));
        return vec![Vec3::new(
            island.center.x,
            launch_tree_surface_y + launch_tree_height + 0.85,
            8.0,
        )];
    }

    let detail_phase = island_index as f32 * 0.77;
    [
        Vec2::new(-0.42, -0.24),
        Vec2::new(0.34, -0.36),
        Vec2::new(0.24, 0.32),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, offset)| {
        let sway = (detail_phase + index as f32).sin() * 0.08;
        let surface = island_visual_surface_position(island, Vec2::new(offset.x + sway, offset.y));
        let trunk_height = 2.1 + index as f32 * 0.25;
        surface + Vec3::Y * (trunk_height + 0.72)
    })
    .collect()
}

fn checkpoint_marker_projection_json(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    markers: &[SemanticRouteMarker],
    islands: &[SkyIsland],
) -> (Option<Vec2>, Vec<String>, usize, usize, usize, bool) {
    let viewport_size = camera.logical_viewport_size();
    let camera_position = camera_transform.translation();
    let mut visible_count = 0usize;
    let mut in_viewport_count = 0usize;
    let mut occluded_count = 0usize;
    let mut current_objective_visible = false;
    let entries = markers
        .iter()
        .map(|marker| {
            let projected = camera
                .world_to_viewport_with_depth(camera_transform, marker.world_position)
                .ok();
            let in_viewport = projected
                .zip(viewport_size)
                .is_some_and(|(screen, viewport)| {
                    screen.x >= 0.0
                        && screen.y >= 0.0
                        && screen.x <= viewport.x
                        && screen.y <= viewport.y
                        && screen.z.is_finite()
                        && screen.z > 0.0
                });
            let behind_camera = projected.is_some_and(|screen| {
                screen.z.is_finite() && screen.z <= 0.0
            });
            let occlusion = in_viewport
                .then(|| marker_occlusion_between(camera_position, marker.world_position, islands))
                .flatten();
            let visibility = if behind_camera {
                SemanticMarkerVisibility::BehindCamera
            } else if !in_viewport {
                SemanticMarkerVisibility::Offscreen
            } else if occlusion.is_some() {
                SemanticMarkerVisibility::Occluded
            } else {
                SemanticMarkerVisibility::Visible
            };
            if in_viewport {
                in_viewport_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Occluded {
                occluded_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Visible {
                visible_count += 1;
                current_objective_visible |= marker.current_objective;
            }

            let screen_json = projected
                .map(|screen| {
                    format!(
                        "{{\"x\": {}, \"y\": {}, \"depth_m\": {}}}",
                        terrain_export_json_number(screen.x),
                        terrain_export_json_number(screen.y),
                        terrain_export_json_number(screen.z)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let occluder_json = occlusion
                .map(|occlusion| {
                    format!(
                        "{{\"kind\": \"sky_island\", \"label\": {}, \"distance_m\": {}}}",
                        terrain_export_json_string(occlusion.island_name),
                        terrain_export_json_number(occlusion.distance_m)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let camera_distance_m = marker.world_position.distance(camera_position);

            format!(
                "{{\"kind\": {}, \"label\": {}, \"current_objective\": {}, \"world\": {}, \"screen\": {}, \"in_viewport\": {}, \"visibility\": {}, \"occluder\": {}, \"camera_distance_m\": {}}}",
                terrain_export_json_string(marker.kind),
                terrain_export_json_string(marker.label),
                marker.current_objective,
                terrain_export_json_vec3(marker.world_position),
                screen_json,
                in_viewport,
                terrain_export_json_string(visibility.label()),
                occluder_json,
                terrain_export_json_number(camera_distance_m)
            )
        })
        .collect();

    (
        viewport_size,
        entries,
        visible_count,
        in_viewport_count,
        occluded_count,
        current_objective_visible,
    )
}

pub(crate) fn marker_occlusion_between(
    camera_position: Vec3,
    marker_position: Vec3,
    islands: &[SkyIsland],
) -> Option<SemanticMarkerOcclusion> {
    let mut nearest = None;
    for island in islands {
        let Some(distance_m) =
            island_segment_occlusion_distance(camera_position, marker_position, *island)
        else {
            continue;
        };
        if nearest
            .as_ref()
            .is_none_or(|occlusion: &SemanticMarkerOcclusion| distance_m < occlusion.distance_m)
        {
            nearest = Some(SemanticMarkerOcclusion {
                island_name: island.name,
                distance_m,
            });
        }
    }
    nearest
}

fn island_segment_occlusion_distance(
    camera_position: Vec3,
    marker_position: Vec3,
    island: SkyIsland,
) -> Option<f32> {
    let segment = marker_position - camera_position;
    let length = segment.length();
    if length <= 0.01 {
        return None;
    }
    let direction = segment / length;
    let max_distance = length - 2.0;
    if max_distance <= 1.0 {
        return None;
    }
    let steps = ((length / 6.0).ceil() as usize).clamp(12, 96);

    for step in 1..steps {
        let distance_m = length * step as f32 / steps as f32;
        if distance_m >= max_distance {
            break;
        }
        let point = camera_position + direction * distance_m;
        if island_blocks_marker_ray(island, point) {
            return Some(distance_m);
        }
    }

    None
}

fn island_blocks_marker_ray(island: SkyIsland, point: Vec3) -> bool {
    let dx = (point.x - island.center.x) / island.half_extents.x.max(0.001);
    let dz = (point.z - island.center.z) / island.half_extents.y.max(0.001);
    if dx * dx + dz * dz > 1.10 {
        return false;
    }

    let top_y = island.mesh_top_y_at(point) + 0.9;
    let bottom_y = island.center.y - island.thickness * 1.15;
    point.y >= bottom_y && point.y <= top_y
}

fn checkpoint_scene_sample_projection_json(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    samples: &[SemanticSceneSample],
    scene_islands: &[SkyIsland],
) -> (Vec<String>, usize, usize, usize, usize) {
    let viewport_size = camera.logical_viewport_size();
    let camera_position = camera_transform.translation();
    let mut visible_count = 0usize;
    let mut in_viewport_count = 0usize;
    let mut occluded_count = 0usize;
    let mut visible_materials = HashSet::new();
    let entries = samples
        .iter()
        .map(|sample| {
            let projected = camera
                .world_to_viewport_with_depth(camera_transform, sample.world_position)
                .ok();
            let in_viewport = projected
                .zip(viewport_size)
                .is_some_and(|(screen, viewport)| {
                    screen.x >= 0.0
                        && screen.y >= 0.0
                        && screen.x <= viewport.x
                        && screen.y <= viewport.y
                        && screen.z.is_finite()
                        && screen.z > 0.0
                });
            let behind_camera = projected.is_some_and(|screen| {
                screen.z.is_finite() && screen.z <= 0.0
            });
            let occlusion = in_viewport
                .then(|| {
                    marker_occlusion_between(camera_position, sample.world_position, scene_islands)
                })
                .flatten();
            let visibility = if behind_camera {
                SemanticMarkerVisibility::BehindCamera
            } else if !in_viewport {
                SemanticMarkerVisibility::Offscreen
            } else if occlusion.is_some() {
                SemanticMarkerVisibility::Occluded
            } else {
                SemanticMarkerVisibility::Visible
            };
            if in_viewport {
                in_viewport_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Occluded {
                occluded_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Visible {
                visible_count += 1;
                visible_materials.insert(sample.expected_material);
            }

            let screen_json = projected
                .map(|screen| {
                    format!(
                        "{{\"x\": {}, \"y\": {}, \"depth_m\": {}}}",
                        terrain_export_json_number(screen.x),
                        terrain_export_json_number(screen.y),
                        terrain_export_json_number(screen.z)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let occluder_json = occlusion
                .map(|occlusion| {
                    format!(
                        "{{\"kind\": \"sky_island\", \"label\": {}, \"distance_m\": {}}}",
                        terrain_export_json_string(occlusion.island_name),
                        terrain_export_json_number(occlusion.distance_m)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let camera_distance_m = sample.world_position.distance(camera_position);

            format!(
                "{{\"kind\": {}, \"label\": {}, \"expected_material\": {}, \"world\": {}, \"screen\": {}, \"in_viewport\": {}, \"visibility\": {}, \"occluder\": {}, \"camera_distance_m\": {}}}",
                terrain_export_json_string(sample.kind),
                terrain_export_json_string(sample.label),
                terrain_export_json_string(sample.expected_material),
                terrain_export_json_vec3(sample.world_position),
                screen_json,
                in_viewport,
                terrain_export_json_string(visibility.label()),
                occluder_json,
                terrain_export_json_number(camera_distance_m)
            )
        })
        .collect();

    (
        entries,
        visible_count,
        in_viewport_count,
        occluded_count,
        visible_materials.len(),
    )
}
