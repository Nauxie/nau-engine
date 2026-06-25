use crate::movement::FlightMode;
use bevy::prelude::*;

use super::{json_array3, json_number, json_string, vec3_array};

#[derive(Clone, Copy, Debug, Default)]
pub struct EvalObjectiveProgress {
    pub completed_count: usize,
    pub total_count: usize,
    pub current_label: &'static str,
    pub current_distance_m: f32,
    pub complete: bool,
}

impl EvalObjectiveProgress {
    pub fn new(
        completed_count: usize,
        total_count: usize,
        current_label: &'static str,
        current_distance_m: f32,
        complete: bool,
    ) -> Self {
        Self {
            completed_count: completed_count.min(total_count),
            total_count,
            current_label,
            current_distance_m,
            complete,
        }
    }

    pub fn current_step(self) -> usize {
        if self.total_count == 0 {
            0
        } else {
            (self.completed_count + 1).min(self.total_count)
        }
    }

    fn to_json(self) -> String {
        format!(
            "{{\"completed_count\":{},\"total_count\":{},\"current_step\":{},\"current_label\":{},\"current_distance_m\":{},\"complete\":{}}}",
            self.completed_count,
            self.total_count,
            self.current_step(),
            json_string(self.current_label),
            json_number(self.current_distance_m),
            self.complete,
        )
    }
}

#[derive(Clone, Debug)]
pub struct EvalMovementMetrics {
    pub desired_body_yaw_error_degrees: f32,
    pub body_roll_degrees: f32,
    pub desired_heading_alignment_mps: f32,
    pub lateral_response_mps: f32,
    pub lateral_input_active: bool,
    pub movement_axis: Vec2,
}

#[derive(Clone, Debug)]
pub struct EvalSample {
    pub frame: u32,
    pub time_secs: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub speed_mps: f32,
    pub altitude_m: f32,
    pub mode: &'static str,
    pub desired_body_yaw_error_degrees: f32,
    pub desired_body_heading_error_degrees: f32,
    pub body_roll_degrees: f32,
    pub desired_heading_alignment_mps: f32,
    pub lateral_response_mps: f32,
    pub lateral_input_active: bool,
    pub movement_input_lateral_axis: f32,
    pub movement_input_forward_axis: f32,
    pub camera_distance_m: f32,
    pub camera_surface_clearance_m: f32,
    pub camera_player_angle_degrees: f32,
    pub camera_pitch_degrees: f32,
    pub camera_yaw_offset_degrees: f32,
    pub camera_pitch_offset_degrees: f32,
    pub camera_step_distance_m: f32,
    pub camera_rotation_delta_degrees: f32,
    pub camera_orbit_alignment_degrees: f32,
    pub camera_follow_direction_error_degrees: f32,
    pub camera_view_yaw_degrees: f32,
    pub camera_world_yaw_degrees: f32,
    pub camera_obstruction_adjustment_m: f32,
    pub camera_obstruction_hits: usize,
    pub visible_wind_fields: usize,
    pub wind_field_count: usize,
    pub active_lift_fields: usize,
    pub readable_lift_fields: usize,
    pub lift_field_count: usize,
    pub target_distance_m: f32,
    pub on_landing_target: bool,
    pub objective: EvalObjectiveProgress,
    pub sky_island_count: usize,
    pub active_chunk_count: usize,
    pub active_island_count: usize,
    pub near_lod_islands: usize,
    pub mid_lod_islands: usize,
    pub far_lod_islands: usize,
    pub visible_island_terrain_count: usize,
    pub hidden_island_terrain_count: usize,
    pub visible_island_impostor_count: usize,
    pub hidden_island_impostor_count: usize,
    pub visible_island_detail_count: usize,
    pub hidden_island_detail_count: usize,
    pub visible_route_beacon_count: usize,
    pub weather_cloud_count: usize,
    pub environment_motion_visual_count: usize,
    pub max_environment_motion_offset_m: f32,
    pub island_terrain_surface_count: usize,
    pub min_island_terrain_mesh_vertices: usize,
    pub min_island_terrain_color_bands: usize,
    pub min_island_terrain_material_weight_bands: usize,
    pub min_island_terrain_material_channels: usize,
    pub min_island_terrain_material_regions: usize,
    pub min_island_terrain_texture_detail_bands: usize,
    pub min_island_terrain_relief_range_m: f32,
    pub min_island_cliff_color_bands: usize,
    pub min_island_impostor_mesh_vertices: usize,
    pub min_island_impostor_color_bands: usize,
    pub procedural_island_body_count: usize,
    pub primitive_island_body_count: usize,
    pub min_island_body_silhouette_segments: usize,
    pub avg_island_body_silhouette_segments: f32,
    pub min_island_body_mesh_vertices: usize,
    pub max_island_body_mesh_vertices: usize,
    pub generated_ground_cover_patch_count: usize,
    pub min_ground_cover_blade_count: usize,
    pub min_ground_cover_mesh_vertices: usize,
    pub generated_tree_trunk_count: usize,
    pub generated_tree_canopy_count: usize,
    pub min_tree_trunk_mesh_vertices: usize,
    pub min_tree_canopy_mesh_vertices: usize,
    pub detail_biome_palette_count: usize,
    pub generated_rock_count: usize,
    pub min_rock_mesh_vertices: usize,
    pub generated_weather_cloud_count: usize,
    pub generated_weather_cloud_bank_count: usize,
    pub min_weather_cloud_bank_depth_m: f32,
    pub min_weather_cloud_lobe_count: usize,
    pub max_weather_cloud_lobe_count: usize,
    pub min_weather_cloud_mesh_vertices: usize,
    pub resident_island_visual_count: usize,
    pub stream_visibility_changes_this_frame: usize,
    pub max_stream_visibility_changes_per_frame: usize,
    pub total_stream_visibility_changes: usize,
    pub catalog_island_visual_count: usize,
    pub hidden_island_visual_count: usize,
    pub resident_island_visual_fraction: f32,
    pub stream_spawned_visuals_this_frame: usize,
    pub stream_despawned_visuals_this_frame: usize,
    pub max_stream_spawned_visuals_per_frame: usize,
    pub max_stream_despawned_visuals_per_frame: usize,
    pub total_stream_spawned_visuals: usize,
    pub total_stream_despawned_visuals: usize,
    pub entity_count: usize,
    pub visual_asset_slot_count: usize,
    pub gltf_scene_asset_slot_count: usize,
    pub ready_visual_asset_slot_count: usize,
    pub placeholder_visual_asset_slot_count: usize,
    pub streaming_visual_asset_slot_count: usize,
    pub missing_visual_asset_slot_count: usize,
    pub deferred_visual_asset_scene_count: usize,
    pub queued_visual_asset_scene_count: usize,
    pub loading_visual_asset_scene_count: usize,
    pub loaded_visual_asset_scene_count: usize,
    pub dependency_loaded_visual_asset_scene_count: usize,
    pub preload_ready_visual_asset_scene_count: usize,
    pub failed_visual_asset_scene_count: usize,
    pub spawned_visual_asset_scene_count: usize,
    pub ready_visual_asset_scene_count: usize,
    pub visible_authored_world_fixture_count: usize,
    pub always_visual_asset_slot_count: usize,
    pub stream_window_visual_asset_slot_count: usize,
    pub near_lod_visual_asset_slot_count: usize,
    pub far_lod_visual_asset_slot_count: usize,
    pub weather_visual_asset_slot_count: usize,
    pub always_preload_ready_visual_asset_slot_count: usize,
    pub streaming_preload_ready_visual_asset_slot_count: usize,
    pub declared_animation_clip_count: usize,
    pub ready_animation_clip_count: usize,
    pub animation_player_count: usize,
    pub animation_graph_count: usize,
    pub power_up_count: usize,
    pub visible_power_up_count: usize,
    pub collected_power_up_count: usize,
    pub active_power_up_effects: usize,
    pub total_power_up_activations: usize,
    pub visual_foot_gap_m: f32,
}

impl EvalSample {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        frame: u32,
        fixed_dt: f32,
        position: Vec3,
        velocity: Vec3,
        mode: FlightMode,
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
            desired_body_yaw_error_degrees: f32::NAN,
            desired_body_heading_error_degrees: f32::NAN,
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
            island_terrain_surface_count: 0,
            min_island_terrain_mesh_vertices: 0,
            min_island_terrain_color_bands: 0,
            min_island_terrain_material_weight_bands: 0,
            min_island_terrain_material_channels: 0,
            min_island_terrain_material_regions: 0,
            min_island_terrain_texture_detail_bands: 0,
            min_island_terrain_relief_range_m: 0.0,
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
            generated_weather_cloud_count: 0,
            generated_weather_cloud_bank_count: 0,
            min_weather_cloud_bank_depth_m: 0.0,
            min_weather_cloud_lobe_count: 0,
            max_weather_cloud_lobe_count: 0,
            min_weather_cloud_mesh_vertices: 0,
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
        self.body_roll_degrees = metrics.body_roll_degrees;
        self.desired_heading_alignment_mps = metrics.desired_heading_alignment_mps;
        self.lateral_response_mps = metrics.lateral_response_mps;
        self.lateral_input_active = metrics.lateral_input_active;
        self.movement_input_lateral_axis = metrics.movement_axis.x;
        self.movement_input_forward_axis = metrics.movement_axis.y;
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
    pub fn with_content_metrics(
        mut self,
        island_terrain_surface_count: usize,
        min_island_terrain_mesh_vertices: usize,
        min_island_terrain_color_bands: usize,
        min_island_terrain_relief_range_m: f32,
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
        generated_weather_cloud_count: usize,
        generated_weather_cloud_bank_count: usize,
        min_weather_cloud_bank_depth_m: f32,
        min_weather_cloud_lobe_count: usize,
        max_weather_cloud_lobe_count: usize,
        min_weather_cloud_mesh_vertices: usize,
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
        self.generated_weather_cloud_count = generated_weather_cloud_count;
        self.generated_weather_cloud_bank_count = generated_weather_cloud_bank_count;
        self.min_weather_cloud_bank_depth_m = min_weather_cloud_bank_depth_m;
        self.min_weather_cloud_lobe_count = min_weather_cloud_lobe_count;
        self.max_weather_cloud_lobe_count = max_weather_cloud_lobe_count;
        self.min_weather_cloud_mesh_vertices = min_weather_cloud_mesh_vertices;
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

    pub fn to_json(&self) -> String {
        let json = format!(
            "{{\"frame\":{},\"time_secs\":{},\"position\":{},\"velocity\":{},\"speed_mps\":{},\"altitude_m\":{},\"mode\":{},\"desired_body_yaw_error_degrees\":{},\"desired_body_heading_error_degrees\":{},\"body_roll_degrees\":{},\"desired_heading_alignment_mps\":{},\"lateral_response_mps\":{},\"lateral_input_active\":{},\"movement_input_lateral_axis\":{},\"movement_input_forward_axis\":{},\"camera_distance_m\":{},\"camera_surface_clearance_m\":{},\"camera_player_angle_degrees\":{},\"camera_pitch_degrees\":{},\"camera_yaw_offset_degrees\":{},\"camera_pitch_offset_degrees\":{},\"camera_step_distance_m\":{},\"camera_rotation_delta_degrees\":{},\"camera_orbit_alignment_degrees\":{},\"camera_follow_direction_error_degrees\":{},\"camera_view_yaw_degrees\":{},\"camera_world_yaw_degrees\":{},\"camera_obstruction_adjustment_m\":{},\"camera_obstruction_hits\":{},\"visible_wind_fields\":{},\"wind_field_count\":{},\"active_lift_fields\":{},\"readable_lift_fields\":{},\"lift_field_count\":{},\"target_distance_m\":{},\"on_landing_target\":{},\"objective\":{},\"sky_island_count\":{},\"active_chunk_count\":{},\"active_island_count\":{},\"near_lod_islands\":{},\"mid_lod_islands\":{},\"far_lod_islands\":{},\"visible_island_terrain_count\":{},\"hidden_island_terrain_count\":{},\"visible_island_impostor_count\":{},\"hidden_island_impostor_count\":{},\"visible_island_detail_count\":{},\"hidden_island_detail_count\":{},\"visible_route_beacon_count\":{},\"weather_cloud_count\":{},\"environment_motion_visual_count\":{},\"max_environment_motion_offset_m\":{},\"island_terrain_surface_count\":{},\"min_island_terrain_mesh_vertices\":{},\"min_island_terrain_color_bands\":{},\"min_island_terrain_material_weight_bands\":{},\"min_island_terrain_material_channels\":{},\"min_island_terrain_material_regions\":{},\"min_island_terrain_texture_detail_bands\":{},\"min_island_terrain_relief_range_m\":{},\"min_island_cliff_color_bands\":{},\"procedural_island_body_count\":{},\"primitive_island_body_count\":{},\"min_island_body_silhouette_segments\":{},\"avg_island_body_silhouette_segments\":{},\"max_island_body_mesh_vertices\":{},\"generated_ground_cover_patch_count\":{},\"min_ground_cover_blade_count\":{},\"min_ground_cover_mesh_vertices\":{},\"generated_tree_trunk_count\":{},\"generated_tree_canopy_count\":{},\"min_tree_trunk_mesh_vertices\":{},\"min_tree_canopy_mesh_vertices\":{},\"detail_biome_palette_count\":{},\"generated_rock_count\":{},\"min_rock_mesh_vertices\":{},\"generated_weather_cloud_count\":{},\"generated_weather_cloud_bank_count\":{},\"min_weather_cloud_bank_depth_m\":{},\"min_weather_cloud_lobe_count\":{},\"max_weather_cloud_lobe_count\":{},\"min_weather_cloud_mesh_vertices\":{},\"resident_island_visual_count\":{},\"stream_visibility_changes_this_frame\":{},\"max_stream_visibility_changes_per_frame\":{},\"total_stream_visibility_changes\":{},\"catalog_island_visual_count\":{},\"hidden_island_visual_count\":{},\"resident_island_visual_fraction\":{},\"stream_spawned_visuals_this_frame\":{},\"stream_despawned_visuals_this_frame\":{},\"max_stream_spawned_visuals_per_frame\":{},\"max_stream_despawned_visuals_per_frame\":{},\"total_stream_spawned_visuals\":{},\"total_stream_despawned_visuals\":{},\"entity_count\":{},\"visual_asset_slot_count\":{},\"gltf_scene_asset_slot_count\":{},\"ready_visual_asset_slot_count\":{},\"placeholder_visual_asset_slot_count\":{},\"streaming_visual_asset_slot_count\":{},\"missing_visual_asset_slot_count\":{},\"queued_visual_asset_scene_count\":{},\"loading_visual_asset_scene_count\":{},\"loaded_visual_asset_scene_count\":{},\"dependency_loaded_visual_asset_scene_count\":{},\"preload_ready_visual_asset_scene_count\":{},\"failed_visual_asset_scene_count\":{},\"spawned_visual_asset_scene_count\":{},\"ready_visual_asset_scene_count\":{},\"visible_authored_world_fixture_count\":{},\"always_visual_asset_slot_count\":{},\"stream_window_visual_asset_slot_count\":{},\"near_lod_visual_asset_slot_count\":{},\"far_lod_visual_asset_slot_count\":{},\"weather_visual_asset_slot_count\":{},\"always_preload_ready_visual_asset_slot_count\":{},\"streaming_preload_ready_visual_asset_slot_count\":{},\"declared_animation_clip_count\":{},\"ready_animation_clip_count\":{},\"animation_player_count\":{},\"animation_graph_count\":{},\"power_up_count\":{},\"visible_power_up_count\":{},\"collected_power_up_count\":{},\"active_power_up_effects\":{},\"total_power_up_activations\":{},\"visual_foot_gap_m\":{}}}",
            self.frame,
            json_number(self.time_secs),
            json_array3(self.position),
            json_array3(self.velocity),
            json_number(self.speed_mps),
            json_number(self.altitude_m),
            json_string(self.mode),
            json_number(self.desired_body_yaw_error_degrees),
            json_number(self.desired_body_heading_error_degrees),
            json_number(self.body_roll_degrees),
            json_number(self.desired_heading_alignment_mps),
            json_number(self.lateral_response_mps),
            self.lateral_input_active,
            json_number(self.movement_input_lateral_axis),
            json_number(self.movement_input_forward_axis),
            json_number(self.camera_distance_m),
            json_number(self.camera_surface_clearance_m),
            json_number(self.camera_player_angle_degrees),
            json_number(self.camera_pitch_degrees),
            json_number(self.camera_yaw_offset_degrees),
            json_number(self.camera_pitch_offset_degrees),
            json_number(self.camera_step_distance_m),
            json_number(self.camera_rotation_delta_degrees),
            json_number(self.camera_orbit_alignment_degrees),
            json_number(self.camera_follow_direction_error_degrees),
            json_number(self.camera_view_yaw_degrees),
            json_number(self.camera_world_yaw_degrees),
            json_number(self.camera_obstruction_adjustment_m),
            self.camera_obstruction_hits,
            self.visible_wind_fields,
            self.wind_field_count,
            self.active_lift_fields,
            self.readable_lift_fields,
            self.lift_field_count,
            json_number(self.target_distance_m),
            self.on_landing_target,
            self.objective.to_json(),
            self.sky_island_count,
            self.active_chunk_count,
            self.active_island_count,
            self.near_lod_islands,
            self.mid_lod_islands,
            self.far_lod_islands,
            self.visible_island_terrain_count,
            self.hidden_island_terrain_count,
            self.visible_island_impostor_count,
            self.hidden_island_impostor_count,
            self.visible_island_detail_count,
            self.hidden_island_detail_count,
            self.visible_route_beacon_count,
            self.weather_cloud_count,
            self.environment_motion_visual_count,
            json_number(self.max_environment_motion_offset_m),
            self.island_terrain_surface_count,
            self.min_island_terrain_mesh_vertices,
            self.min_island_terrain_color_bands,
            self.min_island_terrain_material_weight_bands,
            self.min_island_terrain_material_channels,
            self.min_island_terrain_material_regions,
            self.min_island_terrain_texture_detail_bands,
            json_number(self.min_island_terrain_relief_range_m),
            self.min_island_cliff_color_bands,
            self.procedural_island_body_count,
            self.primitive_island_body_count,
            self.min_island_body_silhouette_segments,
            json_number(self.avg_island_body_silhouette_segments),
            self.max_island_body_mesh_vertices,
            self.generated_ground_cover_patch_count,
            self.min_ground_cover_blade_count,
            self.min_ground_cover_mesh_vertices,
            self.generated_tree_trunk_count,
            self.generated_tree_canopy_count,
            self.min_tree_trunk_mesh_vertices,
            self.min_tree_canopy_mesh_vertices,
            self.detail_biome_palette_count,
            self.generated_rock_count,
            self.min_rock_mesh_vertices,
            self.generated_weather_cloud_count,
            self.generated_weather_cloud_bank_count,
            json_number(self.min_weather_cloud_bank_depth_m),
            self.min_weather_cloud_lobe_count,
            self.max_weather_cloud_lobe_count,
            self.min_weather_cloud_mesh_vertices,
            self.resident_island_visual_count,
            self.stream_visibility_changes_this_frame,
            self.max_stream_visibility_changes_per_frame,
            self.total_stream_visibility_changes,
            self.catalog_island_visual_count,
            self.hidden_island_visual_count,
            json_number(self.resident_island_visual_fraction),
            self.stream_spawned_visuals_this_frame,
            self.stream_despawned_visuals_this_frame,
            self.max_stream_spawned_visuals_per_frame,
            self.max_stream_despawned_visuals_per_frame,
            self.total_stream_spawned_visuals,
            self.total_stream_despawned_visuals,
            self.entity_count,
            self.visual_asset_slot_count,
            self.gltf_scene_asset_slot_count,
            self.ready_visual_asset_slot_count,
            self.placeholder_visual_asset_slot_count,
            self.streaming_visual_asset_slot_count,
            self.missing_visual_asset_slot_count,
            self.queued_visual_asset_scene_count,
            self.loading_visual_asset_scene_count,
            self.loaded_visual_asset_scene_count,
            self.dependency_loaded_visual_asset_scene_count,
            self.preload_ready_visual_asset_scene_count,
            self.failed_visual_asset_scene_count,
            self.spawned_visual_asset_scene_count,
            self.ready_visual_asset_scene_count,
            self.visible_authored_world_fixture_count,
            self.always_visual_asset_slot_count,
            self.stream_window_visual_asset_slot_count,
            self.near_lod_visual_asset_slot_count,
            self.far_lod_visual_asset_slot_count,
            self.weather_visual_asset_slot_count,
            self.always_preload_ready_visual_asset_slot_count,
            self.streaming_preload_ready_visual_asset_slot_count,
            self.declared_animation_clip_count,
            self.ready_animation_clip_count,
            self.animation_player_count,
            self.animation_graph_count,
            self.power_up_count,
            self.visible_power_up_count,
            self.collected_power_up_count,
            self.active_power_up_effects,
            self.total_power_up_activations,
            json_number(self.visual_foot_gap_m),
        );
        let impostor_count_key = "\"visible_island_detail_count\"";
        let impostor_metrics = format!(
            "\"min_island_impostor_mesh_vertices\":{},\"min_island_impostor_color_bands\":{},{}",
            self.min_island_impostor_mesh_vertices,
            self.min_island_impostor_color_bands,
            impostor_count_key
        );
        let json = json.replacen(impostor_count_key, &impostor_metrics, 1);
        let body_mesh_key = "\"max_island_body_mesh_vertices\"";
        let body_mesh_metrics = format!(
            "\"min_island_body_mesh_vertices\":{},{}",
            self.min_island_body_mesh_vertices, body_mesh_key
        );
        let json = json.replacen(body_mesh_key, &body_mesh_metrics, 1);
        let deferred_asset_key = "\"queued_visual_asset_scene_count\"";
        let deferred_asset_metrics = format!(
            "\"deferred_visual_asset_scene_count\":{},{}",
            self.deferred_visual_asset_scene_count, deferred_asset_key
        );
        json.replacen(deferred_asset_key, &deferred_asset_metrics, 1)
    }
}
