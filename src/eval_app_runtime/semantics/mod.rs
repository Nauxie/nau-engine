mod markers;
mod occlusion;
mod projection;
mod samples;

use super::scene::EvalScene;
use crate::content_export::{terrain_export_json_number, terrain_export_json_string};
use crate::eval_runtime::{EvalCheckpointCapture, EvalRun, path_string};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use nau_engine::eval::EvalScenario;
use std::{fs, path::Path};

pub(crate) use occlusion::{SemanticMarkerOcclusion, marker_occlusion_between};

pub(super) fn capture_due_checkpoint_screenshots(
    commands: &mut Commands,
    run: &mut EvalRun,
    scene: &EvalScene,
) -> std::io::Result<()> {
    let frame = run.frame;
    let scenario = run.scenario;
    for checkpoint in run.checkpoint_captures.iter_mut().filter(|checkpoint| {
        checkpoint.capture_requested && !checkpoint.captured && checkpoint.frame == frame
    }) {
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

fn write_checkpoint_marker_metadata(
    path: &Path,
    scenario: EvalScenario,
    checkpoint: &EvalCheckpointCapture,
    scene: &EvalScene,
) -> std::io::Result<()> {
    let markers = markers::semantic_route_markers(scene);
    let scene_samples = samples::semantic_scene_samples(scene);
    let target_island_name = checkpoint_target_island(scenario, checkpoint);
    let expected_objective_count = scene.route.route_objectives(target_island_name).len();
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
        visible_wind_scene_sample_count,
    ) = match scene.camera_projection.single() {
        Ok((camera, camera_transform)) => {
            let (
                viewport_size,
                marker_json,
                visible_count,
                in_viewport_marker_count,
                occluded_marker_count,
                current_objective_visible,
            ) = projection::checkpoint_marker_projection_json(
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
                visible_wind_scene_sample_count,
            ) = projection::checkpoint_scene_sample_projection_json(
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
                visible_wind_scene_sample_count,
            )
        }
        Err(_) => (None, Vec::new(), 0, 0, 0, false, Vec::new(), 0, 0, 0, 0, 0),
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
    let route_marker_projection_required =
        checkpoint_requires_route_marker_projection(scenario, &checkpoint.name);
    let streaming_settled = scene.stream_diagnostics.visibility_changes_this_frame == 0
        && scene.stream_diagnostics.spawned_visuals_this_frame == 0
        && scene.stream_diagnostics.despawned_visuals_this_frame == 0;
    let passed = markers.len() >= expected_objective_count
        && (!route_marker_projection_required || visible_count > 0)
        && (!checkpoint_requires_settled_streaming(scenario) || streaming_settled)
        && scene_samples.len() >= 4
        && visible_scene_sample_count > 0
        && (!checkpoint_requires_wind_visual_sample(scenario, &checkpoint.name)
            || visible_wind_scene_sample_count > 0)
        && viewport_size.is_some();
    let target_island = target_island_name
        .map(terrain_export_json_string)
        .unwrap_or_else(|| "null".to_string());
    let target_view = checkpoint
        .target_view
        .map(|view| terrain_export_json_string(view.label()))
        .unwrap_or_else(|| "null".to_string());
    let json = format!(
        "{{\n  \"passed\": {},\n  \"scenario\": {},\n  \"target_island\": {},\n  \"target_view\": {},\n  \"frame\": {},\n  \"checkpoint\": {},\n  \"screenshot\": {},\n  \"viewport\": {},\n  \"route_marker_projection_required\": {},\n  \"streaming_settled\": {},\n  \"stream_visibility_changes_this_frame\": {},\n  \"stream_spawned_visuals_this_frame\": {},\n  \"stream_despawned_visuals_this_frame\": {},\n  \"semantic_marker_count\": {},\n  \"expected_objective_marker_count\": {},\n  \"in_viewport_semantic_marker_count\": {},\n  \"occluded_semantic_marker_count\": {},\n  \"visible_semantic_marker_count\": {},\n  \"current_objective_visible\": {},\n  \"semantic_scene_sample_count\": {},\n  \"in_viewport_semantic_scene_sample_count\": {},\n  \"occluded_semantic_scene_sample_count\": {},\n  \"visible_semantic_scene_sample_count\": {},\n  \"visible_semantic_scene_material_count\": {},\n  \"visible_wind_scene_sample_count\": {},\n  \"markers\": [\n{}\n  ],\n  \"scene_samples\": [\n{}\n  ]\n}}\n",
        passed,
        terrain_export_json_string(scenario.name),
        target_island,
        target_view,
        checkpoint.frame,
        terrain_export_json_string(&checkpoint.name),
        terrain_export_json_string(&path_string(&checkpoint.path)),
        viewport_json,
        route_marker_projection_required,
        streaming_settled,
        scene.stream_diagnostics.visibility_changes_this_frame,
        scene.stream_diagnostics.spawned_visuals_this_frame,
        scene.stream_diagnostics.despawned_visuals_this_frame,
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
        visible_wind_scene_sample_count,
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

fn checkpoint_target_island(
    scenario: EvalScenario,
    checkpoint: &EvalCheckpointCapture,
) -> Option<&'static str> {
    checkpoint
        .target_island_name
        .or(scenario.target_island_name)
}

fn checkpoint_requires_settled_streaming(scenario: EvalScenario) -> bool {
    scenario.name == "island_hero_gallery"
}

fn checkpoint_requires_route_marker_projection(
    scenario: EvalScenario,
    checkpoint_name: &str,
) -> bool {
    !matches!(
        (scenario.name, checkpoint_name),
        ("plateau_arrival_camera", _)
            | ("great_sky_plateau_vistas", _)
            | ("island_surface_review", _)
            | ("island_hero_gallery", _)
            | (
                "great_sky_plateau_route",
                "waterfall_vista" | "plateau_arrival_reveal"
            )
    )
}

fn checkpoint_requires_wind_visual_sample(scenario: EvalScenario, checkpoint_name: &str) -> bool {
    matches!(
        (scenario.name, checkpoint_name),
        ("updraft_route", "updraft_entry" | "high_glide")
            | (
                "island_launch_to_landing",
                "launch_updraft_entry" | "midroute_lift_view"
            )
            | (
                "branch_recovery_route",
                "branch_choice" | "recovery_approach"
            )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::eval::scenario_named;

    #[test]
    fn route_marker_projection_is_optional_at_cinematic_review_views() {
        let plateau_camera =
            scenario_named("plateau_arrival_camera").expect("plateau camera scenario");
        let plateau_route =
            scenario_named("great_sky_plateau_route").expect("plateau route scenario");
        let plateau_vistas =
            scenario_named("great_sky_plateau_vistas").expect("plateau vistas scenario");
        let surface_review =
            scenario_named("island_surface_review").expect("surface review scenario");
        let hero_gallery = scenario_named("island_hero_gallery").expect("hero gallery scenario");
        let updraft = scenario_named("updraft_route").expect("updraft scenario");

        assert!(!checkpoint_requires_route_marker_projection(
            plateau_camera,
            "settled_view"
        ));
        assert!(!checkpoint_requires_route_marker_projection(
            plateau_camera,
            "any_checkpoint"
        ));
        assert!(!checkpoint_requires_route_marker_projection(
            plateau_route,
            "waterfall_vista"
        ));
        assert!(!checkpoint_requires_route_marker_projection(
            plateau_route,
            "plateau_arrival_reveal"
        ));
        assert!(!checkpoint_requires_route_marker_projection(
            plateau_vistas,
            "waterfall_vista"
        ));
        for checkpoint in [
            "ruins_and_rock_detail",
            "dense_flora_detail",
            "lake_river_waterfall_detail",
        ] {
            assert!(!checkpoint_requires_route_marker_projection(
                surface_review,
                checkpoint
            ));
        }
        assert!(!checkpoint_requires_route_marker_projection(
            hero_gallery,
            "island_00_launch_mesa_near"
        ));
        assert!(checkpoint_requires_route_marker_projection(
            plateau_route,
            "plateau_approach"
        ));
        assert!(checkpoint_requires_route_marker_projection(
            updraft,
            "updraft_entry"
        ));
    }

    #[test]
    fn hero_gallery_checkpoints_require_settled_streaming() {
        let hero_gallery = scenario_named("island_hero_gallery").expect("hero gallery scenario");
        let surface_review =
            scenario_named("island_surface_review").expect("surface review scenario");

        assert!(checkpoint_requires_settled_streaming(hero_gallery));
        assert!(!checkpoint_requires_settled_streaming(surface_review));
    }

    #[test]
    fn sidecar_target_prefers_dynamic_checkpoint_island() {
        use std::path::PathBuf;

        let scenario = scenario_named("island_surface_review").expect("static target scenario");
        let mut checkpoint = EvalCheckpointCapture {
            frame: 8,
            name: "island_00_launch_mesa_near".to_string(),
            path: PathBuf::from("capture.png"),
            marker_metadata_path: PathBuf::from("capture.markers.json"),
            capture_requested: true,
            target_island_name: Some("launch mesa"),
            target_view: None,
            island_index: Some(0),
            pose: None,
            captured: false,
            marker_metadata_written: false,
        };

        assert_eq!(
            checkpoint_target_island(scenario, &checkpoint),
            Some("launch mesa")
        );
        checkpoint.target_island_name = None;
        assert_eq!(
            checkpoint_target_island(scenario, &checkpoint),
            Some("great sky plateau")
        );
    }

    #[test]
    fn wind_visual_sidecar_gate_only_applies_to_wind_critical_checkpoints() {
        let updraft = scenario_named("updraft_route").expect("updraft scenario");
        let island = scenario_named("island_launch_to_landing").expect("island scenario");
        let branch = scenario_named("branch_recovery_route").expect("branch scenario");
        let camera = scenario_named("camera_mouse_control").expect("camera scenario");
        let surface_review =
            scenario_named("island_surface_review").expect("surface review scenario");

        assert!(checkpoint_requires_wind_visual_sample(
            updraft,
            "updraft_entry"
        ));
        assert!(checkpoint_requires_wind_visual_sample(
            updraft,
            "high_glide"
        ));
        assert!(checkpoint_requires_wind_visual_sample(
            island,
            "launch_updraft_entry"
        ));
        assert!(checkpoint_requires_wind_visual_sample(
            island,
            "midroute_lift_view"
        ));
        assert!(checkpoint_requires_wind_visual_sample(
            branch,
            "branch_choice"
        ));
        assert!(checkpoint_requires_wind_visual_sample(
            branch,
            "recovery_approach"
        ));
        assert!(!checkpoint_requires_wind_visual_sample(
            branch,
            "branch_landing_approach"
        ));
        assert!(!checkpoint_requires_wind_visual_sample(
            camera,
            "settled_view"
        ));
        for checkpoint in [
            "ruins_and_rock_detail",
            "dense_flora_detail",
            "lake_river_waterfall_detail",
        ] {
            assert!(!checkpoint_requires_wind_visual_sample(
                surface_review,
                checkpoint
            ));
        }
    }
}
