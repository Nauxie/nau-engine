use crate::authored_assets::{AuthoredAnimationDiagnostics, VisualAssetDiagnostics};
use crate::camera_runtime::{
    CAMERA_PLAYER_FOCUS_HEIGHT, CameraDiagnostics, CameraFollowFilter, MouseLookState,
};
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::debug_visuals::DebugVisuals;
use crate::environment_visuals::{WindResponsiveVisual, wind_responsive_visual_metrics};
use crate::island_visuals::IslandStreamDiagnostics;
use crate::player_runtime::WindForceDiagnostics;
use crate::power_up_runtime::PowerUpCollectionState;
use crate::world_collision_runtime::WorldCollisionDiagnostics;
use crate::{Player, RouteObjectiveTracker};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use nau_engine::camera::{
    CameraControlState, camera_distance, camera_pitch_degrees, camera_target_angle_degrees,
};
use nau_engine::diagnostics::frame_ms;
use nau_engine::environment::{LiftField, WindField, active_lift_fields_at, visible_fields_at};
use nau_engine::movement::{FlightController, Velocity, body_roll_degrees};
use nau_engine::world::SkyRoute;

#[derive(Component)]
pub(crate) struct DebugReadout;

#[derive(SystemParam)]
pub(crate) struct DebugScene<'w, 's> {
    route: Res<'w, SkyRoute>,
    player: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static Velocity,
            &'static FlightController,
        ),
        With<Player>,
    >,
    camera: Query<'w, 's, &'static Transform, CameraFollowFilter>,
    camera_control: Res<'w, CameraControlState>,
    camera_diagnostics: Res<'w, CameraDiagnostics>,
    mouse_look: Res<'w, MouseLookState>,
    stream_diagnostics: Res<'w, IslandStreamDiagnostics>,
    content_diagnostics: Res<'w, IslandContentDiagnostics>,
    asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    authored_animation_diagnostics: Res<'w, AuthoredAnimationDiagnostics>,
    route_objectives: Res<'w, RouteObjectiveTracker>,
    power_ups: Res<'w, PowerUpCollectionState>,
    collision_diagnostics: Res<'w, WorldCollisionDiagnostics>,
    wind_force_diagnostics: Res<'w, WindForceDiagnostics>,
    wind_fields: Query<'w, 's, &'static WindField>,
    lift_fields: Query<'w, 's, &'static LiftField>,
    wind_responsive_visuals: Query<'w, 's, (&'static WindResponsiveVisual, &'static Transform)>,
}

pub(crate) fn update_debug_readout(
    time: Res<Time>,
    visuals: Res<DebugVisuals>,
    scene: DebugScene,
    mut readout: Query<&mut Text, With<DebugReadout>>,
) {
    let Ok((transform, velocity, controller)) = scene.player.single() else {
        return;
    };
    let Ok(mut text) = readout.single_mut() else {
        return;
    };
    let player_focus = transform.translation + Vec3::Y * CAMERA_PLAYER_FOCUS_HEIGHT;
    let (distance, pitch, framing_angle) = scene
        .camera
        .single()
        .map(|camera_transform| {
            (
                camera_distance(camera_transform.translation, transform.translation),
                camera_pitch_degrees(camera_transform.rotation),
                camera_target_angle_degrees(
                    camera_transform.translation,
                    camera_transform.rotation,
                    player_focus,
                ),
            )
        })
        .unwrap_or_default();
    let visible_wind_fields =
        visible_fields_at(transform.translation, scene.wind_fields.iter().copied());
    let wind_field_count = scene.wind_fields.iter().count();
    let active_lift_fields =
        active_lift_fields_at(transform.translation, scene.lift_fields.iter().copied());
    let lift_field_count = scene.lift_fields.iter().count();
    let target_distance = scene.route.target_distance(transform.translation);
    let on_target = scene
        .route
        .on_landing_target(transform.translation, controller.mode);
    let streaming_lod = scene.route.streaming_lod_stats(transform.translation);
    let lod_visuals = scene.stream_diagnostics.counts;
    let asset_metrics = scene.asset_diagnostics.metrics;
    let content_metrics = *scene.content_diagnostics;
    let camera_yaw = scene.camera_control.orbit.yaw_degrees();
    let camera_pitch_offset = scene.camera_control.orbit.pitch_degrees();
    let mouse_lock = if scene.mouse_look.captured {
        "locked"
    } else {
        "free"
    };
    let objective_step =
        (scene.route_objectives.completed_count + 1).min(scene.route_objectives.total_count);
    let objective_state = if scene.route_objectives.complete {
        "done"
    } else {
        "go"
    };
    let (environment_motion_visuals, max_environment_motion_offset_m) =
        wind_responsive_visual_metrics(scene.wind_responsive_visuals.iter());
    let body_roll = body_roll_degrees(transform.rotation);

    **text = format!(
        "frame {:>4.1} ms\nmode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\ntarget {:>5.1} m {}\nobjective {}/{} {} {:>5.1} m {}\ncamera pitch {:>5.1} deg\ncamera distance {:>5.1} m\ncamera frame {:>5.1} deg\ncamera motion {:>4.1} m / {:>4.1} deg\ncamera orbit {:>5.1} deg\ncamera obstruction {:>4.1} m / {}\nmouse yaw {:>5.1} deg\nmouse pitch {:>5.1} deg\nmouse {}\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\nbody bank/roll {:>5.1} / {:>5.1} deg\npower ups visible/collected/active {} / {} / {}\nvisual assets {} gltf {} ready {} placeholders {} missing {} stream {}\nasset load queued/loading/loaded/deferred/failed {} / {} / {} / {} / {}\nasset preload deps/ready {} / {} always/stream {} / {}\nasset scene spawned/ready {} / {}\nauthored world fixtures {}\nasset anim clips ready/declared {} / {} players {} graphs {}\nauthored anim current/desired {} / {} players {} transition {} ms\nasset residency always/window/near/far/weather {} / {} / {} / {} / {}\nvisual wind fields {} / {}\nwind force fields/cross/swirl {} / {} / {} load {:>4.2}\nwind force delta/cross/swirl {:>4.2} / {:>4.2} / {:>4.2} m/s\nlift fields {} / {}\nworld collisions proxies/resolved/push {} / {} / {:>4.2} m\nsky islands {}\nisland terrain surfaces {} vertices {} color bands {} material bands/channels/regions/texture {} / {} / {} / {} relief {:>4.2} m cliff bands {}\nisland body proc/prim {} / {} silhouette min/avg {} / {:>4.1} vertices min/max {} / {}\nground cover patches {} blades {} vertices {}\ngenerated trees trunk/canopy {} / {} vertices {} / {} biome palettes {}\ngenerated rocks {} vertices {}\ngenerated landmarks {} cairn/launch/landing/pond {} / {} / {} / {} vertices {}\ngenerated clouds {} banks {} depth {:>4.1} m lobes min/max {} / {} vertices {} filaments {}\nstream chunk [{}, {}] active {} / {}\nlod near/mid/far {} / {} / {}\nstream terrain visible/hidden {} / {}\nstream impostor visible/hidden {} / {}\nlod detail visible/hidden {} / {}\nenvironment motion {} / {:>4.2} m\nstream residency {} / {} {:>4.1}% hidden {}\nstream spawn/despawn {} / {} max {} / {} total {} / {}\nstream entity changes {} max {} total {}\nroute beacons {}\nlaunch cooldown {:>4.1}s\nlaunch ready {}\ndebug visuals {} (F1)\nWASD camera-relative  Click mouse lock  Esc release  Space glider  E launch  Shift dive  R reset",
        frame_ms(time.delta_secs()),
        controller.mode.label(),
        velocity.0.length(),
        transform.translation.y,
        target_distance,
        if on_target { "landed" } else { "out" },
        objective_step,
        scene.route_objectives.total_count,
        scene.route_objectives.current_label,
        scene.route_objectives.current_distance_m,
        objective_state,
        pitch,
        distance,
        framing_angle,
        scene.camera_diagnostics.step_distance_m,
        scene.camera_diagnostics.rotation_delta_degrees,
        scene.camera_diagnostics.orbit_alignment_degrees,
        scene.camera_diagnostics.obstruction_adjustment_m,
        scene.camera_diagnostics.obstruction_hits,
        camera_yaw,
        camera_pitch_offset,
        mouse_lock,
        velocity.0.x,
        velocity.0.y,
        velocity.0.z,
        controller.bank_degrees,
        body_roll,
        scene.power_ups.visible_count(),
        scene.power_ups.collected_count(),
        scene.power_ups.active_effects(),
        asset_metrics.slot_count,
        asset_metrics.gltf_scene_slot_count,
        asset_metrics.ready_slot_count,
        asset_metrics.placeholder_slot_count,
        asset_metrics.missing_slot_count,
        asset_metrics.streaming_slot_count,
        asset_metrics.queued_scene_count,
        asset_metrics.loading_scene_count,
        asset_metrics.loaded_scene_count,
        asset_metrics.deferred_scene_count,
        asset_metrics.failed_scene_count,
        asset_metrics.dependency_loaded_scene_count,
        asset_metrics.preload_ready_scene_count,
        asset_metrics.always_preload_ready_slot_count,
        asset_metrics.streaming_preload_ready_slot_count,
        asset_metrics.spawned_scene_count,
        asset_metrics.ready_scene_count,
        scene.asset_diagnostics.visible_world_fixture_count,
        asset_metrics.ready_animation_clip_count,
        asset_metrics.declared_animation_clip_count,
        asset_metrics.animation_player_count,
        asset_metrics.animation_graph_count,
        scene.authored_animation_diagnostics.current_label(),
        scene.authored_animation_diagnostics.desired_label(),
        scene.authored_animation_diagnostics.player_count,
        scene.authored_animation_diagnostics.transition_duration_ms,
        asset_metrics.always_slot_count,
        asset_metrics.stream_window_slot_count,
        asset_metrics.near_lod_slot_count,
        asset_metrics.far_lod_slot_count,
        asset_metrics.weather_slot_count,
        visible_wind_fields,
        wind_field_count,
        scene.wind_force_diagnostics.active_fields,
        scene.wind_force_diagnostics.crosswind_fields,
        scene.wind_force_diagnostics.updraft_swirl_fields,
        scene.wind_force_diagnostics.wind_lateral_load,
        scene.wind_force_diagnostics.applied_delta_mps,
        scene.wind_force_diagnostics.crosswind_delta_mps,
        scene.wind_force_diagnostics.updraft_swirl_delta_mps,
        active_lift_fields,
        lift_field_count,
        scene.collision_diagnostics.proxy_count,
        scene.collision_diagnostics.resolved_count,
        scene.collision_diagnostics.max_push_m,
        scene.route.islands().len(),
        content_metrics.island_terrain_surface_count,
        content_metrics.min_island_terrain_mesh_vertices,
        content_metrics.min_island_terrain_color_bands,
        content_metrics.min_island_terrain_material_weight_bands,
        content_metrics.min_island_terrain_material_channels,
        content_metrics.min_island_terrain_material_regions,
        content_metrics.min_island_terrain_texture_detail_bands,
        content_metrics.min_island_terrain_relief_range_m(),
        content_metrics.min_island_cliff_color_bands,
        content_metrics.procedural_island_body_count,
        content_metrics.primitive_island_body_count,
        content_metrics.min_island_body_silhouette_segments,
        content_metrics.average_island_body_silhouette_segments(),
        content_metrics.min_island_body_mesh_vertices,
        content_metrics.max_island_body_mesh_vertices,
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
        content_metrics.generated_landmark_count,
        content_metrics.generated_route_cairn_count,
        content_metrics.generated_launch_beacon_count,
        content_metrics.generated_landing_garden_marker_count,
        content_metrics.generated_pond_surface_count,
        content_metrics.min_landmark_mesh_vertices,
        content_metrics.generated_weather_cloud_count,
        content_metrics.generated_weather_cloud_bank_count,
        content_metrics.min_weather_cloud_bank_depth_m(),
        content_metrics.min_weather_cloud_lobe_count,
        content_metrics.max_weather_cloud_lobe_count,
        content_metrics.min_weather_cloud_mesh_vertices,
        content_metrics.min_weather_cloud_filament_ribbon_detail_count,
        streaming_lod.player_chunk.x,
        streaming_lod.player_chunk.z,
        streaming_lod.active_island_count,
        streaming_lod.active_chunk_count,
        streaming_lod.near_lod_islands,
        streaming_lod.mid_lod_islands,
        streaming_lod.far_lod_islands,
        lod_visuals.visible_terrain_count,
        lod_visuals.hidden_terrain_count,
        lod_visuals.visible_impostor_count,
        lod_visuals.hidden_impostor_count,
        lod_visuals.visible_detail_count,
        lod_visuals.hidden_detail_count,
        environment_motion_visuals,
        max_environment_motion_offset_m,
        lod_visuals.resident_count(),
        lod_visuals.catalog_count(),
        lod_visuals.resident_fraction() * 100.0,
        lod_visuals.hidden_count(),
        scene.stream_diagnostics.spawned_visuals_this_frame,
        scene.stream_diagnostics.despawned_visuals_this_frame,
        scene.stream_diagnostics.max_spawned_visuals_per_frame,
        scene.stream_diagnostics.max_despawned_visuals_per_frame,
        scene.stream_diagnostics.total_spawned_visuals,
        scene.stream_diagnostics.total_despawned_visuals,
        scene.stream_diagnostics.visibility_changes_this_frame,
        scene.stream_diagnostics.max_visibility_changes_per_frame,
        scene.stream_diagnostics.total_visibility_changes,
        lod_visuals.visible_beacon_count,
        controller.launch_cooldown_remaining,
        if controller.launch_available {
            "yes"
        } else {
            "no"
        },
        if visuals.enabled { "on" } else { "off" }
    );
}
