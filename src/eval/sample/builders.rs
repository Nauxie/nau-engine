use bevy::prelude::*;

use crate::movement::FlightMode;

use super::super::vec3_array;
use super::types::{
    EvalMovementMetrics, EvalObjectiveProgress, EvalPoseReadabilityMetrics, EvalSample,
};

impl EvalSample {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        frame: u32,
        fixed_dt: f32,
        position: Vec3,
        velocity: Vec3,
        mode: FlightMode,
        pose_intent_label: &'static str,
        camera_distance_m: f32,
        camera_surface_clearance_m: f32,
        camera_player_angle_degrees: f32,
        camera_pitch_degrees: f32,
        camera_yaw_offset_degrees: f32,
        camera_pitch_offset_degrees: f32,
        camera_step_distance_m: f32,
        camera_rotation_delta_degrees: f32,
        camera_orbit_alignment_degrees: f32,
        camera_view_yaw_degrees: f32,
        camera_obstruction_adjustment_m: f32,
        camera_obstruction_hits: usize,
        visible_wind_fields: usize,
        wind_field_count: usize,
        dynamic_wind_flow_fields: usize,
        max_wind_flow_speed_mps: f32,
        max_wind_flow_variation: f32,
        active_lift_fields: usize,
        readable_lift_fields: usize,
        lift_field_count: usize,
        target_distance_m: f32,
        on_landing_target: bool,
        objective: EvalObjectiveProgress,
        sky_island_count: usize,
        active_chunk_count: usize,
        active_island_count: usize,
        near_lod_islands: usize,
        mid_lod_islands: usize,
        far_lod_islands: usize,
        visible_island_terrain_count: usize,
        hidden_island_terrain_count: usize,
        visible_island_impostor_count: usize,
        hidden_island_impostor_count: usize,
        visible_island_detail_count: usize,
        hidden_island_detail_count: usize,
        visible_route_beacon_count: usize,
        weather_cloud_count: usize,
        environment_motion_visual_count: usize,
        max_environment_motion_offset_m: f32,
        resident_island_visual_count: usize,
        stream_visibility_changes_this_frame: usize,
        max_stream_visibility_changes_per_frame: usize,
        total_stream_visibility_changes: usize,
        catalog_island_visual_count: usize,
        hidden_island_visual_count: usize,
        resident_island_visual_fraction: f32,
        stream_spawned_visuals_this_frame: usize,
        stream_despawned_visuals_this_frame: usize,
        max_stream_spawned_visuals_per_frame: usize,
        max_stream_despawned_visuals_per_frame: usize,
        total_stream_spawned_visuals: usize,
        total_stream_despawned_visuals: usize,
        entity_count: usize,
        visual_asset_slot_count: usize,
        gltf_scene_asset_slot_count: usize,
        ready_visual_asset_slot_count: usize,
        placeholder_visual_asset_slot_count: usize,
        streaming_visual_asset_slot_count: usize,
        missing_visual_asset_slot_count: usize,
        queued_visual_asset_scene_count: usize,
        loading_visual_asset_scene_count: usize,
        loaded_visual_asset_scene_count: usize,
        dependency_loaded_visual_asset_scene_count: usize,
        preload_ready_visual_asset_scene_count: usize,
        failed_visual_asset_scene_count: usize,
        spawned_visual_asset_scene_count: usize,
        ready_visual_asset_scene_count: usize,
        always_visual_asset_slot_count: usize,
        stream_window_visual_asset_slot_count: usize,
        near_lod_visual_asset_slot_count: usize,
        far_lod_visual_asset_slot_count: usize,
        weather_visual_asset_slot_count: usize,
        always_preload_ready_visual_asset_slot_count: usize,
        streaming_preload_ready_visual_asset_slot_count: usize,
        declared_animation_clip_count: usize,
        ready_animation_clip_count: usize,
        animation_player_count: usize,
        animation_graph_count: usize,
        power_up_count: usize,
        visible_power_up_count: usize,
        collected_power_up_count: usize,
        active_power_up_effects: usize,
        total_power_up_activations: usize,
    ) -> Self {
        Self {
            frame,
            time_secs: frame as f32 * fixed_dt,
            position: vec3_array(position),
            velocity: vec3_array(velocity),
            speed_mps: velocity.length(),
            altitude_m: position.y,
            mode: mode.label(),
            pose_intent_label,
            pose_torso_pitch_degrees: 0.0,
            pose_arm_spread_degrees: 0.0,
            pose_leg_tuck_degrees: 0.0,
            pose_lateral_lean_degrees: 0.0,
            pose_signed_lateral_lean_degrees: 0.0,
            pose_landing_crouch_m: 0.0,
            pose_wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
            desired_body_yaw_error_degrees: f32::NAN,
            desired_body_heading_error_degrees: f32::NAN,
            body_travel_heading_error_degrees: f32::NAN,
            body_roll_degrees: 0.0,
            desired_heading_alignment_mps: f32::NAN,
            lateral_response_mps: 0.0,
            lateral_input_active: false,
            movement_input_lateral_axis: 0.0,
            movement_input_forward_axis: 0.0,
            camera_distance_m,
            camera_surface_clearance_m,
            camera_player_angle_degrees,
            camera_pitch_degrees,
            camera_yaw_offset_degrees,
            camera_pitch_offset_degrees,
            camera_step_distance_m,
            camera_rotation_delta_degrees,
            camera_orbit_alignment_degrees,
            camera_follow_direction_error_degrees: 0.0,
            camera_view_yaw_degrees,
            camera_world_yaw_degrees: 0.0,
            camera_obstruction_adjustment_m,
            camera_obstruction_hits,
            visible_wind_fields,
            wind_field_count,
            dynamic_wind_flow_fields,
            max_wind_flow_speed_mps,
            max_wind_flow_variation,
            active_wind_force_fields: 0,
            crosswind_force_fields: 0,
            updraft_swirl_force_fields: 0,
            max_wind_force_delta_mps: 0.0,
            max_crosswind_force_delta_mps: 0.0,
            max_updraft_swirl_force_delta_mps: 0.0,
            max_wind_force_flow_speed_mps: 0.0,
            max_wind_force_variation: 0.0,
            active_lift_fields,
            readable_lift_fields,
            lift_field_count,
            target_distance_m,
            on_landing_target,
            objective,
            sky_island_count,
            active_chunk_count,
            active_island_count,
            near_lod_islands,
            mid_lod_islands,
            far_lod_islands,
            visible_island_terrain_count,
            hidden_island_terrain_count,
            visible_island_impostor_count,
            hidden_island_impostor_count,
            visible_island_detail_count,
            hidden_island_detail_count,
            visible_route_beacon_count,
            weather_cloud_count,
            environment_motion_visual_count,
            max_environment_motion_offset_m,
            updraft_guide_visual_count: 0,
            updraft_ribbon_visual_count: 0,
            crosswind_guide_visual_count: 0,
            crosswind_ribbon_visual_count: 0,
            max_updraft_visual_motion_m: 0.0,
            max_crosswind_visual_motion_m: 0.0,
            world_collision_proxy_count: 0,
            world_collision_resolved_count: 0,
            max_world_collision_push_m: 0.0,
            island_terrain_surface_count: 0,
            min_island_terrain_mesh_vertices: 0,
            min_island_terrain_color_bands: 0,
            min_island_terrain_material_weight_bands: 0,
            min_island_terrain_material_channels: 0,
            min_island_terrain_material_regions: 0,
            min_island_terrain_texture_detail_bands: 0,
            min_island_terrain_relief_range_m: 0.0,
            island_terrain_archetype_count: 0,
            min_island_cliff_color_bands: 0,
            min_island_impostor_mesh_vertices: 0,
            min_island_impostor_color_bands: 0,
            procedural_island_body_count: 0,
            primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 0,
            avg_island_body_silhouette_segments: 0.0,
            min_island_body_mesh_vertices: 0,
            max_island_body_mesh_vertices: 0,
            generated_ground_cover_patch_count: 0,
            min_ground_cover_blade_count: 0,
            min_ground_cover_mesh_vertices: 0,
            generated_tree_trunk_count: 0,
            generated_tree_canopy_count: 0,
            min_tree_trunk_mesh_vertices: 0,
            min_tree_canopy_mesh_vertices: 0,
            detail_biome_palette_count: 0,
            generated_rock_count: 0,
            min_rock_mesh_vertices: 0,
            generated_landmark_count: 0,
            generated_route_cairn_count: 0,
            generated_launch_beacon_count: 0,
            generated_landing_garden_marker_count: 0,
            generated_pond_surface_count: 0,
            min_landmark_mesh_vertices: 0,
            generated_weather_cloud_count: 0,
            generated_weather_cloud_bank_count: 0,
            min_weather_cloud_bank_depth_m: 0.0,
            min_weather_cloud_lobe_count: 0,
            max_weather_cloud_lobe_count: 0,
            min_weather_cloud_mesh_vertices: 0,
            min_weather_cloud_filament_ribbon_detail_count: 0,
            resident_island_visual_count,
            stream_visibility_changes_this_frame,
            max_stream_visibility_changes_per_frame,
            total_stream_visibility_changes,
            catalog_island_visual_count,
            hidden_island_visual_count,
            resident_island_visual_fraction,
            stream_spawned_visuals_this_frame,
            stream_despawned_visuals_this_frame,
            max_stream_spawned_visuals_per_frame,
            max_stream_despawned_visuals_per_frame,
            total_stream_spawned_visuals,
            total_stream_despawned_visuals,
            entity_count,
            visual_asset_slot_count,
            gltf_scene_asset_slot_count,
            ready_visual_asset_slot_count,
            placeholder_visual_asset_slot_count,
            streaming_visual_asset_slot_count,
            missing_visual_asset_slot_count,
            deferred_visual_asset_scene_count: 0,
            queued_visual_asset_scene_count,
            loading_visual_asset_scene_count,
            loaded_visual_asset_scene_count,
            dependency_loaded_visual_asset_scene_count,
            preload_ready_visual_asset_scene_count,
            failed_visual_asset_scene_count,
            spawned_visual_asset_scene_count,
            ready_visual_asset_scene_count,
            visible_authored_world_fixture_count: 0,
            always_visual_asset_slot_count,
            stream_window_visual_asset_slot_count,
            near_lod_visual_asset_slot_count,
            far_lod_visual_asset_slot_count,
            weather_visual_asset_slot_count,
            always_preload_ready_visual_asset_slot_count,
            streaming_preload_ready_visual_asset_slot_count,
            declared_animation_clip_count,
            ready_animation_clip_count,
            animation_player_count,
            animation_graph_count,
            power_up_count,
            visible_power_up_count,
            collected_power_up_count,
            active_power_up_effects,
            total_power_up_activations,
            visual_foot_gap_m: 0.0,
        }
    }

    pub fn with_movement_metrics(mut self, metrics: EvalMovementMetrics) -> Self {
        self.desired_body_yaw_error_degrees = metrics.desired_body_yaw_error_degrees;
        self.desired_body_heading_error_degrees = metrics.desired_body_yaw_error_degrees.abs();
        self.body_travel_heading_error_degrees = metrics.body_travel_heading_error_degrees;
        self.body_roll_degrees = metrics.body_roll_degrees;
        self.desired_heading_alignment_mps = metrics.desired_heading_alignment_mps;
        self.lateral_response_mps = metrics.lateral_response_mps;
        self.lateral_input_active = metrics.lateral_input_active;
        self.movement_input_lateral_axis = metrics.movement_axis.x;
        self.movement_input_forward_axis = metrics.movement_axis.y;
        self
    }

    pub fn with_body_travel_heading_error_degrees(mut self, error_degrees: f32) -> Self {
        self.body_travel_heading_error_degrees = if error_degrees.is_finite() {
            error_degrees.max(0.0)
        } else {
            f32::NAN
        };
        self
    }

    pub fn with_pose_readability_metrics(mut self, metrics: EvalPoseReadabilityMetrics) -> Self {
        self.pose_torso_pitch_degrees = metrics.torso_pitch_degrees.max(0.0);
        self.pose_arm_spread_degrees = metrics.arm_spread_degrees.max(0.0);
        self.pose_leg_tuck_degrees = metrics.leg_tuck_degrees.max(0.0);
        self.pose_lateral_lean_degrees = metrics.lateral_lean_degrees.max(0.0);
        self.pose_signed_lateral_lean_degrees = if metrics.signed_lateral_lean_degrees.is_finite() {
            metrics.signed_lateral_lean_degrees
        } else {
            0.0
        };
        self.pose_landing_crouch_m = metrics.landing_crouch_m.max(0.0);
        self.pose_wing_airflow_strength = metrics.wing_airflow_strength.clamp(0.0, 1.0);
        self.key_pose_readability_score = metrics.key_pose_readability_score.clamp(0.0, 1.0);
        self
    }

    pub fn with_camera_follow_metrics(
        mut self,
        camera_follow_direction_error_degrees: f32,
    ) -> Self {
        self.camera_follow_direction_error_degrees =
            if camera_follow_direction_error_degrees.is_finite() {
                camera_follow_direction_error_degrees
            } else {
                0.0
            };
        self
    }

    pub fn with_camera_world_yaw_metrics(mut self, camera_world_yaw_degrees: f32) -> Self {
        self.camera_world_yaw_degrees = if camera_world_yaw_degrees.is_finite() {
            camera_world_yaw_degrees
        } else {
            0.0
        };
        self
    }

    pub fn with_camera_view_yaw_metrics(mut self, camera_view_yaw_degrees: f32) -> Self {
        self.camera_view_yaw_degrees = if camera_view_yaw_degrees.is_finite() {
            camera_view_yaw_degrees
        } else {
            0.0
        };
        self
    }

    pub fn with_visual_foot_gap(mut self, visual_foot_gap_m: f32) -> Self {
        self.visual_foot_gap_m = visual_foot_gap_m;
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_wind_force_metrics(
        mut self,
        active_field_count: usize,
        crosswind_field_count: usize,
        updraft_swirl_field_count: usize,
        max_force_delta_mps: f32,
        max_crosswind_force_delta_mps: f32,
        max_updraft_swirl_force_delta_mps: f32,
        max_flow_speed_mps: f32,
        max_variation: f32,
    ) -> Self {
        self.active_wind_force_fields = active_field_count;
        self.crosswind_force_fields = crosswind_field_count;
        self.updraft_swirl_force_fields = updraft_swirl_field_count;
        self.max_wind_force_delta_mps = max_force_delta_mps.max(0.0);
        self.max_crosswind_force_delta_mps = max_crosswind_force_delta_mps.max(0.0);
        self.max_updraft_swirl_force_delta_mps = max_updraft_swirl_force_delta_mps.max(0.0);
        self.max_wind_force_flow_speed_mps = max_flow_speed_mps.max(0.0);
        self.max_wind_force_variation = max_variation.max(0.0);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_wind_guide_visual_metrics(
        mut self,
        updraft_guide_count: usize,
        updraft_ribbon_count: usize,
        crosswind_guide_count: usize,
        crosswind_ribbon_count: usize,
        max_updraft_motion_m: f32,
        max_crosswind_motion_m: f32,
    ) -> Self {
        self.updraft_guide_visual_count = updraft_guide_count;
        self.updraft_ribbon_visual_count = updraft_ribbon_count;
        self.crosswind_guide_visual_count = crosswind_guide_count;
        self.crosswind_ribbon_visual_count = crosswind_ribbon_count;
        self.max_updraft_visual_motion_m = max_updraft_motion_m.max(0.0);
        self.max_crosswind_visual_motion_m = max_crosswind_motion_m.max(0.0);
        self
    }

    pub fn with_world_collision_metrics(
        mut self,
        proxy_count: usize,
        resolved_count: usize,
        max_push_m: f32,
    ) -> Self {
        self.world_collision_proxy_count = proxy_count;
        self.world_collision_resolved_count = resolved_count;
        self.max_world_collision_push_m = max_push_m.max(0.0);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_content_metrics(
        mut self,
        island_terrain_surface_count: usize,
        min_island_terrain_mesh_vertices: usize,
        min_island_terrain_color_bands: usize,
        min_island_terrain_relief_range_m: f32,
        island_terrain_archetype_count: usize,
        min_island_cliff_color_bands: usize,
        procedural_island_body_count: usize,
        primitive_island_body_count: usize,
        min_island_body_silhouette_segments: usize,
        avg_island_body_silhouette_segments: f32,
        min_island_body_mesh_vertices: usize,
        max_island_body_mesh_vertices: usize,
    ) -> Self {
        self.island_terrain_surface_count = island_terrain_surface_count;
        self.min_island_terrain_mesh_vertices = min_island_terrain_mesh_vertices;
        self.min_island_terrain_color_bands = min_island_terrain_color_bands;
        self.min_island_terrain_relief_range_m = min_island_terrain_relief_range_m;
        self.island_terrain_archetype_count = island_terrain_archetype_count;
        self.min_island_cliff_color_bands = min_island_cliff_color_bands;
        self.procedural_island_body_count = procedural_island_body_count;
        self.primitive_island_body_count = primitive_island_body_count;
        self.min_island_body_silhouette_segments = min_island_body_silhouette_segments;
        self.avg_island_body_silhouette_segments = avg_island_body_silhouette_segments;
        self.min_island_body_mesh_vertices = min_island_body_mesh_vertices;
        self.max_island_body_mesh_vertices = max_island_body_mesh_vertices;
        self
    }

    pub fn with_island_impostor_metrics(
        mut self,
        min_island_impostor_mesh_vertices: usize,
        min_island_impostor_color_bands: usize,
    ) -> Self {
        self.min_island_impostor_mesh_vertices = min_island_impostor_mesh_vertices;
        self.min_island_impostor_color_bands = min_island_impostor_color_bands;
        self
    }

    pub fn with_terrain_material_metrics(
        mut self,
        min_island_terrain_material_weight_bands: usize,
        min_island_terrain_material_channels: usize,
        min_island_terrain_material_regions: usize,
        min_island_terrain_texture_detail_bands: usize,
    ) -> Self {
        self.min_island_terrain_material_weight_bands = min_island_terrain_material_weight_bands;
        self.min_island_terrain_material_channels = min_island_terrain_material_channels;
        self.min_island_terrain_material_regions = min_island_terrain_material_regions;
        self.min_island_terrain_texture_detail_bands = min_island_terrain_texture_detail_bands;
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_generated_visual_shape_metrics(
        mut self,
        generated_ground_cover_patch_count: usize,
        min_ground_cover_blade_count: usize,
        min_ground_cover_mesh_vertices: usize,
        generated_tree_trunk_count: usize,
        generated_tree_canopy_count: usize,
        min_tree_trunk_mesh_vertices: usize,
        min_tree_canopy_mesh_vertices: usize,
        detail_biome_palette_count: usize,
        generated_rock_count: usize,
        min_rock_mesh_vertices: usize,
        generated_landmark_count: usize,
        generated_route_cairn_count: usize,
        generated_launch_beacon_count: usize,
        generated_landing_garden_marker_count: usize,
        generated_pond_surface_count: usize,
        min_landmark_mesh_vertices: usize,
        generated_weather_cloud_count: usize,
        generated_weather_cloud_bank_count: usize,
        min_weather_cloud_bank_depth_m: f32,
        min_weather_cloud_lobe_count: usize,
        max_weather_cloud_lobe_count: usize,
        min_weather_cloud_mesh_vertices: usize,
        min_weather_cloud_filament_ribbon_detail_count: usize,
    ) -> Self {
        self.generated_ground_cover_patch_count = generated_ground_cover_patch_count;
        self.min_ground_cover_blade_count = min_ground_cover_blade_count;
        self.min_ground_cover_mesh_vertices = min_ground_cover_mesh_vertices;
        self.generated_tree_trunk_count = generated_tree_trunk_count;
        self.generated_tree_canopy_count = generated_tree_canopy_count;
        self.min_tree_trunk_mesh_vertices = min_tree_trunk_mesh_vertices;
        self.min_tree_canopy_mesh_vertices = min_tree_canopy_mesh_vertices;
        self.detail_biome_palette_count = detail_biome_palette_count;
        self.generated_rock_count = generated_rock_count;
        self.min_rock_mesh_vertices = min_rock_mesh_vertices;
        self.generated_landmark_count = generated_landmark_count;
        self.generated_route_cairn_count = generated_route_cairn_count;
        self.generated_launch_beacon_count = generated_launch_beacon_count;
        self.generated_landing_garden_marker_count = generated_landing_garden_marker_count;
        self.generated_pond_surface_count = generated_pond_surface_count;
        self.min_landmark_mesh_vertices = min_landmark_mesh_vertices;
        self.generated_weather_cloud_count = generated_weather_cloud_count;
        self.generated_weather_cloud_bank_count = generated_weather_cloud_bank_count;
        self.min_weather_cloud_bank_depth_m = min_weather_cloud_bank_depth_m;
        self.min_weather_cloud_lobe_count = min_weather_cloud_lobe_count;
        self.max_weather_cloud_lobe_count = max_weather_cloud_lobe_count;
        self.min_weather_cloud_mesh_vertices = min_weather_cloud_mesh_vertices;
        self.min_weather_cloud_filament_ribbon_detail_count =
            min_weather_cloud_filament_ribbon_detail_count;
        self
    }

    pub fn with_visible_authored_world_fixture_count(
        mut self,
        visible_authored_world_fixture_count: usize,
    ) -> Self {
        self.visible_authored_world_fixture_count = visible_authored_world_fixture_count;
        self
    }

    pub fn with_deferred_visual_asset_scene_count(
        mut self,
        deferred_visual_asset_scene_count: usize,
    ) -> Self {
        self.deferred_visual_asset_scene_count = deferred_visual_asset_scene_count;
        self
    }
}
