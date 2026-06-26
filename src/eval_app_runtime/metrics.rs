use super::scene::EvalScene;
use crate::camera_runtime::CAMERA_PLAYER_FOCUS_HEIGHT;
use crate::environment_visuals::wind_responsive_visual_metrics;
use crate::eval_runtime::{EvalMovementBasis, EvalRun};
use crate::{grounded_visual_foot_gap_m, movement_facing};
use bevy::prelude::*;
use nau_engine::camera::{
    CameraControlState, camera_distance, camera_pitch_degrees, camera_surface_clearance,
    camera_target_angle_degrees, camera_view_yaw_degrees,
};
use nau_engine::diagnostics::frame_ms;
use nau_engine::environment::{
    AERIAL_POWER_UP_ROUTE, active_lift_fields_at, readable_lift_fields_at, visible_fields_at,
};
use nau_engine::eval::{EvalMovementMetrics, EvalObjectiveProgress, EvalSample, scripted_input};
use nau_engine::movement::{
    FlightMode, body_roll_degrees, body_yaw_error_degrees, desired_heading_alignment_speed,
    desired_planar_movement_direction, lateral_response_speed,
};

pub(super) const EVAL_FRAME_TIME_WARMUP_FRAMES: u32 = 5;

pub(crate) fn collect_eval_frame_time(time: Res<Time>, mut run: ResMut<EvalRun>) {
    if !run.finalized && run.frame >= EVAL_FRAME_TIME_WARMUP_FRAMES {
        run.accumulator
            .observe_frame_time_ms(frame_ms(time.delta_secs()));
    }
}

pub(crate) fn collect_eval_metrics(
    mut run: ResMut<EvalRun>,
    camera_control: Res<CameraControlState>,
    movement_basis: Res<EvalMovementBasis>,
    scene: EvalScene,
) {
    if run.finalized || !run.scenario.should_sample(run.frame) {
        return;
    }

    let Ok((transform, velocity, controller)) = scene.player.single() else {
        return;
    };
    let (
        camera_distance_m,
        camera_surface_clearance_m,
        camera_player_angle_degrees,
        camera_pitch_degrees,
        camera_view_yaw,
        camera_world_yaw,
    ) = scene
        .camera
        .single()
        .map(|camera_transform| {
            let camera_floor_y = scene.route.ground_at(camera_transform.translation).floor_y;
            let player_focus = transform.translation + Vec3::Y * CAMERA_PLAYER_FOCUS_HEIGHT;
            (
                camera_distance(camera_transform.translation, transform.translation),
                camera_surface_clearance(camera_transform.translation, camera_floor_y),
                camera_target_angle_degrees(
                    camera_transform.translation,
                    camera_transform.rotation,
                    player_focus,
                ),
                camera_pitch_degrees(camera_transform.rotation),
                camera_view_yaw_degrees(
                    camera_transform.rotation,
                    scene.camera_diagnostics.follow_direction,
                ),
                camera_view_yaw_degrees(camera_transform.rotation, Vec3::NEG_Z),
            )
        })
        .unwrap_or_default();
    let visible_wind_fields =
        visible_fields_at(transform.translation, scene.wind_fields.iter().copied());
    let active_lift_fields =
        active_lift_fields_at(transform.translation, scene.lift_fields.iter().copied());
    let readable_lift_fields = readable_lift_fields_at(
        transform.translation,
        scene.lift_fields.iter().copied(),
        scene.wind_fields.iter().copied(),
    );
    let player_ground = scene.route.ground_at(transform.translation);
    let visual_foot_gap_m = grounded_visual_foot_gap_m(
        transform.translation.y,
        player_ground.floor_y,
        controller.mode,
    );
    let scenario_target = run.scenario.target_island_name;
    let target_distance_m = scene
        .route
        .target_distance_to(transform.translation, scenario_target);
    let on_landing_target = scene.route.on_landing_target_named(
        transform.translation,
        controller.mode,
        scenario_target,
    );
    let objective = EvalObjectiveProgress::new(
        scene.route_objectives.completed_count,
        scene.route_objectives.total_count,
        scene.route_objectives.current_label,
        scene.route_objectives.current_distance_m,
        scene.route_objectives.complete,
    );
    let streaming_lod = scene.route.streaming_lod_stats(transform.translation);
    let lod_visuals = scene.stream_diagnostics.counts;
    let asset_metrics = scene.asset_diagnostics.metrics;
    let content_metrics = *scene.content_diagnostics;
    let (environment_motion_visuals, max_environment_motion_offset_m) =
        wind_responsive_visual_metrics(scene.wind_responsive_visuals.iter());
    let movement_input = scripted_input(run.scenario, run.frame);
    let movement_axis = movement_input.planar_axis();
    let movement_facing = if movement_basis.frame == run.frame {
        movement_basis
            .facing
            .unwrap_or_else(|| movement_facing(scene.camera.single().ok(), transform))
    } else {
        movement_facing(scene.camera.single().ok(), transform)
    };
    let desired_movement_direction =
        if movement_input.forward || movement_input.left || movement_input.right {
            desired_planar_movement_direction(movement_input, movement_facing)
        } else {
            None
        };
    let desired_body_yaw_error_degrees = desired_movement_direction
        .map(|direction| body_yaw_error_degrees(transform.rotation, direction))
        .unwrap_or(f32::NAN);
    let desired_heading_alignment_mps = desired_movement_direction
        .map(|direction| desired_heading_alignment_speed(velocity.0, direction))
        .unwrap_or(f32::NAN);
    let lateral_axis_active = movement_input.has_lateral_axis();
    let lateral_input_active = lateral_axis_active && controller.mode != FlightMode::Grounded;
    let lateral_response_mps = if lateral_axis_active {
        lateral_response_speed(velocity.0, movement_input, movement_facing)
    } else {
        0.0
    };
    let sample = EvalSample::new(
        run.frame,
        run.scenario.fixed_dt,
        transform.translation,
        velocity.0,
        controller.mode,
        camera_distance_m,
        camera_surface_clearance_m,
        camera_player_angle_degrees,
        camera_pitch_degrees,
        camera_control.orbit.yaw_degrees(),
        camera_control.orbit.pitch_degrees(),
        scene.camera_diagnostics.step_distance_m,
        scene.camera_diagnostics.rotation_delta_degrees,
        scene.camera_diagnostics.orbit_alignment_degrees,
        camera_view_yaw,
        scene.camera_diagnostics.obstruction_adjustment_m,
        scene.camera_diagnostics.obstruction_hits,
        visible_wind_fields,
        scene.wind_fields.iter().count(),
        active_lift_fields,
        readable_lift_fields,
        scene.lift_fields.iter().count(),
        target_distance_m,
        on_landing_target,
        objective,
        scene.route.islands().len(),
        streaming_lod.active_chunk_count,
        streaming_lod.active_island_count,
        streaming_lod.near_lod_islands,
        streaming_lod.mid_lod_islands,
        streaming_lod.far_lod_islands,
        lod_visuals.visible_terrain_count,
        lod_visuals.hidden_terrain_count,
        lod_visuals.visible_impostor_count,
        lod_visuals.hidden_impostor_count,
        lod_visuals.visible_detail_count,
        lod_visuals.hidden_detail_count,
        lod_visuals.visible_beacon_count,
        scene.weather_clouds.iter().count(),
        environment_motion_visuals,
        max_environment_motion_offset_m,
        lod_visuals.resident_count(),
        scene.stream_diagnostics.visibility_changes_this_frame,
        scene.stream_diagnostics.max_visibility_changes_per_frame,
        scene.stream_diagnostics.total_visibility_changes,
        lod_visuals.catalog_count(),
        lod_visuals.hidden_count(),
        lod_visuals.resident_fraction(),
        scene.stream_diagnostics.spawned_visuals_this_frame,
        scene.stream_diagnostics.despawned_visuals_this_frame,
        scene.stream_diagnostics.max_spawned_visuals_per_frame,
        scene.stream_diagnostics.max_despawned_visuals_per_frame,
        scene.stream_diagnostics.total_spawned_visuals,
        scene.stream_diagnostics.total_despawned_visuals,
        scene.all_entities.iter().count(),
        asset_metrics.slot_count,
        asset_metrics.gltf_scene_slot_count,
        asset_metrics.ready_slot_count,
        asset_metrics.placeholder_slot_count,
        asset_metrics.streaming_slot_count,
        asset_metrics.missing_slot_count,
        asset_metrics.queued_scene_count,
        asset_metrics.loading_scene_count,
        asset_metrics.loaded_scene_count,
        asset_metrics.dependency_loaded_scene_count,
        asset_metrics.preload_ready_scene_count,
        asset_metrics.failed_scene_count,
        asset_metrics.spawned_scene_count,
        asset_metrics.ready_scene_count,
        asset_metrics.always_slot_count,
        asset_metrics.stream_window_slot_count,
        asset_metrics.near_lod_slot_count,
        asset_metrics.far_lod_slot_count,
        asset_metrics.weather_slot_count,
        asset_metrics.always_preload_ready_slot_count,
        asset_metrics.streaming_preload_ready_slot_count,
        asset_metrics.declared_animation_clip_count,
        asset_metrics.ready_animation_clip_count,
        asset_metrics.animation_player_count,
        asset_metrics.animation_graph_count,
        AERIAL_POWER_UP_ROUTE.len(),
        scene.power_ups.visible_count(),
        scene.power_ups.collected_count(),
        scene.power_ups.active_effects(),
        scene.power_ups.total_activations(),
    )
    .with_visible_authored_world_fixture_count(scene.asset_diagnostics.visible_world_fixture_count)
    .with_deferred_visual_asset_scene_count(asset_metrics.deferred_scene_count)
    .with_camera_follow_metrics(scene.camera_diagnostics.follow_direction_error_degrees)
    .with_camera_world_yaw_metrics(camera_world_yaw)
    .with_visual_foot_gap(visual_foot_gap_m)
    .with_content_metrics(
        content_metrics.island_terrain_surface_count,
        content_metrics.min_island_terrain_mesh_vertices,
        content_metrics.min_island_terrain_color_bands,
        content_metrics.min_island_terrain_relief_range_m(),
        content_metrics.min_island_cliff_color_bands,
        content_metrics.procedural_island_body_count,
        content_metrics.primitive_island_body_count,
        content_metrics.min_island_body_silhouette_segments,
        content_metrics.average_island_body_silhouette_segments(),
        content_metrics.min_island_body_mesh_vertices,
        content_metrics.max_island_body_mesh_vertices,
    )
    .with_island_impostor_metrics(
        content_metrics.min_island_impostor_mesh_vertices,
        content_metrics.min_island_impostor_color_bands,
    )
    .with_terrain_material_metrics(
        content_metrics.min_island_terrain_material_weight_bands,
        content_metrics.min_island_terrain_material_channels,
        content_metrics.min_island_terrain_material_regions,
        content_metrics.min_island_terrain_texture_detail_bands,
    )
    .with_generated_visual_shape_metrics(
        content_metrics.generated_ground_cover_patch_count,
        content_metrics.min_ground_cover_blade_count,
        content_metrics.min_ground_cover_mesh_vertices,
        content_metrics.generated_tree_trunk_count,
        content_metrics.generated_tree_canopy_count,
        content_metrics.min_tree_trunk_mesh_vertices,
        content_metrics.min_tree_canopy_mesh_vertices,
        content_metrics.detail_biome_palette_count(),
        content_metrics.generated_rock_count,
        content_metrics.min_rock_mesh_vertices,
        content_metrics.generated_weather_cloud_count,
        content_metrics.generated_weather_cloud_bank_count,
        content_metrics.min_weather_cloud_bank_depth_m(),
        content_metrics.min_weather_cloud_lobe_count,
        content_metrics.max_weather_cloud_lobe_count,
        content_metrics.min_weather_cloud_mesh_vertices,
    )
    .with_movement_metrics(EvalMovementMetrics {
        desired_body_yaw_error_degrees,
        body_roll_degrees: body_roll_degrees(transform.rotation),
        desired_heading_alignment_mps,
        lateral_response_mps,
        lateral_input_active,
        movement_axis,
    });

    if let Err(error) = run.record_sample(sample) {
        run.io_error = Some(format!("failed to write eval sample: {error}"));
    }
}
