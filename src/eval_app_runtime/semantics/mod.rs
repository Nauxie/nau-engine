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

fn write_checkpoint_marker_metadata(
    path: &Path,
    scenario: EvalScenario,
    checkpoint: &EvalCheckpointCapture,
    scene: &EvalScene,
) -> std::io::Result<()> {
    let markers = markers::semantic_route_markers(scene);
    let scene_samples = samples::semantic_scene_samples(scene);
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
    let passed = markers.len() >= expected_objective_count
        && visible_count > 0
        && scene_samples.len() >= 4
        && visible_scene_sample_count > 0
        && (!checkpoint_requires_wind_visual_sample(scenario, checkpoint.name)
            || visible_wind_scene_sample_count > 0)
        && viewport_size.is_some();
    let target_island = scenario
        .target_island_name
        .map(terrain_export_json_string)
        .unwrap_or_else(|| "null".to_string());
    let json = format!(
        "{{\n  \"passed\": {},\n  \"scenario\": {},\n  \"target_island\": {},\n  \"frame\": {},\n  \"checkpoint\": {},\n  \"screenshot\": {},\n  \"viewport\": {},\n  \"semantic_marker_count\": {},\n  \"expected_objective_marker_count\": {},\n  \"in_viewport_semantic_marker_count\": {},\n  \"occluded_semantic_marker_count\": {},\n  \"visible_semantic_marker_count\": {},\n  \"current_objective_visible\": {},\n  \"semantic_scene_sample_count\": {},\n  \"in_viewport_semantic_scene_sample_count\": {},\n  \"occluded_semantic_scene_sample_count\": {},\n  \"visible_semantic_scene_sample_count\": {},\n  \"visible_semantic_scene_material_count\": {},\n  \"visible_wind_scene_sample_count\": {},\n  \"markers\": [\n{}\n  ],\n  \"scene_samples\": [\n{}\n  ]\n}}\n",
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

fn checkpoint_requires_wind_visual_sample(scenario: EvalScenario, checkpoint_name: &str) -> bool {
    matches!(
        (scenario.name, checkpoint_name),
        ("updraft_route", "updraft_entry" | "high_glide")
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
    fn wind_visual_sidecar_gate_only_applies_to_wind_critical_checkpoints() {
        let updraft = scenario_named("updraft_route").expect("updraft scenario");
        let branch = scenario_named("branch_recovery_route").expect("branch scenario");
        let camera = scenario_named("camera_mouse_control").expect("camera scenario");

        assert!(checkpoint_requires_wind_visual_sample(
            updraft,
            "updraft_entry"
        ));
        assert!(checkpoint_requires_wind_visual_sample(
            updraft,
            "high_glide"
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
    }
}
