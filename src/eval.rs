use crate::{
    asset_pipeline::{
        MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT, MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
        MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
        MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT, MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
        MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
        MIN_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ASSET_SLOT_COUNT,
        MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT, MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
        MIN_VISUAL_ANIMATION_GRAPH_COUNT, MIN_VISUAL_ANIMATION_PLAYER_COUNT,
    },
    movement::FlightMode,
};
use bevy::prelude::*;

mod scenarios;
mod thresholds;
pub use scenarios::{
    AIR_CONTROL_RESPONSE, BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE, CAMERA_MOUSE_CONTROL,
    CAMERA_STRAFE_STABILITY, CAMERA_TURN_STABILITY, CAMERA_YAW_STABILITY, EvalCheckpoint,
    EvalScenario, GROUND_TAXI_CONTROL, ISLAND_LAUNCH_TO_LANDING, LONG_GLIDE_VISIBILITY,
    SCENARIO_NAMES, UPDRAFT_ROUTE, scenario_named, scripted_camera_input, scripted_input,
};
use thresholds::*;
pub use thresholds::{EvalThresholds, MAX_RESIDENT_ISLAND_VISUAL_FRACTION};
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

#[derive(Default, Clone, Debug)]
pub struct EvalAccumulator {
    first_sample: Option<EvalSample>,
    final_sample: Option<EvalSample>,
    frame_times_ms: Vec<f32>,
    sample_count: u32,
    max_altitude_m: f32,
    min_altitude_m: f32,
    max_grounded_visual_foot_gap_m: f32,
    max_speed_mps: f32,
    max_camera_distance_m: f32,
    min_camera_surface_clearance_m: f32,
    max_camera_player_angle_degrees: f32,
    max_camera_step_distance_m: f32,
    max_camera_rotation_delta_degrees: f32,
    max_camera_orbit_alignment_degrees: f32,
    camera_follow_direction_error_sum_degrees: f32,
    camera_follow_direction_error_samples: u32,
    camera_follow_direction_error_values_degrees: Vec<f32>,
    max_camera_follow_direction_error_degrees: f32,
    max_abs_camera_view_yaw_degrees: f32,
    first_camera_view_yaw_degrees: Option<f32>,
    max_camera_view_yaw_drift_degrees: f32,
    first_camera_world_yaw_degrees: Option<f32>,
    max_camera_world_yaw_drift_degrees: f32,
    max_camera_obstruction_adjustment_m: f32,
    max_camera_obstruction_hits: usize,
    desired_body_heading_error_sum_degrees: f32,
    desired_body_heading_samples: u32,
    desired_body_heading_error_values_degrees: Vec<f32>,
    max_desired_body_heading_error_degrees: f32,
    previous_desired_body_yaw_error_degrees: Option<f32>,
    max_body_yaw_error_step_degrees: f32,
    previous_body_yaw_error_sign: Option<f32>,
    body_yaw_oscillation_count: u32,
    previous_body_roll_degrees: Option<f32>,
    max_body_roll_step_degrees: f32,
    max_right_body_bank_degrees: f32,
    max_left_body_bank_degrees: f32,
    max_desired_heading_alignment_mps: f32,
    max_lateral_response_mps: f32,
    first_lateral_input_time_secs: Option<f32>,
    first_lateral_response_time_secs: Option<f32>,
    max_right_lateral_response_mps: f32,
    first_right_lateral_input_time_secs: Option<f32>,
    first_right_lateral_response_time_secs: Option<f32>,
    max_left_lateral_response_mps: f32,
    first_left_lateral_input_time_secs: Option<f32>,
    first_left_lateral_response_time_secs: Option<f32>,
    max_backward_lateral_response_mps: f32,
    first_backward_lateral_input_time_secs: Option<f32>,
    first_backward_lateral_response_time_secs: Option<f32>,
    max_backward_right_lateral_response_mps: f32,
    max_backward_right_rear_response_mps: f32,
    first_backward_right_lateral_input_time_secs: Option<f32>,
    first_backward_right_lateral_response_time_secs: Option<f32>,
    max_backward_left_lateral_response_mps: f32,
    max_backward_left_rear_response_mps: f32,
    first_backward_left_lateral_input_time_secs: Option<f32>,
    first_backward_left_lateral_response_time_secs: Option<f32>,
    backward_air_control_start_speed_mps: Option<f32>,
    min_backward_air_control_speed_mps: Option<f32>,
    backward_air_control_start_planar_speed_mps: Option<f32>,
    min_backward_air_control_planar_speed_mps: Option<f32>,
    max_air_brake_speed_drop_mps: f32,
    max_air_brake_planar_speed_drop_mps: f32,
    max_post_brake_forward_alignment_mps: f32,
    min_target_distance_m: f32,
    min_camera_pitch_degrees: f32,
    max_camera_pitch_degrees: f32,
    max_abs_camera_yaw_offset_degrees: f32,
    min_camera_pitch_offset_degrees: f32,
    max_camera_pitch_offset_degrees: f32,
    max_visible_wind_fields: usize,
    max_active_lift_fields: usize,
    max_readable_lift_fields: usize,
    max_sky_island_count: usize,
    max_active_chunk_count: usize,
    max_active_island_count: usize,
    max_near_lod_islands: usize,
    max_mid_lod_islands: usize,
    max_far_lod_islands: usize,
    max_visible_island_terrain_count: usize,
    max_hidden_island_terrain_count: usize,
    max_visible_island_impostor_count: usize,
    max_hidden_island_impostor_count: usize,
    max_visible_island_detail_count: usize,
    max_hidden_island_detail_count: usize,
    max_visible_route_beacon_count: usize,
    max_weather_cloud_count: usize,
    max_environment_motion_visual_count: usize,
    max_environment_motion_offset_m: f32,
    min_island_terrain_surface_count: usize,
    min_island_terrain_mesh_vertices: usize,
    min_island_terrain_color_bands: usize,
    min_island_terrain_material_weight_bands: usize,
    min_island_terrain_material_channels: usize,
    min_island_terrain_material_regions: usize,
    min_island_terrain_texture_detail_bands: usize,
    min_island_terrain_relief_range_m: f32,
    min_island_cliff_color_bands: usize,
    min_island_impostor_mesh_vertices: usize,
    min_island_impostor_color_bands: usize,
    min_procedural_island_body_count: usize,
    max_primitive_island_body_count: usize,
    min_island_body_silhouette_segments: usize,
    max_avg_island_body_silhouette_segments: f32,
    min_island_body_mesh_vertices: usize,
    max_island_body_mesh_vertices: usize,
    min_generated_ground_cover_patch_count: usize,
    min_ground_cover_blade_count: usize,
    min_ground_cover_mesh_vertices: usize,
    min_generated_tree_trunk_count: usize,
    min_generated_tree_canopy_count: usize,
    min_tree_trunk_mesh_vertices: usize,
    min_tree_canopy_mesh_vertices: usize,
    min_detail_biome_palette_count: usize,
    min_generated_rock_count: usize,
    min_rock_mesh_vertices: usize,
    min_generated_weather_cloud_count: usize,
    min_generated_weather_cloud_bank_count: usize,
    min_weather_cloud_bank_depth_m: f32,
    min_weather_cloud_lobe_count: usize,
    min_max_weather_cloud_lobe_count: usize,
    min_weather_cloud_mesh_vertices: usize,
    max_resident_island_visual_count: usize,
    max_stream_visibility_changes_per_frame: usize,
    total_stream_visibility_changes: usize,
    max_catalog_island_visual_count: usize,
    max_hidden_island_visual_count: usize,
    max_resident_island_visual_fraction: f32,
    max_stream_spawned_visuals_per_frame: usize,
    max_stream_despawned_visuals_per_frame: usize,
    total_stream_spawned_visuals: usize,
    total_stream_despawned_visuals: usize,
    max_entity_count: usize,
    max_objective_total_count: usize,
    max_completed_objective_count: usize,
    min_objective_distance_m: f32,
    objective_complete_samples: u32,
    max_visual_asset_slot_count: usize,
    max_gltf_scene_asset_slot_count: usize,
    max_ready_visual_asset_slot_count: usize,
    max_placeholder_visual_asset_slot_count: usize,
    max_streaming_visual_asset_slot_count: usize,
    max_missing_visual_asset_slot_count: usize,
    max_deferred_visual_asset_scene_count: usize,
    max_queued_visual_asset_scene_count: usize,
    max_loading_visual_asset_scene_count: usize,
    max_loaded_visual_asset_scene_count: usize,
    max_dependency_loaded_visual_asset_scene_count: usize,
    max_preload_ready_visual_asset_scene_count: usize,
    max_failed_visual_asset_scene_count: usize,
    max_spawned_visual_asset_scene_count: usize,
    max_ready_visual_asset_scene_count: usize,
    max_visible_authored_world_fixture_count: usize,
    max_always_visual_asset_slot_count: usize,
    max_stream_window_visual_asset_slot_count: usize,
    max_near_lod_visual_asset_slot_count: usize,
    max_far_lod_visual_asset_slot_count: usize,
    max_weather_visual_asset_slot_count: usize,
    max_always_preload_ready_visual_asset_slot_count: usize,
    max_streaming_preload_ready_visual_asset_slot_count: usize,
    max_declared_animation_clip_count: usize,
    max_ready_animation_clip_count: usize,
    max_animation_player_count: usize,
    max_animation_graph_count: usize,
    max_power_up_count: usize,
    min_visible_power_up_count: usize,
    max_collected_power_up_count: usize,
    power_up_effect_samples: u32,
    total_power_up_activations: usize,
    target_landing_samples: u32,
    lifted_samples: u32,
    readable_lift_samples: u32,
    unreadable_lift_samples: u32,
    gliding_samples: u32,
    launching_samples: u32,
    grounded_samples: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct EvalFrameTimeStats {
    avg_ms: f32,
    p95_ms: f32,
    p99_ms: f32,
    max_ms: f32,
}

impl EvalFrameTimeStats {
    fn from_samples(samples: &[f32]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(f32::total_cmp);

        let sum: f32 = sorted.iter().sum();
        Self {
            avg_ms: sum / sorted.len() as f32,
            p95_ms: percentile(&sorted, 0.95),
            p99_ms: percentile(&sorted, 0.99),
            max_ms: *sorted.last().unwrap_or(&0.0),
        }
    }
}

fn response_latency_secs(
    input_time_secs: Option<f32>,
    response_time_secs: Option<f32>,
    scenario: EvalScenario,
) -> f32 {
    match (input_time_secs, response_time_secs) {
        (Some(input_time), Some(response_time)) => (response_time - input_time).max(0.0),
        (Some(_), None) => scenario.duration_secs(),
        _ => 0.0,
    }
}

fn backward_diagonal_rear_response_mps(sample: &EvalSample) -> Option<f32> {
    if sample.movement_input_forward_axis >= 0.0
        || sample.movement_input_lateral_axis.abs() <= f32::EPSILON
        || !sample.desired_heading_alignment_mps.is_finite()
        || !sample.lateral_response_mps.is_finite()
    {
        return None;
    }

    Some(
        sample.desired_heading_alignment_mps * std::f32::consts::SQRT_2
            - sample.lateral_response_mps,
    )
}

impl EvalAccumulator {
    pub fn observe_frame_time_ms(&mut self, frame_time_ms: f32) {
        if frame_time_ms.is_finite() && frame_time_ms >= 0.0 {
            self.frame_times_ms.push(frame_time_ms);
        }
    }

    pub fn observe(&mut self, sample: EvalSample) {
        if self.first_sample.is_none() {
            self.first_sample = Some(sample.clone());
            self.min_altitude_m = sample.altitude_m;
            self.min_camera_surface_clearance_m = sample.camera_surface_clearance_m;
            self.min_target_distance_m = sample.target_distance_m;
            self.min_objective_distance_m = sample.objective.current_distance_m;
            self.min_camera_pitch_degrees = sample.camera_pitch_degrees;
            self.max_camera_pitch_degrees = sample.camera_pitch_degrees;
            self.min_camera_pitch_offset_degrees = sample.camera_pitch_offset_degrees;
            self.max_camera_pitch_offset_degrees = sample.camera_pitch_offset_degrees;
            self.min_visible_power_up_count = sample.visible_power_up_count;
            self.min_island_terrain_surface_count = sample.island_terrain_surface_count;
            self.min_island_terrain_mesh_vertices = sample.min_island_terrain_mesh_vertices;
            self.min_island_terrain_color_bands = sample.min_island_terrain_color_bands;
            self.min_island_terrain_material_weight_bands =
                sample.min_island_terrain_material_weight_bands;
            self.min_island_terrain_material_channels = sample.min_island_terrain_material_channels;
            self.min_island_terrain_material_regions = sample.min_island_terrain_material_regions;
            self.min_island_terrain_texture_detail_bands =
                sample.min_island_terrain_texture_detail_bands;
            self.min_island_terrain_relief_range_m = sample.min_island_terrain_relief_range_m;
            self.min_island_cliff_color_bands = sample.min_island_cliff_color_bands;
            self.min_island_impostor_mesh_vertices = sample.min_island_impostor_mesh_vertices;
            self.min_island_impostor_color_bands = sample.min_island_impostor_color_bands;
            self.min_procedural_island_body_count = sample.procedural_island_body_count;
            self.min_island_body_silhouette_segments = sample.min_island_body_silhouette_segments;
            self.min_island_body_mesh_vertices = sample.min_island_body_mesh_vertices;
            self.min_generated_ground_cover_patch_count = sample.generated_ground_cover_patch_count;
            self.min_ground_cover_blade_count = sample.min_ground_cover_blade_count;
            self.min_ground_cover_mesh_vertices = sample.min_ground_cover_mesh_vertices;
            self.min_generated_tree_trunk_count = sample.generated_tree_trunk_count;
            self.min_generated_tree_canopy_count = sample.generated_tree_canopy_count;
            self.min_tree_trunk_mesh_vertices = sample.min_tree_trunk_mesh_vertices;
            self.min_tree_canopy_mesh_vertices = sample.min_tree_canopy_mesh_vertices;
            self.min_detail_biome_palette_count = sample.detail_biome_palette_count;
            self.min_generated_rock_count = sample.generated_rock_count;
            self.min_rock_mesh_vertices = sample.min_rock_mesh_vertices;
            self.min_generated_weather_cloud_count = sample.generated_weather_cloud_count;
            self.min_generated_weather_cloud_bank_count = sample.generated_weather_cloud_bank_count;
            self.min_weather_cloud_bank_depth_m = sample.min_weather_cloud_bank_depth_m;
            self.min_weather_cloud_lobe_count = sample.min_weather_cloud_lobe_count;
            self.min_max_weather_cloud_lobe_count = sample.max_weather_cloud_lobe_count;
            self.min_weather_cloud_mesh_vertices = sample.min_weather_cloud_mesh_vertices;
        }

        self.sample_count += 1;
        self.max_altitude_m = self.max_altitude_m.max(sample.altitude_m);
        self.min_altitude_m = self.min_altitude_m.min(sample.altitude_m);
        if sample.mode == FlightMode::Grounded.label() && sample.visual_foot_gap_m.is_finite() {
            self.max_grounded_visual_foot_gap_m = self
                .max_grounded_visual_foot_gap_m
                .max(sample.visual_foot_gap_m.abs());
        }
        self.max_speed_mps = self.max_speed_mps.max(sample.speed_mps);
        self.max_camera_distance_m = self.max_camera_distance_m.max(sample.camera_distance_m);
        self.min_camera_surface_clearance_m = self
            .min_camera_surface_clearance_m
            .min(sample.camera_surface_clearance_m);
        self.max_camera_player_angle_degrees = self
            .max_camera_player_angle_degrees
            .max(sample.camera_player_angle_degrees);
        self.max_camera_step_distance_m = self
            .max_camera_step_distance_m
            .max(sample.camera_step_distance_m);
        self.max_camera_rotation_delta_degrees = self
            .max_camera_rotation_delta_degrees
            .max(sample.camera_rotation_delta_degrees);
        self.max_camera_orbit_alignment_degrees = self
            .max_camera_orbit_alignment_degrees
            .max(sample.camera_orbit_alignment_degrees);
        if sample.camera_follow_direction_error_degrees.is_finite() {
            self.camera_follow_direction_error_sum_degrees +=
                sample.camera_follow_direction_error_degrees;
            self.camera_follow_direction_error_samples += 1;
            self.camera_follow_direction_error_values_degrees
                .push(sample.camera_follow_direction_error_degrees);
            self.max_camera_follow_direction_error_degrees = self
                .max_camera_follow_direction_error_degrees
                .max(sample.camera_follow_direction_error_degrees);
        }
        self.max_abs_camera_view_yaw_degrees = self
            .max_abs_camera_view_yaw_degrees
            .max(sample.camera_view_yaw_degrees.abs());
        if sample.camera_view_yaw_degrees.is_finite() {
            let first_yaw = self
                .first_camera_view_yaw_degrees
                .get_or_insert(sample.camera_view_yaw_degrees);
            self.max_camera_view_yaw_drift_degrees = self
                .max_camera_view_yaw_drift_degrees
                .max((sample.camera_view_yaw_degrees - *first_yaw).abs());
        }
        if sample.camera_world_yaw_degrees.is_finite() {
            let first_world_yaw = self
                .first_camera_world_yaw_degrees
                .get_or_insert(sample.camera_world_yaw_degrees);
            self.max_camera_world_yaw_drift_degrees = self
                .max_camera_world_yaw_drift_degrees
                .max((sample.camera_world_yaw_degrees - *first_world_yaw).abs());
        }
        self.max_camera_obstruction_adjustment_m = self
            .max_camera_obstruction_adjustment_m
            .max(sample.camera_obstruction_adjustment_m);
        self.max_camera_obstruction_hits = self
            .max_camera_obstruction_hits
            .max(sample.camera_obstruction_hits);
        if sample.desired_body_yaw_error_degrees.is_finite() {
            let heading_error = sample.desired_body_heading_error_degrees;
            self.desired_body_heading_error_sum_degrees += heading_error;
            self.desired_body_heading_samples += 1;
            self.desired_body_heading_error_values_degrees
                .push(heading_error);
            self.max_desired_body_heading_error_degrees = self
                .max_desired_body_heading_error_degrees
                .max(heading_error);
            if let Some(previous_error) = self.previous_desired_body_yaw_error_degrees {
                self.max_body_yaw_error_step_degrees = self
                    .max_body_yaw_error_step_degrees
                    .max((sample.desired_body_yaw_error_degrees - previous_error).abs());
            }
            self.previous_desired_body_yaw_error_degrees =
                Some(sample.desired_body_yaw_error_degrees);

            if sample.desired_body_yaw_error_degrees.abs()
                >= AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES
            {
                let sign = sample.desired_body_yaw_error_degrees.signum();
                if self
                    .previous_body_yaw_error_sign
                    .is_some_and(|previous| previous != sign)
                {
                    self.body_yaw_oscillation_count += 1;
                }
                self.previous_body_yaw_error_sign = Some(sign);
            }
        }
        if !sample.body_roll_degrees.is_finite() || sample.mode == FlightMode::Grounded.label() {
            self.previous_body_roll_degrees = None;
        } else {
            if let Some(previous_roll) = self.previous_body_roll_degrees {
                self.max_body_roll_step_degrees = self
                    .max_body_roll_step_degrees
                    .max((sample.body_roll_degrees - previous_roll).abs());
            }
            self.previous_body_roll_degrees = Some(sample.body_roll_degrees);

            match sample.movement_input_lateral_axis.signum() {
                sign if sign > 0.0 => {
                    self.max_right_body_bank_degrees = self
                        .max_right_body_bank_degrees
                        .max((-sample.body_roll_degrees).max(0.0));
                }
                sign if sign < 0.0 => {
                    self.max_left_body_bank_degrees = self
                        .max_left_body_bank_degrees
                        .max(sample.body_roll_degrees.max(0.0));
                }
                _ => {}
            }
        }
        if sample.desired_heading_alignment_mps.is_finite() {
            self.max_desired_heading_alignment_mps = self
                .max_desired_heading_alignment_mps
                .max(sample.desired_heading_alignment_mps);
        }
        let lateral_axis_active =
            sample.lateral_input_active || sample.movement_input_lateral_axis.abs() > f32::EPSILON;
        if lateral_axis_active {
            if self.first_lateral_input_time_secs.is_none() {
                self.first_lateral_input_time_secs = Some(sample.time_secs);
            }
            self.max_lateral_response_mps = self
                .max_lateral_response_mps
                .max(sample.lateral_response_mps);
            if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                && self.first_lateral_response_time_secs.is_none()
            {
                self.first_lateral_response_time_secs = Some(sample.time_secs);
            }
            match sample.movement_input_lateral_axis.signum() {
                sign if sign > 0.0 => {
                    if self.first_right_lateral_input_time_secs.is_none() {
                        self.first_right_lateral_input_time_secs = Some(sample.time_secs);
                    }
                    self.max_right_lateral_response_mps = self
                        .max_right_lateral_response_mps
                        .max(sample.lateral_response_mps);
                    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                        && self.first_right_lateral_response_time_secs.is_none()
                    {
                        self.first_right_lateral_response_time_secs = Some(sample.time_secs);
                    }
                    if sample.movement_input_forward_axis < 0.0 {
                        if self.first_backward_right_lateral_input_time_secs.is_none() {
                            self.first_backward_right_lateral_input_time_secs =
                                Some(sample.time_secs);
                        }
                        self.max_backward_right_lateral_response_mps = self
                            .max_backward_right_lateral_response_mps
                            .max(sample.lateral_response_mps);
                        if let Some(rear_response_mps) =
                            backward_diagonal_rear_response_mps(&sample)
                        {
                            self.max_backward_right_rear_response_mps = self
                                .max_backward_right_rear_response_mps
                                .max(rear_response_mps);
                        }
                        if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                            && self
                                .first_backward_right_lateral_response_time_secs
                                .is_none()
                        {
                            self.first_backward_right_lateral_response_time_secs =
                                Some(sample.time_secs);
                        }
                    }
                }
                sign if sign < 0.0 => {
                    if self.first_left_lateral_input_time_secs.is_none() {
                        self.first_left_lateral_input_time_secs = Some(sample.time_secs);
                    }
                    self.max_left_lateral_response_mps = self
                        .max_left_lateral_response_mps
                        .max(sample.lateral_response_mps);
                    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                        && self.first_left_lateral_response_time_secs.is_none()
                    {
                        self.first_left_lateral_response_time_secs = Some(sample.time_secs);
                    }
                    if sample.movement_input_forward_axis < 0.0 {
                        if self.first_backward_left_lateral_input_time_secs.is_none() {
                            self.first_backward_left_lateral_input_time_secs =
                                Some(sample.time_secs);
                        }
                        self.max_backward_left_lateral_response_mps = self
                            .max_backward_left_lateral_response_mps
                            .max(sample.lateral_response_mps);
                        if let Some(rear_response_mps) =
                            backward_diagonal_rear_response_mps(&sample)
                        {
                            self.max_backward_left_rear_response_mps = self
                                .max_backward_left_rear_response_mps
                                .max(rear_response_mps);
                        }
                        if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                            && self
                                .first_backward_left_lateral_response_time_secs
                                .is_none()
                        {
                            self.first_backward_left_lateral_response_time_secs =
                                Some(sample.time_secs);
                        }
                    }
                }
                _ => {}
            }
            if sample.movement_input_forward_axis < 0.0 {
                if self.first_backward_lateral_input_time_secs.is_none() {
                    self.first_backward_lateral_input_time_secs = Some(sample.time_secs);
                }
                self.max_backward_lateral_response_mps = self
                    .max_backward_lateral_response_mps
                    .max(sample.lateral_response_mps);
                if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                    && self.first_backward_lateral_response_time_secs.is_none()
                {
                    self.first_backward_lateral_response_time_secs = Some(sample.time_secs);
                }
            }
        }
        if sample.movement_input_forward_axis < 0.0 && sample.mode != FlightMode::Grounded.label() {
            let planar_speed = Vec2::new(sample.velocity[0], sample.velocity[2]).length();
            if self.backward_air_control_start_speed_mps.is_none() {
                self.backward_air_control_start_speed_mps = Some(sample.speed_mps);
            }
            if self.backward_air_control_start_planar_speed_mps.is_none() {
                self.backward_air_control_start_planar_speed_mps = Some(planar_speed);
            }
            let min_speed = self
                .min_backward_air_control_speed_mps
                .map_or(sample.speed_mps, |speed| speed.min(sample.speed_mps));
            self.min_backward_air_control_speed_mps = Some(min_speed);
            let min_planar_speed = self
                .min_backward_air_control_planar_speed_mps
                .map_or(planar_speed, |speed| speed.min(planar_speed));
            self.min_backward_air_control_planar_speed_mps = Some(min_planar_speed);
            if let Some(start_speed) = self.backward_air_control_start_speed_mps {
                self.max_air_brake_speed_drop_mps = self
                    .max_air_brake_speed_drop_mps
                    .max(start_speed - min_speed);
            }
            if let Some(start_planar_speed) = self.backward_air_control_start_planar_speed_mps {
                self.max_air_brake_planar_speed_drop_mps = self
                    .max_air_brake_planar_speed_drop_mps
                    .max(start_planar_speed - min_planar_speed);
            }
        } else if self.backward_air_control_start_speed_mps.is_some()
            && sample.movement_input_forward_axis > 0.0
            && sample.desired_heading_alignment_mps.is_finite()
        {
            self.max_post_brake_forward_alignment_mps = self
                .max_post_brake_forward_alignment_mps
                .max(sample.desired_heading_alignment_mps);
        }
        self.min_target_distance_m = self.min_target_distance_m.min(sample.target_distance_m);
        self.min_camera_pitch_degrees = self
            .min_camera_pitch_degrees
            .min(sample.camera_pitch_degrees);
        self.max_camera_pitch_degrees = self
            .max_camera_pitch_degrees
            .max(sample.camera_pitch_degrees);
        self.max_abs_camera_yaw_offset_degrees = self
            .max_abs_camera_yaw_offset_degrees
            .max(sample.camera_yaw_offset_degrees.abs());
        self.min_camera_pitch_offset_degrees = self
            .min_camera_pitch_offset_degrees
            .min(sample.camera_pitch_offset_degrees);
        self.max_camera_pitch_offset_degrees = self
            .max_camera_pitch_offset_degrees
            .max(sample.camera_pitch_offset_degrees);
        self.max_visible_wind_fields = self.max_visible_wind_fields.max(sample.visible_wind_fields);
        self.max_active_lift_fields = self.max_active_lift_fields.max(sample.active_lift_fields);
        self.max_readable_lift_fields = self
            .max_readable_lift_fields
            .max(sample.readable_lift_fields);
        self.max_sky_island_count = self.max_sky_island_count.max(sample.sky_island_count);
        self.max_active_chunk_count = self.max_active_chunk_count.max(sample.active_chunk_count);
        self.max_active_island_count = self.max_active_island_count.max(sample.active_island_count);
        self.max_near_lod_islands = self.max_near_lod_islands.max(sample.near_lod_islands);
        self.max_mid_lod_islands = self.max_mid_lod_islands.max(sample.mid_lod_islands);
        self.max_far_lod_islands = self.max_far_lod_islands.max(sample.far_lod_islands);
        self.max_visible_island_terrain_count = self
            .max_visible_island_terrain_count
            .max(sample.visible_island_terrain_count);
        self.max_hidden_island_terrain_count = self
            .max_hidden_island_terrain_count
            .max(sample.hidden_island_terrain_count);
        self.max_visible_island_impostor_count = self
            .max_visible_island_impostor_count
            .max(sample.visible_island_impostor_count);
        self.max_hidden_island_impostor_count = self
            .max_hidden_island_impostor_count
            .max(sample.hidden_island_impostor_count);
        self.max_visible_island_detail_count = self
            .max_visible_island_detail_count
            .max(sample.visible_island_detail_count);
        self.max_hidden_island_detail_count = self
            .max_hidden_island_detail_count
            .max(sample.hidden_island_detail_count);
        self.max_visible_route_beacon_count = self
            .max_visible_route_beacon_count
            .max(sample.visible_route_beacon_count);
        self.max_weather_cloud_count = self.max_weather_cloud_count.max(sample.weather_cloud_count);
        self.max_environment_motion_visual_count = self
            .max_environment_motion_visual_count
            .max(sample.environment_motion_visual_count);
        self.max_environment_motion_offset_m = self
            .max_environment_motion_offset_m
            .max(sample.max_environment_motion_offset_m);
        self.min_island_terrain_surface_count = self
            .min_island_terrain_surface_count
            .min(sample.island_terrain_surface_count);
        self.min_island_terrain_mesh_vertices = self
            .min_island_terrain_mesh_vertices
            .min(sample.min_island_terrain_mesh_vertices);
        self.min_island_terrain_color_bands = self
            .min_island_terrain_color_bands
            .min(sample.min_island_terrain_color_bands);
        self.min_island_terrain_material_weight_bands = self
            .min_island_terrain_material_weight_bands
            .min(sample.min_island_terrain_material_weight_bands);
        self.min_island_terrain_material_channels = self
            .min_island_terrain_material_channels
            .min(sample.min_island_terrain_material_channels);
        self.min_island_terrain_material_regions = self
            .min_island_terrain_material_regions
            .min(sample.min_island_terrain_material_regions);
        self.min_island_terrain_texture_detail_bands = self
            .min_island_terrain_texture_detail_bands
            .min(sample.min_island_terrain_texture_detail_bands);
        self.min_island_terrain_relief_range_m = self
            .min_island_terrain_relief_range_m
            .min(sample.min_island_terrain_relief_range_m);
        self.min_island_cliff_color_bands = self
            .min_island_cliff_color_bands
            .min(sample.min_island_cliff_color_bands);
        self.min_island_impostor_mesh_vertices = self
            .min_island_impostor_mesh_vertices
            .min(sample.min_island_impostor_mesh_vertices);
        self.min_island_impostor_color_bands = self
            .min_island_impostor_color_bands
            .min(sample.min_island_impostor_color_bands);
        self.min_procedural_island_body_count = self
            .min_procedural_island_body_count
            .min(sample.procedural_island_body_count);
        self.max_primitive_island_body_count = self
            .max_primitive_island_body_count
            .max(sample.primitive_island_body_count);
        self.min_island_body_silhouette_segments = self
            .min_island_body_silhouette_segments
            .min(sample.min_island_body_silhouette_segments);
        self.max_avg_island_body_silhouette_segments = self
            .max_avg_island_body_silhouette_segments
            .max(sample.avg_island_body_silhouette_segments);
        self.min_island_body_mesh_vertices = self
            .min_island_body_mesh_vertices
            .min(sample.min_island_body_mesh_vertices);
        self.max_island_body_mesh_vertices = self
            .max_island_body_mesh_vertices
            .max(sample.max_island_body_mesh_vertices);
        self.min_generated_ground_cover_patch_count = self
            .min_generated_ground_cover_patch_count
            .min(sample.generated_ground_cover_patch_count);
        self.min_ground_cover_blade_count = self
            .min_ground_cover_blade_count
            .min(sample.min_ground_cover_blade_count);
        self.min_ground_cover_mesh_vertices = self
            .min_ground_cover_mesh_vertices
            .min(sample.min_ground_cover_mesh_vertices);
        self.min_generated_tree_trunk_count = self
            .min_generated_tree_trunk_count
            .min(sample.generated_tree_trunk_count);
        self.min_generated_tree_canopy_count = self
            .min_generated_tree_canopy_count
            .min(sample.generated_tree_canopy_count);
        self.min_tree_trunk_mesh_vertices = self
            .min_tree_trunk_mesh_vertices
            .min(sample.min_tree_trunk_mesh_vertices);
        self.min_tree_canopy_mesh_vertices = self
            .min_tree_canopy_mesh_vertices
            .min(sample.min_tree_canopy_mesh_vertices);
        self.min_detail_biome_palette_count = self
            .min_detail_biome_palette_count
            .min(sample.detail_biome_palette_count);
        self.min_generated_rock_count = self
            .min_generated_rock_count
            .min(sample.generated_rock_count);
        self.min_rock_mesh_vertices = self
            .min_rock_mesh_vertices
            .min(sample.min_rock_mesh_vertices);
        self.min_generated_weather_cloud_count = self
            .min_generated_weather_cloud_count
            .min(sample.generated_weather_cloud_count);
        self.min_generated_weather_cloud_bank_count = self
            .min_generated_weather_cloud_bank_count
            .min(sample.generated_weather_cloud_bank_count);
        self.min_weather_cloud_bank_depth_m = self
            .min_weather_cloud_bank_depth_m
            .min(sample.min_weather_cloud_bank_depth_m);
        self.min_weather_cloud_lobe_count = self
            .min_weather_cloud_lobe_count
            .min(sample.min_weather_cloud_lobe_count);
        self.min_max_weather_cloud_lobe_count = self
            .min_max_weather_cloud_lobe_count
            .min(sample.max_weather_cloud_lobe_count);
        self.min_weather_cloud_mesh_vertices = self
            .min_weather_cloud_mesh_vertices
            .min(sample.min_weather_cloud_mesh_vertices);
        self.max_resident_island_visual_count = self
            .max_resident_island_visual_count
            .max(sample.resident_island_visual_count);
        self.max_stream_visibility_changes_per_frame = self
            .max_stream_visibility_changes_per_frame
            .max(sample.max_stream_visibility_changes_per_frame);
        self.total_stream_visibility_changes = self
            .total_stream_visibility_changes
            .max(sample.total_stream_visibility_changes);
        self.max_catalog_island_visual_count = self
            .max_catalog_island_visual_count
            .max(sample.catalog_island_visual_count);
        self.max_hidden_island_visual_count = self
            .max_hidden_island_visual_count
            .max(sample.hidden_island_visual_count);
        self.max_resident_island_visual_fraction = self
            .max_resident_island_visual_fraction
            .max(sample.resident_island_visual_fraction);
        self.max_stream_spawned_visuals_per_frame = self
            .max_stream_spawned_visuals_per_frame
            .max(sample.max_stream_spawned_visuals_per_frame);
        self.max_stream_despawned_visuals_per_frame = self
            .max_stream_despawned_visuals_per_frame
            .max(sample.max_stream_despawned_visuals_per_frame);
        self.total_stream_spawned_visuals = self
            .total_stream_spawned_visuals
            .max(sample.total_stream_spawned_visuals);
        self.total_stream_despawned_visuals = self
            .total_stream_despawned_visuals
            .max(sample.total_stream_despawned_visuals);
        self.max_entity_count = self.max_entity_count.max(sample.entity_count);
        self.max_objective_total_count = self
            .max_objective_total_count
            .max(sample.objective.total_count);
        self.max_completed_objective_count = self
            .max_completed_objective_count
            .max(sample.objective.completed_count);
        self.min_objective_distance_m = self
            .min_objective_distance_m
            .min(sample.objective.current_distance_m);
        if sample.objective.complete {
            self.objective_complete_samples += 1;
        }
        self.max_visual_asset_slot_count = self
            .max_visual_asset_slot_count
            .max(sample.visual_asset_slot_count);
        self.max_gltf_scene_asset_slot_count = self
            .max_gltf_scene_asset_slot_count
            .max(sample.gltf_scene_asset_slot_count);
        self.max_ready_visual_asset_slot_count = self
            .max_ready_visual_asset_slot_count
            .max(sample.ready_visual_asset_slot_count);
        self.max_placeholder_visual_asset_slot_count = self
            .max_placeholder_visual_asset_slot_count
            .max(sample.placeholder_visual_asset_slot_count);
        self.max_streaming_visual_asset_slot_count = self
            .max_streaming_visual_asset_slot_count
            .max(sample.streaming_visual_asset_slot_count);
        self.max_missing_visual_asset_slot_count = self
            .max_missing_visual_asset_slot_count
            .max(sample.missing_visual_asset_slot_count);
        self.max_deferred_visual_asset_scene_count = self
            .max_deferred_visual_asset_scene_count
            .max(sample.deferred_visual_asset_scene_count);
        self.max_queued_visual_asset_scene_count = self
            .max_queued_visual_asset_scene_count
            .max(sample.queued_visual_asset_scene_count);
        self.max_loading_visual_asset_scene_count = self
            .max_loading_visual_asset_scene_count
            .max(sample.loading_visual_asset_scene_count);
        self.max_loaded_visual_asset_scene_count = self
            .max_loaded_visual_asset_scene_count
            .max(sample.loaded_visual_asset_scene_count);
        self.max_dependency_loaded_visual_asset_scene_count = self
            .max_dependency_loaded_visual_asset_scene_count
            .max(sample.dependency_loaded_visual_asset_scene_count);
        self.max_preload_ready_visual_asset_scene_count = self
            .max_preload_ready_visual_asset_scene_count
            .max(sample.preload_ready_visual_asset_scene_count);
        self.max_failed_visual_asset_scene_count = self
            .max_failed_visual_asset_scene_count
            .max(sample.failed_visual_asset_scene_count);
        self.max_spawned_visual_asset_scene_count = self
            .max_spawned_visual_asset_scene_count
            .max(sample.spawned_visual_asset_scene_count);
        self.max_ready_visual_asset_scene_count = self
            .max_ready_visual_asset_scene_count
            .max(sample.ready_visual_asset_scene_count);
        self.max_visible_authored_world_fixture_count = self
            .max_visible_authored_world_fixture_count
            .max(sample.visible_authored_world_fixture_count);
        self.max_always_visual_asset_slot_count = self
            .max_always_visual_asset_slot_count
            .max(sample.always_visual_asset_slot_count);
        self.max_stream_window_visual_asset_slot_count = self
            .max_stream_window_visual_asset_slot_count
            .max(sample.stream_window_visual_asset_slot_count);
        self.max_near_lod_visual_asset_slot_count = self
            .max_near_lod_visual_asset_slot_count
            .max(sample.near_lod_visual_asset_slot_count);
        self.max_far_lod_visual_asset_slot_count = self
            .max_far_lod_visual_asset_slot_count
            .max(sample.far_lod_visual_asset_slot_count);
        self.max_weather_visual_asset_slot_count = self
            .max_weather_visual_asset_slot_count
            .max(sample.weather_visual_asset_slot_count);
        self.max_always_preload_ready_visual_asset_slot_count = self
            .max_always_preload_ready_visual_asset_slot_count
            .max(sample.always_preload_ready_visual_asset_slot_count);
        self.max_streaming_preload_ready_visual_asset_slot_count = self
            .max_streaming_preload_ready_visual_asset_slot_count
            .max(sample.streaming_preload_ready_visual_asset_slot_count);
        self.max_declared_animation_clip_count = self
            .max_declared_animation_clip_count
            .max(sample.declared_animation_clip_count);
        self.max_ready_animation_clip_count = self
            .max_ready_animation_clip_count
            .max(sample.ready_animation_clip_count);
        self.max_animation_player_count = self
            .max_animation_player_count
            .max(sample.animation_player_count);
        self.max_animation_graph_count = self
            .max_animation_graph_count
            .max(sample.animation_graph_count);
        self.max_power_up_count = self.max_power_up_count.max(sample.power_up_count);
        self.min_visible_power_up_count = self
            .min_visible_power_up_count
            .min(sample.visible_power_up_count);
        self.max_collected_power_up_count = self
            .max_collected_power_up_count
            .max(sample.collected_power_up_count);
        self.total_power_up_activations = self
            .total_power_up_activations
            .max(sample.total_power_up_activations);
        if sample.active_power_up_effects > 0 {
            self.power_up_effect_samples += 1;
        }
        if sample.on_landing_target {
            self.target_landing_samples += 1;
        }
        if sample.active_lift_fields > 0 {
            self.lifted_samples += 1;
            if sample.readable_lift_fields > 0 {
                self.readable_lift_samples += 1;
            } else {
                self.unreadable_lift_samples += 1;
            }
        }

        match sample.mode {
            "gliding" => self.gliding_samples += 1,
            "launching" => self.launching_samples += 1,
            "grounded" => self.grounded_samples += 1,
            _ => {}
        }

        self.final_sample = Some(sample);
    }

    pub fn summary(&self, scenario: EvalScenario, artifacts: EvalArtifacts) -> EvalSummary {
        let horizontal_distance_m = match (&self.first_sample, &self.final_sample) {
            (Some(first), Some(final_sample)) => {
                horizontal_distance(first.position, final_sample.position)
            }
            _ => 0.0,
        };
        let thresholds = scenario.thresholds;
        let final_target_distance_m = self
            .final_sample
            .as_ref()
            .map_or(0.0, |sample| sample.target_distance_m);
        let final_objective_completed_count = self
            .final_sample
            .as_ref()
            .map_or(0, |sample| sample.objective.completed_count);
        let final_objective_distance_m = self
            .final_sample
            .as_ref()
            .map_or(0.0, |sample| sample.objective.current_distance_m);
        let frame_time_stats = EvalFrameTimeStats::from_samples(&self.frame_times_ms);
        let avg_desired_body_heading_error_degrees = if self.desired_body_heading_samples == 0 {
            0.0
        } else {
            self.desired_body_heading_error_sum_degrees / self.desired_body_heading_samples as f32
        };
        let mut desired_body_heading_error_values_degrees =
            self.desired_body_heading_error_values_degrees.clone();
        desired_body_heading_error_values_degrees.sort_by(f32::total_cmp);
        let p95_desired_body_heading_error_degrees =
            percentile(&desired_body_heading_error_values_degrees, 0.95);
        let avg_camera_follow_direction_error_degrees =
            if self.camera_follow_direction_error_samples == 0 {
                0.0
            } else {
                self.camera_follow_direction_error_sum_degrees
                    / self.camera_follow_direction_error_samples as f32
            };
        let mut camera_follow_direction_error_values_degrees =
            self.camera_follow_direction_error_values_degrees.clone();
        camera_follow_direction_error_values_degrees.sort_by(f32::total_cmp);
        let p95_camera_follow_direction_error_degrees =
            percentile(&camera_follow_direction_error_values_degrees, 0.95);
        let lateral_response_latency_secs = response_latency_secs(
            self.first_lateral_input_time_secs,
            self.first_lateral_response_time_secs,
            scenario,
        );
        let right_lateral_response_latency_secs = response_latency_secs(
            self.first_right_lateral_input_time_secs,
            self.first_right_lateral_response_time_secs,
            scenario,
        );
        let left_lateral_response_latency_secs = response_latency_secs(
            self.first_left_lateral_input_time_secs,
            self.first_left_lateral_response_time_secs,
            scenario,
        );
        let backward_lateral_response_latency_secs = response_latency_secs(
            self.first_backward_lateral_input_time_secs,
            self.first_backward_lateral_response_time_secs,
            scenario,
        );
        let backward_right_lateral_response_latency_secs = response_latency_secs(
            self.first_backward_right_lateral_input_time_secs,
            self.first_backward_right_lateral_response_time_secs,
            scenario,
        );
        let backward_left_lateral_response_latency_secs = response_latency_secs(
            self.first_backward_left_lateral_input_time_secs,
            self.first_backward_left_lateral_response_time_secs,
            scenario,
        );
        let mut checks = vec![
            EvalCheck::at_least(
                "sample_count",
                self.sample_count as f32,
                thresholds.min_samples as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "horizontal_distance",
                horizontal_distance_m,
                thresholds.min_horizontal_distance_m,
                "m",
            ),
            EvalCheck::at_least(
                "max_altitude",
                self.max_altitude_m,
                thresholds.min_max_altitude_m,
                "m",
            ),
            EvalCheck::at_least(
                "max_speed",
                self.max_speed_mps,
                thresholds.min_max_speed_mps,
                "m/s",
            ),
            EvalCheck::at_least(
                "gliding_samples",
                self.gliding_samples as f32,
                thresholds.min_gliding_samples as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "grounded_samples",
                self.grounded_samples as f32,
                thresholds.min_grounded_samples as f32,
                "samples",
            ),
            EvalCheck::at_most(
                "grounded_visual_foot_gap",
                self.max_grounded_visual_foot_gap_m,
                MAX_GROUNDED_VISUAL_FOOT_GAP_M,
                "m",
            ),
            EvalCheck::at_least(
                "lifted_samples",
                self.lifted_samples as f32,
                thresholds.min_lifted_samples as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "sky_island_count",
                self.max_sky_island_count as f32,
                thresholds.min_sky_island_count as f32,
                "islands",
            ),
            EvalCheck::at_least(
                "active_island_count",
                self.max_active_island_count as f32,
                thresholds.min_active_island_count as f32,
                "islands",
            ),
            EvalCheck::at_most(
                "active_chunk_count",
                self.max_active_chunk_count as f32,
                thresholds.max_active_chunk_count as f32,
                "chunks",
            ),
            EvalCheck::at_least(
                "near_lod_island_count",
                self.max_near_lod_islands as f32,
                thresholds.min_near_lod_island_count as f32,
                "islands",
            ),
            EvalCheck::at_least(
                "mid_lod_island_count",
                self.max_mid_lod_islands as f32,
                thresholds.min_mid_lod_island_count as f32,
                "islands",
            ),
            EvalCheck::at_least(
                "far_lod_island_count",
                self.max_far_lod_islands as f32,
                thresholds.min_far_lod_island_count as f32,
                "islands",
            ),
            EvalCheck::at_most(
                "visible_island_terrain_count",
                self.max_visible_island_terrain_count as f32,
                thresholds.max_visible_island_terrain_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "hidden_island_terrain_count",
                self.max_hidden_island_terrain_count as f32,
                thresholds.min_hidden_island_terrain_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "visible_island_impostor_count",
                self.max_visible_island_impostor_count as f32,
                thresholds.min_visible_island_impostor_count as f32,
                "entities",
            ),
            EvalCheck::at_most(
                "visible_island_detail_count",
                self.max_visible_island_detail_count as f32,
                thresholds.max_visible_island_detail_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "hidden_island_detail_count",
                self.max_hidden_island_detail_count as f32,
                thresholds.min_hidden_island_detail_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "visible_route_beacon_count",
                self.max_visible_route_beacon_count as f32,
                thresholds.min_visible_route_beacon_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "weather_cloud_count",
                self.max_weather_cloud_count as f32,
                thresholds.min_weather_cloud_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "environment_motion_visual_count",
                self.max_environment_motion_visual_count as f32,
                thresholds.min_environment_motion_visual_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "environment_motion_offset",
                self.max_environment_motion_offset_m,
                thresholds.min_environment_motion_offset_m,
                "m",
            ),
            EvalCheck::at_least(
                "island_terrain_surface_count",
                self.min_island_terrain_surface_count as f32,
                thresholds.min_island_terrain_surface_count as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "island_terrain_mesh_vertices",
                self.min_island_terrain_mesh_vertices as f32,
                thresholds.min_island_terrain_mesh_vertices as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "island_terrain_color_bands",
                self.min_island_terrain_color_bands as f32,
                thresholds.min_island_terrain_color_bands as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "island_terrain_material_weight_bands",
                self.min_island_terrain_material_weight_bands as f32,
                MIN_ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "island_terrain_material_channels",
                self.min_island_terrain_material_channels as f32,
                MIN_ISLAND_TERRAIN_MATERIAL_CHANNELS as f32,
                "channels",
            ),
            EvalCheck::at_least(
                "island_terrain_material_regions",
                self.min_island_terrain_material_regions as f32,
                MIN_ISLAND_TERRAIN_MATERIAL_REGIONS as f32,
                "regions",
            ),
            EvalCheck::at_least(
                "island_terrain_texture_detail_bands",
                self.min_island_terrain_texture_detail_bands as f32,
                MIN_ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "island_terrain_relief_range",
                self.min_island_terrain_relief_range_m,
                thresholds.min_island_terrain_relief_range_m,
                "m",
            ),
            EvalCheck::at_least(
                "island_cliff_color_bands",
                self.min_island_cliff_color_bands as f32,
                thresholds.min_island_cliff_color_bands as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "island_impostor_mesh_vertices",
                self.min_island_impostor_mesh_vertices as f32,
                MIN_ISLAND_IMPOSTOR_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "island_impostor_color_bands",
                self.min_island_impostor_color_bands as f32,
                MIN_ISLAND_IMPOSTOR_COLOR_BANDS as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "procedural_island_body_count",
                self.min_procedural_island_body_count as f32,
                thresholds.min_procedural_island_body_count as f32,
                "islands",
            ),
            EvalCheck::at_most(
                "primitive_island_body_count",
                self.max_primitive_island_body_count as f32,
                thresholds.max_primitive_island_body_count as f32,
                "islands",
            ),
            EvalCheck::at_least(
                "island_body_silhouette_segments",
                self.min_island_body_silhouette_segments as f32,
                thresholds.min_island_body_silhouette_segments as f32,
                "segments",
            ),
            EvalCheck::at_least(
                "island_body_mesh_vertices",
                self.min_island_body_mesh_vertices as f32,
                MIN_ISLAND_BODY_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "generated_ground_cover_patch_count",
                self.min_generated_ground_cover_patch_count as f32,
                MIN_GENERATED_GROUND_COVER_PATCH_COUNT as f32,
                "patches",
            ),
            EvalCheck::at_least(
                "ground_cover_blade_count",
                self.min_ground_cover_blade_count as f32,
                MIN_GROUND_COVER_BLADE_COUNT as f32,
                "blades",
            ),
            EvalCheck::at_least(
                "ground_cover_mesh_vertices",
                self.min_ground_cover_mesh_vertices as f32,
                MIN_GROUND_COVER_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "generated_tree_trunk_count",
                self.min_generated_tree_trunk_count as f32,
                MIN_GENERATED_TREE_TRUNK_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "generated_tree_canopy_count",
                self.min_generated_tree_canopy_count as f32,
                MIN_GENERATED_TREE_CANOPY_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "tree_trunk_mesh_vertices",
                self.min_tree_trunk_mesh_vertices as f32,
                MIN_TREE_TRUNK_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "tree_canopy_mesh_vertices",
                self.min_tree_canopy_mesh_vertices as f32,
                MIN_TREE_CANOPY_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "detail_biome_palette_count",
                self.min_detail_biome_palette_count as f32,
                MIN_DETAIL_BIOME_PALETTE_COUNT as f32,
                "palettes",
            ),
            EvalCheck::at_least(
                "generated_rock_count",
                self.min_generated_rock_count as f32,
                MIN_GENERATED_ROCK_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "rock_mesh_vertices",
                self.min_rock_mesh_vertices as f32,
                MIN_ROCK_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "generated_weather_cloud_count",
                self.min_generated_weather_cloud_count as f32,
                MIN_GENERATED_WEATHER_CLOUD_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "generated_weather_cloud_bank_count",
                self.min_generated_weather_cloud_bank_count as f32,
                MIN_GENERATED_WEATHER_CLOUD_BANK_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "weather_cloud_bank_depth",
                self.min_weather_cloud_bank_depth_m,
                MIN_WEATHER_CLOUD_BANK_DEPTH_M,
                "m",
            ),
            EvalCheck::at_least(
                "weather_cloud_lobe_count",
                self.min_weather_cloud_lobe_count as f32,
                MIN_WEATHER_CLOUD_LOBE_COUNT as f32,
                "lobes",
            ),
            EvalCheck::at_least(
                "weather_cloud_bank_lobe_count",
                self.min_max_weather_cloud_lobe_count as f32,
                MIN_MAX_WEATHER_CLOUD_LOBE_COUNT as f32,
                "lobes",
            ),
            EvalCheck::at_least(
                "weather_cloud_mesh_vertices",
                self.min_weather_cloud_mesh_vertices as f32,
                MIN_WEATHER_CLOUD_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_most(
                "resident_island_visual_count",
                self.max_resident_island_visual_count as f32,
                thresholds.max_resident_island_visual_count as f32,
                "entities",
            ),
            EvalCheck::at_most(
                "stream_visibility_changes_per_frame",
                self.max_stream_visibility_changes_per_frame as f32,
                thresholds.max_stream_visibility_changes_per_frame as f32,
                "entities/frame",
            ),
            EvalCheck::at_least(
                "hidden_island_visual_count",
                self.max_hidden_island_visual_count as f32,
                (thresholds.min_hidden_island_terrain_count
                    + thresholds.min_hidden_island_detail_count) as f32,
                "entities",
            ),
            EvalCheck::at_most(
                "resident_island_visual_fraction",
                self.max_resident_island_visual_fraction,
                MAX_RESIDENT_ISLAND_VISUAL_FRACTION,
                "ratio",
            ),
            EvalCheck::at_most(
                "stream_spawned_visuals_per_frame",
                self.max_stream_spawned_visuals_per_frame as f32,
                thresholds.max_stream_visibility_changes_per_frame as f32,
                "entities/frame",
            ),
            EvalCheck::at_most(
                "stream_despawned_visuals_per_frame",
                self.max_stream_despawned_visuals_per_frame as f32,
                thresholds.max_stream_visibility_changes_per_frame as f32,
                "entities/frame",
            ),
            EvalCheck::at_least(
                "entity_count",
                self.max_entity_count as f32,
                thresholds.min_entity_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "objective_total_count",
                self.max_objective_total_count as f32,
                thresholds.min_objective_total_count as f32,
                "objectives",
            ),
            EvalCheck::at_least(
                "completed_objective_count",
                self.max_completed_objective_count as f32,
                thresholds.min_completed_objective_count as f32,
                "objectives",
            ),
            EvalCheck::at_least(
                "visual_asset_slot_count",
                self.max_visual_asset_slot_count as f32,
                thresholds.min_visual_asset_slot_count as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "gltf_scene_asset_slot_count",
                self.max_gltf_scene_asset_slot_count as f32,
                thresholds.min_gltf_scene_asset_slot_count as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "ready_visual_asset_slot_count",
                self.max_ready_visual_asset_slot_count as f32,
                MIN_READY_VISUAL_ASSET_SLOT_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_most(
                "missing_visual_asset_slot_count",
                self.max_missing_visual_asset_slot_count as f32,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_most(
                "deferred_visual_asset_scene_count",
                self.max_deferred_visual_asset_scene_count as f32,
                MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "streaming_visual_asset_slot_count",
                self.max_streaming_visual_asset_slot_count as f32,
                thresholds.min_streaming_visual_asset_slot_count as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "loaded_visual_asset_scene_count",
                self.max_loaded_visual_asset_scene_count as f32,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "dependency_loaded_visual_asset_scene_count",
                self.max_dependency_loaded_visual_asset_scene_count as f32,
                MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "preload_ready_visual_asset_scene_count",
                self.max_preload_ready_visual_asset_scene_count as f32,
                MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "always_preload_ready_visual_asset_slot_count",
                self.max_always_preload_ready_visual_asset_slot_count as f32,
                MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "streaming_preload_ready_visual_asset_slot_count",
                self.max_streaming_preload_ready_visual_asset_slot_count as f32,
                MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "spawned_visual_asset_scene_count",
                self.max_spawned_visual_asset_scene_count as f32,
                MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "ready_visual_asset_scene_count",
                self.max_ready_visual_asset_scene_count as f32,
                MIN_READY_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "visible_authored_world_fixture_count",
                self.max_visible_authored_world_fixture_count as f32,
                MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "declared_animation_clip_count",
                self.max_declared_animation_clip_count as f32,
                thresholds.min_declared_animation_clip_count as f32,
                "clips",
            ),
            EvalCheck::at_least(
                "ready_animation_clip_count",
                self.max_ready_animation_clip_count as f32,
                MIN_READY_VISUAL_ANIMATION_CLIP_COUNT as f32,
                "clips",
            ),
            EvalCheck::at_least(
                "animation_player_count",
                self.max_animation_player_count as f32,
                MIN_VISUAL_ANIMATION_PLAYER_COUNT as f32,
                "players",
            ),
            EvalCheck::at_least(
                "animation_graph_count",
                self.max_animation_graph_count as f32,
                MIN_VISUAL_ANIMATION_GRAPH_COUNT as f32,
                "graphs",
            ),
            EvalCheck::at_most(
                "failed_visual_asset_scene_count",
                self.max_failed_visual_asset_scene_count as f32,
                thresholds.max_failed_visual_asset_scene_count as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "power_up_count",
                self.max_power_up_count as f32,
                thresholds.min_power_up_count as f32,
                "power-ups",
            ),
            EvalCheck::at_least(
                "collected_power_up_count",
                self.max_collected_power_up_count as f32,
                thresholds.min_collected_power_up_count as f32,
                "power-ups",
            ),
            EvalCheck::at_least(
                "power_up_effect_samples",
                self.power_up_effect_samples as f32,
                thresholds.min_power_up_effect_samples as f32,
                "samples",
            ),
            EvalCheck::at_most(
                "max_camera_distance",
                self.max_camera_distance_m,
                thresholds.max_camera_distance_m,
                "m",
            ),
            EvalCheck::at_least(
                "min_camera_surface_clearance",
                self.min_camera_surface_clearance_m,
                thresholds.min_camera_surface_clearance_m,
                "m",
            ),
            EvalCheck::at_most(
                "max_camera_player_angle",
                self.max_camera_player_angle_degrees,
                thresholds.max_camera_player_angle_degrees,
                "deg",
            ),
            EvalCheck::at_most(
                "max_camera_step_distance",
                self.max_camera_step_distance_m,
                thresholds.max_camera_step_distance_m,
                "m",
            ),
            EvalCheck::at_most(
                "max_camera_rotation_delta",
                self.max_camera_rotation_delta_degrees,
                thresholds.max_camera_rotation_delta_degrees,
                "deg",
            ),
            EvalCheck::at_most(
                "max_camera_orbit_alignment",
                self.max_camera_orbit_alignment_degrees,
                thresholds.max_camera_orbit_alignment_degrees,
                "deg",
            ),
            EvalCheck::at_most(
                "max_abs_camera_view_yaw",
                self.max_abs_camera_view_yaw_degrees,
                thresholds.max_abs_camera_view_yaw_degrees,
                "deg",
            ),
            EvalCheck::at_least(
                "max_camera_obstruction_adjustment",
                self.max_camera_obstruction_adjustment_m,
                thresholds.min_camera_obstruction_adjustment_m,
                "m",
            ),
            EvalCheck::at_least(
                "max_abs_camera_yaw_offset",
                self.max_abs_camera_yaw_offset_degrees,
                thresholds.min_abs_camera_yaw_degrees,
                "deg",
            ),
            EvalCheck::at_most(
                "min_camera_pitch_offset",
                self.min_camera_pitch_offset_degrees,
                thresholds.min_camera_pitch_offset_degrees,
                "deg",
            ),
            EvalCheck::at_least(
                "max_camera_pitch_offset",
                self.max_camera_pitch_offset_degrees,
                thresholds.max_camera_pitch_offset_degrees,
                "deg",
            ),
        ];
        if thresholds.min_lifted_samples > 0 {
            checks.push(EvalCheck::at_least(
                "readable_lift_samples",
                self.readable_lift_samples as f32,
                thresholds.min_lifted_samples as f32,
                "samples",
            ));
            checks.push(EvalCheck::at_most(
                "unreadable_lift_samples",
                self.unreadable_lift_samples as f32,
                0.0,
                "samples",
            ));
        }
        if thresholds.require_target_landing {
            checks.push(EvalCheck::at_most(
                "final_target_distance",
                final_target_distance_m,
                thresholds.max_final_target_distance_m,
                "m",
            ));
            checks.push(EvalCheck::at_least(
                "target_landing_samples",
                self.target_landing_samples as f32,
                thresholds.min_target_landing_samples as f32,
                "samples",
            ));
        }
        if scenario.name == AIR_CONTROL_RESPONSE {
            checks.push(EvalCheck::at_most(
                "air_control_lateral_response_latency",
                lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_lateral_response",
                self.max_lateral_response_mps,
                AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_right_lateral_response_latency",
                right_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_right_lateral_response",
                self.max_right_lateral_response_mps,
                AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_left_lateral_response_latency",
                left_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_left_lateral_response",
                self.max_left_lateral_response_mps,
                AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_backward_lateral_response_latency",
                backward_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_lateral_response",
                self.max_backward_lateral_response_mps,
                AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_backward_right_lateral_response_latency",
                backward_right_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_right_lateral_response",
                self.max_backward_right_lateral_response_mps,
                AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_right_rear_response",
                self.max_backward_right_rear_response_mps,
                AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_backward_left_lateral_response_latency",
                backward_left_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_left_lateral_response",
                self.max_backward_left_lateral_response_mps,
                AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_left_rear_response",
                self.max_backward_left_rear_response_mps,
                AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_air_brake_speed_drop",
                self.max_air_brake_speed_drop_mps,
                AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_air_brake_planar_speed_drop",
                self.max_air_brake_planar_speed_drop_mps,
                AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_post_brake_forward_alignment",
                self.max_post_brake_forward_alignment_mps,
                AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_desired_heading_alignment",
                self.max_desired_heading_alignment_mps,
                AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_avg_body_heading_error",
                avg_desired_body_heading_error_degrees,
                AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_p95_body_heading_error",
                p95_desired_body_heading_error_degrees,
                AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_max_body_heading_error",
                self.max_desired_body_heading_error_degrees,
                AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_max_body_yaw_error_step",
                self.max_body_yaw_error_step_degrees,
                AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_body_yaw_oscillation_count",
                self.body_yaw_oscillation_count as f32,
                AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
                "sign changes",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_right_body_bank_response",
                self.max_right_body_bank_degrees,
                AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_left_body_bank_response",
                self.max_left_body_bank_degrees,
                AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_max_body_roll_step",
                self.max_body_roll_step_degrees,
                AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_camera_orbit_yaw_offset",
                self.max_abs_camera_yaw_offset_degrees,
                AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_camera_rotation_delta",
                self.max_camera_rotation_delta_degrees,
                AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_camera_view_yaw_drift",
                self.max_camera_view_yaw_drift_degrees,
                AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_avg_camera_follow_direction_error",
                avg_camera_follow_direction_error_degrees,
                AIR_CONTROL_MAX_AVG_CAMERA_FOLLOW_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_p95_camera_follow_direction_error",
                p95_camera_follow_direction_error_degrees,
                AIR_CONTROL_MAX_P95_CAMERA_FOLLOW_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_camera_world_yaw_drift",
                self.max_camera_world_yaw_drift_degrees,
                MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
                "deg",
            ));
        }
        if scenario.name == CAMERA_STRAFE_STABILITY {
            checks.push(EvalCheck::at_least(
                "camera_strafe_right_lateral_response",
                self.max_right_lateral_response_mps,
                CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "camera_strafe_left_lateral_response",
                self.max_left_lateral_response_mps,
                CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "camera_strafe_view_yaw_drift",
                self.max_camera_view_yaw_drift_degrees,
                CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "camera_strafe_world_yaw_drift",
                self.max_camera_world_yaw_drift_degrees,
                MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
                "deg",
            ));
        }
        let passed = checks.iter().all(|check| check.passed);

        EvalSummary {
            scenario_name: scenario.name,
            target_island_name: scenario.target_island_name,
            passed,
            frame_count: scenario.frame_count,
            duration_secs: scenario.duration_secs(),
            thresholds,
            metrics: EvalMetricsSummary {
                sample_count: self.sample_count,
                avg_frame_time_ms: frame_time_stats.avg_ms,
                p95_frame_time_ms: frame_time_stats.p95_ms,
                p99_frame_time_ms: frame_time_stats.p99_ms,
                max_frame_time_ms: frame_time_stats.max_ms,
                horizontal_distance_m,
                max_altitude_m: self.max_altitude_m,
                min_altitude_m: self.min_altitude_m,
                max_grounded_visual_foot_gap_m: self.max_grounded_visual_foot_gap_m,
                max_speed_mps: self.max_speed_mps,
                max_camera_distance_m: self.max_camera_distance_m,
                min_camera_surface_clearance_m: self.min_camera_surface_clearance_m,
                max_camera_player_angle_degrees: self.max_camera_player_angle_degrees,
                max_camera_step_distance_m: self.max_camera_step_distance_m,
                max_camera_rotation_delta_degrees: self.max_camera_rotation_delta_degrees,
                max_camera_orbit_alignment_degrees: self.max_camera_orbit_alignment_degrees,
                avg_camera_follow_direction_error_degrees,
                p95_camera_follow_direction_error_degrees,
                max_camera_follow_direction_error_degrees: self
                    .max_camera_follow_direction_error_degrees,
                max_abs_camera_view_yaw_degrees: self.max_abs_camera_view_yaw_degrees,
                max_camera_view_yaw_drift_degrees: self.max_camera_view_yaw_drift_degrees,
                max_camera_world_yaw_drift_degrees: self.max_camera_world_yaw_drift_degrees,
                max_camera_obstruction_adjustment_m: self.max_camera_obstruction_adjustment_m,
                max_camera_obstruction_hits: self.max_camera_obstruction_hits,
                avg_desired_body_heading_error_degrees,
                p95_desired_body_heading_error_degrees,
                max_desired_body_heading_error_degrees: self.max_desired_body_heading_error_degrees,
                max_body_yaw_error_step_degrees: self.max_body_yaw_error_step_degrees,
                body_yaw_oscillation_count: self.body_yaw_oscillation_count,
                max_body_roll_step_degrees: self.max_body_roll_step_degrees,
                max_right_body_bank_degrees: self.max_right_body_bank_degrees,
                max_left_body_bank_degrees: self.max_left_body_bank_degrees,
                max_desired_heading_alignment_mps: self.max_desired_heading_alignment_mps,
                max_lateral_response_mps: self.max_lateral_response_mps,
                lateral_response_latency_secs,
                max_right_lateral_response_mps: self.max_right_lateral_response_mps,
                right_lateral_response_latency_secs,
                max_left_lateral_response_mps: self.max_left_lateral_response_mps,
                left_lateral_response_latency_secs,
                max_backward_lateral_response_mps: self.max_backward_lateral_response_mps,
                backward_lateral_response_latency_secs,
                max_backward_right_lateral_response_mps: self
                    .max_backward_right_lateral_response_mps,
                backward_right_lateral_response_latency_secs,
                max_backward_right_rear_response_mps: self.max_backward_right_rear_response_mps,
                max_backward_left_lateral_response_mps: self.max_backward_left_lateral_response_mps,
                backward_left_lateral_response_latency_secs,
                max_backward_left_rear_response_mps: self.max_backward_left_rear_response_mps,
                max_air_brake_speed_drop_mps: self.max_air_brake_speed_drop_mps,
                max_air_brake_planar_speed_drop_mps: self.max_air_brake_planar_speed_drop_mps,
                max_post_brake_forward_alignment_mps: self.max_post_brake_forward_alignment_mps,
                min_target_distance_m: self.min_target_distance_m,
                final_target_distance_m,
                min_camera_pitch_degrees: self.min_camera_pitch_degrees,
                max_camera_pitch_degrees: self.max_camera_pitch_degrees,
                max_abs_camera_yaw_offset_degrees: self.max_abs_camera_yaw_offset_degrees,
                min_camera_pitch_offset_degrees: self.min_camera_pitch_offset_degrees,
                max_camera_pitch_offset_degrees: self.max_camera_pitch_offset_degrees,
                max_visible_wind_fields: self.max_visible_wind_fields,
                max_active_lift_fields: self.max_active_lift_fields,
                max_readable_lift_fields: self.max_readable_lift_fields,
                max_sky_island_count: self.max_sky_island_count,
                max_active_chunk_count: self.max_active_chunk_count,
                max_active_island_count: self.max_active_island_count,
                max_near_lod_islands: self.max_near_lod_islands,
                max_mid_lod_islands: self.max_mid_lod_islands,
                max_far_lod_islands: self.max_far_lod_islands,
                max_visible_island_terrain_count: self.max_visible_island_terrain_count,
                max_hidden_island_terrain_count: self.max_hidden_island_terrain_count,
                max_visible_island_impostor_count: self.max_visible_island_impostor_count,
                max_hidden_island_impostor_count: self.max_hidden_island_impostor_count,
                max_visible_island_detail_count: self.max_visible_island_detail_count,
                max_hidden_island_detail_count: self.max_hidden_island_detail_count,
                max_visible_route_beacon_count: self.max_visible_route_beacon_count,
                max_weather_cloud_count: self.max_weather_cloud_count,
                max_environment_motion_visual_count: self.max_environment_motion_visual_count,
                max_environment_motion_offset_m: self.max_environment_motion_offset_m,
                min_island_terrain_surface_count: self.min_island_terrain_surface_count,
                min_island_terrain_mesh_vertices: self.min_island_terrain_mesh_vertices,
                min_island_terrain_color_bands: self.min_island_terrain_color_bands,
                min_island_terrain_material_weight_bands: self
                    .min_island_terrain_material_weight_bands,
                min_island_terrain_material_channels: self.min_island_terrain_material_channels,
                min_island_terrain_material_regions: self.min_island_terrain_material_regions,
                min_island_terrain_texture_detail_bands: self
                    .min_island_terrain_texture_detail_bands,
                min_island_terrain_relief_range_m: self.min_island_terrain_relief_range_m,
                min_island_cliff_color_bands: self.min_island_cliff_color_bands,
                min_island_impostor_mesh_vertices: self.min_island_impostor_mesh_vertices,
                min_island_impostor_color_bands: self.min_island_impostor_color_bands,
                min_procedural_island_body_count: self.min_procedural_island_body_count,
                max_primitive_island_body_count: self.max_primitive_island_body_count,
                min_island_body_silhouette_segments: self.min_island_body_silhouette_segments,
                max_avg_island_body_silhouette_segments: self
                    .max_avg_island_body_silhouette_segments,
                min_island_body_mesh_vertices: self.min_island_body_mesh_vertices,
                max_island_body_mesh_vertices: self.max_island_body_mesh_vertices,
                min_generated_ground_cover_patch_count: self.min_generated_ground_cover_patch_count,
                min_ground_cover_blade_count: self.min_ground_cover_blade_count,
                min_ground_cover_mesh_vertices: self.min_ground_cover_mesh_vertices,
                min_generated_tree_trunk_count: self.min_generated_tree_trunk_count,
                min_generated_tree_canopy_count: self.min_generated_tree_canopy_count,
                min_tree_trunk_mesh_vertices: self.min_tree_trunk_mesh_vertices,
                min_tree_canopy_mesh_vertices: self.min_tree_canopy_mesh_vertices,
                min_detail_biome_palette_count: self.min_detail_biome_palette_count,
                min_generated_rock_count: self.min_generated_rock_count,
                min_rock_mesh_vertices: self.min_rock_mesh_vertices,
                min_generated_weather_cloud_count: self.min_generated_weather_cloud_count,
                min_generated_weather_cloud_bank_count: self.min_generated_weather_cloud_bank_count,
                min_weather_cloud_bank_depth_m: self.min_weather_cloud_bank_depth_m,
                min_weather_cloud_lobe_count: self.min_weather_cloud_lobe_count,
                min_max_weather_cloud_lobe_count: self.min_max_weather_cloud_lobe_count,
                min_weather_cloud_mesh_vertices: self.min_weather_cloud_mesh_vertices,
                max_resident_island_visual_count: self.max_resident_island_visual_count,
                max_stream_visibility_changes_per_frame: self
                    .max_stream_visibility_changes_per_frame,
                total_stream_visibility_changes: self.total_stream_visibility_changes,
                max_catalog_island_visual_count: self.max_catalog_island_visual_count,
                max_hidden_island_visual_count: self.max_hidden_island_visual_count,
                max_resident_island_visual_fraction: self.max_resident_island_visual_fraction,
                max_stream_spawned_visuals_per_frame: self.max_stream_spawned_visuals_per_frame,
                max_stream_despawned_visuals_per_frame: self.max_stream_despawned_visuals_per_frame,
                total_stream_spawned_visuals: self.total_stream_spawned_visuals,
                total_stream_despawned_visuals: self.total_stream_despawned_visuals,
                max_entity_count: self.max_entity_count,
                objective_total_count: self.max_objective_total_count,
                max_completed_objective_count: self.max_completed_objective_count,
                final_objective_completed_count,
                min_objective_distance_m: self.min_objective_distance_m,
                final_objective_distance_m,
                objective_complete_samples: self.objective_complete_samples,
                max_visual_asset_slot_count: self.max_visual_asset_slot_count,
                max_gltf_scene_asset_slot_count: self.max_gltf_scene_asset_slot_count,
                max_ready_visual_asset_slot_count: self.max_ready_visual_asset_slot_count,
                max_placeholder_visual_asset_slot_count: self
                    .max_placeholder_visual_asset_slot_count,
                max_streaming_visual_asset_slot_count: self.max_streaming_visual_asset_slot_count,
                max_missing_visual_asset_slot_count: self.max_missing_visual_asset_slot_count,
                max_deferred_visual_asset_scene_count: self.max_deferred_visual_asset_scene_count,
                max_queued_visual_asset_scene_count: self.max_queued_visual_asset_scene_count,
                max_loading_visual_asset_scene_count: self.max_loading_visual_asset_scene_count,
                max_loaded_visual_asset_scene_count: self.max_loaded_visual_asset_scene_count,
                max_dependency_loaded_visual_asset_scene_count: self
                    .max_dependency_loaded_visual_asset_scene_count,
                max_preload_ready_visual_asset_scene_count: self
                    .max_preload_ready_visual_asset_scene_count,
                max_failed_visual_asset_scene_count: self.max_failed_visual_asset_scene_count,
                max_spawned_visual_asset_scene_count: self.max_spawned_visual_asset_scene_count,
                max_ready_visual_asset_scene_count: self.max_ready_visual_asset_scene_count,
                max_visible_authored_world_fixture_count: self
                    .max_visible_authored_world_fixture_count,
                max_always_visual_asset_slot_count: self.max_always_visual_asset_slot_count,
                max_stream_window_visual_asset_slot_count: self
                    .max_stream_window_visual_asset_slot_count,
                max_near_lod_visual_asset_slot_count: self.max_near_lod_visual_asset_slot_count,
                max_far_lod_visual_asset_slot_count: self.max_far_lod_visual_asset_slot_count,
                max_weather_visual_asset_slot_count: self.max_weather_visual_asset_slot_count,
                max_always_preload_ready_visual_asset_slot_count: self
                    .max_always_preload_ready_visual_asset_slot_count,
                max_streaming_preload_ready_visual_asset_slot_count: self
                    .max_streaming_preload_ready_visual_asset_slot_count,
                max_declared_animation_clip_count: self.max_declared_animation_clip_count,
                max_ready_animation_clip_count: self.max_ready_animation_clip_count,
                max_animation_player_count: self.max_animation_player_count,
                max_animation_graph_count: self.max_animation_graph_count,
                max_power_up_count: self.max_power_up_count,
                min_visible_power_up_count: self.min_visible_power_up_count,
                max_collected_power_up_count: self.max_collected_power_up_count,
                power_up_effect_samples: self.power_up_effect_samples,
                total_power_up_activations: self.total_power_up_activations,
                target_landing_samples: self.target_landing_samples,
                lifted_samples: self.lifted_samples,
                readable_lift_samples: self.readable_lift_samples,
                unreadable_lift_samples: self.unreadable_lift_samples,
                gliding_samples: self.gliding_samples,
                launching_samples: self.launching_samples,
                grounded_samples: self.grounded_samples,
            },
            checks,
            artifacts,
            final_sample: self.final_sample.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EvalArtifacts {
    pub summary_json: String,
    pub samples_ndjson: String,
    pub screenshot_png: Option<String>,
    pub checkpoint_screenshots: Vec<String>,
    pub checkpoint_marker_metadata: Vec<String>,
}

impl EvalArtifacts {
    fn to_json(&self, indent: &str) -> String {
        let screenshot = self
            .screenshot_png
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".to_string());
        let checkpoint_screenshots = json_string_array(&self.checkpoint_screenshots);
        let checkpoint_marker_metadata = json_string_array(&self.checkpoint_marker_metadata);

        format!(
            "{{\n{indent}  \"summary_json\": {},\n{indent}  \"samples_ndjson\": {},\n{indent}  \"screenshot_png\": {},\n{indent}  \"checkpoint_screenshots\": {},\n{indent}  \"checkpoint_marker_metadata\": {}\n{indent}}}",
            json_string(&self.summary_json),
            json_string(&self.samples_ndjson),
            screenshot,
            checkpoint_screenshots,
            checkpoint_marker_metadata,
        )
    }
}

#[derive(Clone, Debug)]
pub struct EvalMetricsSummary {
    pub sample_count: u32,
    pub avg_frame_time_ms: f32,
    pub p95_frame_time_ms: f32,
    pub p99_frame_time_ms: f32,
    pub max_frame_time_ms: f32,
    pub horizontal_distance_m: f32,
    pub max_altitude_m: f32,
    pub min_altitude_m: f32,
    pub max_grounded_visual_foot_gap_m: f32,
    pub max_speed_mps: f32,
    pub max_camera_distance_m: f32,
    pub min_camera_surface_clearance_m: f32,
    pub max_camera_player_angle_degrees: f32,
    pub max_camera_step_distance_m: f32,
    pub max_camera_rotation_delta_degrees: f32,
    pub max_camera_orbit_alignment_degrees: f32,
    pub avg_camera_follow_direction_error_degrees: f32,
    pub p95_camera_follow_direction_error_degrees: f32,
    pub max_camera_follow_direction_error_degrees: f32,
    pub max_abs_camera_view_yaw_degrees: f32,
    pub max_camera_view_yaw_drift_degrees: f32,
    pub max_camera_world_yaw_drift_degrees: f32,
    pub max_camera_obstruction_adjustment_m: f32,
    pub max_camera_obstruction_hits: usize,
    pub avg_desired_body_heading_error_degrees: f32,
    pub p95_desired_body_heading_error_degrees: f32,
    pub max_desired_body_heading_error_degrees: f32,
    pub max_body_yaw_error_step_degrees: f32,
    pub body_yaw_oscillation_count: u32,
    pub max_body_roll_step_degrees: f32,
    pub max_right_body_bank_degrees: f32,
    pub max_left_body_bank_degrees: f32,
    pub max_desired_heading_alignment_mps: f32,
    pub max_lateral_response_mps: f32,
    pub lateral_response_latency_secs: f32,
    pub max_right_lateral_response_mps: f32,
    pub right_lateral_response_latency_secs: f32,
    pub max_left_lateral_response_mps: f32,
    pub left_lateral_response_latency_secs: f32,
    pub max_backward_lateral_response_mps: f32,
    pub backward_lateral_response_latency_secs: f32,
    pub max_backward_right_lateral_response_mps: f32,
    pub backward_right_lateral_response_latency_secs: f32,
    pub max_backward_right_rear_response_mps: f32,
    pub max_backward_left_lateral_response_mps: f32,
    pub backward_left_lateral_response_latency_secs: f32,
    pub max_backward_left_rear_response_mps: f32,
    pub max_air_brake_speed_drop_mps: f32,
    pub max_air_brake_planar_speed_drop_mps: f32,
    pub max_post_brake_forward_alignment_mps: f32,
    pub min_target_distance_m: f32,
    pub final_target_distance_m: f32,
    pub min_camera_pitch_degrees: f32,
    pub max_camera_pitch_degrees: f32,
    pub max_abs_camera_yaw_offset_degrees: f32,
    pub min_camera_pitch_offset_degrees: f32,
    pub max_camera_pitch_offset_degrees: f32,
    pub max_visible_wind_fields: usize,
    pub max_active_lift_fields: usize,
    pub max_readable_lift_fields: usize,
    pub max_sky_island_count: usize,
    pub max_active_chunk_count: usize,
    pub max_active_island_count: usize,
    pub max_near_lod_islands: usize,
    pub max_mid_lod_islands: usize,
    pub max_far_lod_islands: usize,
    pub max_visible_island_terrain_count: usize,
    pub max_hidden_island_terrain_count: usize,
    pub max_visible_island_impostor_count: usize,
    pub max_hidden_island_impostor_count: usize,
    pub max_visible_island_detail_count: usize,
    pub max_hidden_island_detail_count: usize,
    pub max_visible_route_beacon_count: usize,
    pub max_weather_cloud_count: usize,
    pub max_environment_motion_visual_count: usize,
    pub max_environment_motion_offset_m: f32,
    pub min_island_terrain_surface_count: usize,
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
    pub min_procedural_island_body_count: usize,
    pub max_primitive_island_body_count: usize,
    pub min_island_body_silhouette_segments: usize,
    pub max_avg_island_body_silhouette_segments: f32,
    pub min_island_body_mesh_vertices: usize,
    pub max_island_body_mesh_vertices: usize,
    pub min_generated_ground_cover_patch_count: usize,
    pub min_ground_cover_blade_count: usize,
    pub min_ground_cover_mesh_vertices: usize,
    pub min_generated_tree_trunk_count: usize,
    pub min_generated_tree_canopy_count: usize,
    pub min_tree_trunk_mesh_vertices: usize,
    pub min_tree_canopy_mesh_vertices: usize,
    pub min_detail_biome_palette_count: usize,
    pub min_generated_rock_count: usize,
    pub min_rock_mesh_vertices: usize,
    pub min_generated_weather_cloud_count: usize,
    pub min_generated_weather_cloud_bank_count: usize,
    pub min_weather_cloud_bank_depth_m: f32,
    pub min_weather_cloud_lobe_count: usize,
    pub min_max_weather_cloud_lobe_count: usize,
    pub min_weather_cloud_mesh_vertices: usize,
    pub max_resident_island_visual_count: usize,
    pub max_stream_visibility_changes_per_frame: usize,
    pub total_stream_visibility_changes: usize,
    pub max_catalog_island_visual_count: usize,
    pub max_hidden_island_visual_count: usize,
    pub max_resident_island_visual_fraction: f32,
    pub max_stream_spawned_visuals_per_frame: usize,
    pub max_stream_despawned_visuals_per_frame: usize,
    pub total_stream_spawned_visuals: usize,
    pub total_stream_despawned_visuals: usize,
    pub max_entity_count: usize,
    pub objective_total_count: usize,
    pub max_completed_objective_count: usize,
    pub final_objective_completed_count: usize,
    pub min_objective_distance_m: f32,
    pub final_objective_distance_m: f32,
    pub objective_complete_samples: u32,
    pub max_visual_asset_slot_count: usize,
    pub max_gltf_scene_asset_slot_count: usize,
    pub max_ready_visual_asset_slot_count: usize,
    pub max_placeholder_visual_asset_slot_count: usize,
    pub max_streaming_visual_asset_slot_count: usize,
    pub max_missing_visual_asset_slot_count: usize,
    pub max_deferred_visual_asset_scene_count: usize,
    pub max_queued_visual_asset_scene_count: usize,
    pub max_loading_visual_asset_scene_count: usize,
    pub max_loaded_visual_asset_scene_count: usize,
    pub max_dependency_loaded_visual_asset_scene_count: usize,
    pub max_preload_ready_visual_asset_scene_count: usize,
    pub max_failed_visual_asset_scene_count: usize,
    pub max_spawned_visual_asset_scene_count: usize,
    pub max_ready_visual_asset_scene_count: usize,
    pub max_visible_authored_world_fixture_count: usize,
    pub max_always_visual_asset_slot_count: usize,
    pub max_stream_window_visual_asset_slot_count: usize,
    pub max_near_lod_visual_asset_slot_count: usize,
    pub max_far_lod_visual_asset_slot_count: usize,
    pub max_weather_visual_asset_slot_count: usize,
    pub max_always_preload_ready_visual_asset_slot_count: usize,
    pub max_streaming_preload_ready_visual_asset_slot_count: usize,
    pub max_declared_animation_clip_count: usize,
    pub max_ready_animation_clip_count: usize,
    pub max_animation_player_count: usize,
    pub max_animation_graph_count: usize,
    pub max_power_up_count: usize,
    pub min_visible_power_up_count: usize,
    pub max_collected_power_up_count: usize,
    pub power_up_effect_samples: u32,
    pub total_power_up_activations: usize,
    pub target_landing_samples: u32,
    pub lifted_samples: u32,
    pub readable_lift_samples: u32,
    pub unreadable_lift_samples: u32,
    pub gliding_samples: u32,
    pub launching_samples: u32,
    pub grounded_samples: u32,
}

impl EvalMetricsSummary {
    fn to_json(&self, indent: &str) -> String {
        let json = format!(
            "{{\n{indent}  \"sample_count\": {},\n{indent}  \"avg_frame_time_ms\": {},\n{indent}  \"p95_frame_time_ms\": {},\n{indent}  \"p99_frame_time_ms\": {},\n{indent}  \"max_frame_time_ms\": {},\n{indent}  \"horizontal_distance_m\": {},\n{indent}  \"max_altitude_m\": {},\n{indent}  \"min_altitude_m\": {},\n{indent}  \"max_grounded_visual_foot_gap_m\": {},\n{indent}  \"max_speed_mps\": {},\n{indent}  \"max_camera_distance_m\": {},\n{indent}  \"min_camera_surface_clearance_m\": {},\n{indent}  \"max_camera_player_angle_degrees\": {},\n{indent}  \"max_camera_step_distance_m\": {},\n{indent}  \"max_camera_rotation_delta_degrees\": {},\n{indent}  \"max_camera_orbit_alignment_degrees\": {},\n{indent}  \"avg_camera_follow_direction_error_degrees\": {},\n{indent}  \"p95_camera_follow_direction_error_degrees\": {},\n{indent}  \"max_camera_follow_direction_error_degrees\": {},\n{indent}  \"max_abs_camera_view_yaw_degrees\": {},\n{indent}  \"max_camera_view_yaw_drift_degrees\": {},\n{indent}  \"max_camera_world_yaw_drift_degrees\": {},\n{indent}  \"max_camera_obstruction_adjustment_m\": {},\n{indent}  \"max_camera_obstruction_hits\": {},\n{indent}  \"avg_desired_body_heading_error_degrees\": {},\n{indent}  \"p95_desired_body_heading_error_degrees\": {},\n{indent}  \"max_desired_body_heading_error_degrees\": {},\n{indent}  \"max_body_yaw_error_step_degrees\": {},\n{indent}  \"body_yaw_oscillation_count\": {},\n{indent}  \"max_body_roll_step_degrees\": {},\n{indent}  \"max_right_body_bank_degrees\": {},\n{indent}  \"max_left_body_bank_degrees\": {},\n{indent}  \"max_desired_heading_alignment_mps\": {},\n{indent}  \"max_lateral_response_mps\": {},\n{indent}  \"lateral_response_latency_secs\": {},\n{indent}  \"max_right_lateral_response_mps\": {},\n{indent}  \"right_lateral_response_latency_secs\": {},\n{indent}  \"max_left_lateral_response_mps\": {},\n{indent}  \"left_lateral_response_latency_secs\": {},\n{indent}  \"max_backward_lateral_response_mps\": {},\n{indent}  \"backward_lateral_response_latency_secs\": {},\n{indent}  \"max_backward_right_lateral_response_mps\": {},\n{indent}  \"backward_right_lateral_response_latency_secs\": {},\n{indent}  \"max_backward_left_lateral_response_mps\": {},\n{indent}  \"backward_left_lateral_response_latency_secs\": {},\n{indent}  \"max_air_brake_speed_drop_mps\": {},\n{indent}  \"max_post_brake_forward_alignment_mps\": {},\n{indent}  \"min_target_distance_m\": {},\n{indent}  \"final_target_distance_m\": {},\n{indent}  \"min_camera_pitch_degrees\": {},\n{indent}  \"max_camera_pitch_degrees\": {},\n{indent}  \"max_abs_camera_yaw_offset_degrees\": {},\n{indent}  \"min_camera_pitch_offset_degrees\": {},\n{indent}  \"max_camera_pitch_offset_degrees\": {},\n{indent}  \"max_visible_wind_fields\": {},\n{indent}  \"max_active_lift_fields\": {},\n{indent}  \"max_readable_lift_fields\": {},\n{indent}  \"max_sky_island_count\": {},\n{indent}  \"max_active_chunk_count\": {},\n{indent}  \"max_active_island_count\": {},\n{indent}  \"max_near_lod_islands\": {},\n{indent}  \"max_mid_lod_islands\": {},\n{indent}  \"max_far_lod_islands\": {},\n{indent}  \"max_visible_island_terrain_count\": {},\n{indent}  \"max_hidden_island_terrain_count\": {},\n{indent}  \"max_visible_island_impostor_count\": {},\n{indent}  \"max_hidden_island_impostor_count\": {},\n{indent}  \"max_visible_island_detail_count\": {},\n{indent}  \"max_hidden_island_detail_count\": {},\n{indent}  \"max_visible_route_beacon_count\": {},\n{indent}  \"max_weather_cloud_count\": {},\n{indent}  \"max_environment_motion_visual_count\": {},\n{indent}  \"max_environment_motion_offset_m\": {},\n{indent}  \"min_island_terrain_surface_count\": {},\n{indent}  \"min_island_terrain_mesh_vertices\": {},\n{indent}  \"min_island_terrain_color_bands\": {},\n{indent}  \"min_island_terrain_material_weight_bands\": {},\n{indent}  \"min_island_terrain_material_channels\": {},\n{indent}  \"min_island_terrain_material_regions\": {},\n{indent}  \"min_island_terrain_texture_detail_bands\": {},\n{indent}  \"min_island_terrain_relief_range_m\": {},\n{indent}  \"min_island_cliff_color_bands\": {},\n{indent}  \"min_procedural_island_body_count\": {},\n{indent}  \"max_primitive_island_body_count\": {},\n{indent}  \"min_island_body_silhouette_segments\": {},\n{indent}  \"max_avg_island_body_silhouette_segments\": {},\n{indent}  \"max_island_body_mesh_vertices\": {},\n{indent}  \"min_generated_ground_cover_patch_count\": {},\n{indent}  \"min_ground_cover_blade_count\": {},\n{indent}  \"min_ground_cover_mesh_vertices\": {},\n{indent}  \"min_generated_tree_trunk_count\": {},\n{indent}  \"min_generated_tree_canopy_count\": {},\n{indent}  \"min_tree_trunk_mesh_vertices\": {},\n{indent}  \"min_tree_canopy_mesh_vertices\": {},\n{indent}  \"min_detail_biome_palette_count\": {},\n{indent}  \"min_generated_rock_count\": {},\n{indent}  \"min_rock_mesh_vertices\": {},\n{indent}  \"min_generated_weather_cloud_count\": {},\n{indent}  \"min_generated_weather_cloud_bank_count\": {},\n{indent}  \"min_weather_cloud_bank_depth_m\": {},\n{indent}  \"min_weather_cloud_lobe_count\": {},\n{indent}  \"min_max_weather_cloud_lobe_count\": {},\n{indent}  \"min_weather_cloud_mesh_vertices\": {},\n{indent}  \"max_resident_island_visual_count\": {},\n{indent}  \"max_stream_visibility_changes_per_frame\": {},\n{indent}  \"total_stream_visibility_changes\": {},\n{indent}  \"max_catalog_island_visual_count\": {},\n{indent}  \"max_hidden_island_visual_count\": {},\n{indent}  \"max_resident_island_visual_fraction\": {},\n{indent}  \"max_stream_spawned_visuals_per_frame\": {},\n{indent}  \"max_stream_despawned_visuals_per_frame\": {},\n{indent}  \"total_stream_spawned_visuals\": {},\n{indent}  \"total_stream_despawned_visuals\": {},\n{indent}  \"max_entity_count\": {},\n{indent}  \"objective_total_count\": {},\n{indent}  \"max_completed_objective_count\": {},\n{indent}  \"final_objective_completed_count\": {},\n{indent}  \"min_objective_distance_m\": {},\n{indent}  \"final_objective_distance_m\": {},\n{indent}  \"objective_complete_samples\": {},\n{indent}  \"max_visual_asset_slot_count\": {},\n{indent}  \"max_gltf_scene_asset_slot_count\": {},\n{indent}  \"max_ready_visual_asset_slot_count\": {},\n{indent}  \"max_placeholder_visual_asset_slot_count\": {},\n{indent}  \"max_streaming_visual_asset_slot_count\": {},\n{indent}  \"max_missing_visual_asset_slot_count\": {},\n{indent}  \"max_queued_visual_asset_scene_count\": {},\n{indent}  \"max_loading_visual_asset_scene_count\": {},\n{indent}  \"max_loaded_visual_asset_scene_count\": {},\n{indent}  \"max_dependency_loaded_visual_asset_scene_count\": {},\n{indent}  \"max_preload_ready_visual_asset_scene_count\": {},\n{indent}  \"max_failed_visual_asset_scene_count\": {},\n{indent}  \"max_spawned_visual_asset_scene_count\": {},\n{indent}  \"max_ready_visual_asset_scene_count\": {},\n{indent}  \"max_visible_authored_world_fixture_count\": {},\n{indent}  \"max_always_visual_asset_slot_count\": {},\n{indent}  \"max_stream_window_visual_asset_slot_count\": {},\n{indent}  \"max_near_lod_visual_asset_slot_count\": {},\n{indent}  \"max_far_lod_visual_asset_slot_count\": {},\n{indent}  \"max_weather_visual_asset_slot_count\": {},\n{indent}  \"max_always_preload_ready_visual_asset_slot_count\": {},\n{indent}  \"max_streaming_preload_ready_visual_asset_slot_count\": {},\n{indent}  \"max_declared_animation_clip_count\": {},\n{indent}  \"max_ready_animation_clip_count\": {},\n{indent}  \"max_animation_player_count\": {},\n{indent}  \"max_animation_graph_count\": {},\n{indent}  \"max_power_up_count\": {},\n{indent}  \"min_visible_power_up_count\": {},\n{indent}  \"max_collected_power_up_count\": {},\n{indent}  \"power_up_effect_samples\": {},\n{indent}  \"total_power_up_activations\": {},\n{indent}  \"target_landing_samples\": {},\n{indent}  \"lifted_samples\": {},\n{indent}  \"readable_lift_samples\": {},\n{indent}  \"unreadable_lift_samples\": {},\n{indent}  \"gliding_samples\": {},\n{indent}  \"launching_samples\": {},\n{indent}  \"grounded_samples\": {}\n{indent}}}",
            self.sample_count,
            json_number(self.avg_frame_time_ms),
            json_number(self.p95_frame_time_ms),
            json_number(self.p99_frame_time_ms),
            json_number(self.max_frame_time_ms),
            json_number(self.horizontal_distance_m),
            json_number(self.max_altitude_m),
            json_number(self.min_altitude_m),
            json_number(self.max_grounded_visual_foot_gap_m),
            json_number(self.max_speed_mps),
            json_number(self.max_camera_distance_m),
            json_number(self.min_camera_surface_clearance_m),
            json_number(self.max_camera_player_angle_degrees),
            json_number(self.max_camera_step_distance_m),
            json_number(self.max_camera_rotation_delta_degrees),
            json_number(self.max_camera_orbit_alignment_degrees),
            json_number(self.avg_camera_follow_direction_error_degrees),
            json_number(self.p95_camera_follow_direction_error_degrees),
            json_number(self.max_camera_follow_direction_error_degrees),
            json_number(self.max_abs_camera_view_yaw_degrees),
            json_number(self.max_camera_view_yaw_drift_degrees),
            json_number(self.max_camera_world_yaw_drift_degrees),
            json_number(self.max_camera_obstruction_adjustment_m),
            self.max_camera_obstruction_hits,
            json_number(self.avg_desired_body_heading_error_degrees),
            json_number(self.p95_desired_body_heading_error_degrees),
            json_number(self.max_desired_body_heading_error_degrees),
            json_number(self.max_body_yaw_error_step_degrees),
            self.body_yaw_oscillation_count,
            json_number(self.max_body_roll_step_degrees),
            json_number(self.max_right_body_bank_degrees),
            json_number(self.max_left_body_bank_degrees),
            json_number(self.max_desired_heading_alignment_mps),
            json_number(self.max_lateral_response_mps),
            json_number(self.lateral_response_latency_secs),
            json_number(self.max_right_lateral_response_mps),
            json_number(self.right_lateral_response_latency_secs),
            json_number(self.max_left_lateral_response_mps),
            json_number(self.left_lateral_response_latency_secs),
            json_number(self.max_backward_lateral_response_mps),
            json_number(self.backward_lateral_response_latency_secs),
            json_number(self.max_backward_right_lateral_response_mps),
            json_number(self.backward_right_lateral_response_latency_secs),
            json_number(self.max_backward_left_lateral_response_mps),
            json_number(self.backward_left_lateral_response_latency_secs),
            json_number(self.max_air_brake_speed_drop_mps),
            json_number(self.max_post_brake_forward_alignment_mps),
            json_number(self.min_target_distance_m),
            json_number(self.final_target_distance_m),
            json_number(self.min_camera_pitch_degrees),
            json_number(self.max_camera_pitch_degrees),
            json_number(self.max_abs_camera_yaw_offset_degrees),
            json_number(self.min_camera_pitch_offset_degrees),
            json_number(self.max_camera_pitch_offset_degrees),
            self.max_visible_wind_fields,
            self.max_active_lift_fields,
            self.max_readable_lift_fields,
            self.max_sky_island_count,
            self.max_active_chunk_count,
            self.max_active_island_count,
            self.max_near_lod_islands,
            self.max_mid_lod_islands,
            self.max_far_lod_islands,
            self.max_visible_island_terrain_count,
            self.max_hidden_island_terrain_count,
            self.max_visible_island_impostor_count,
            self.max_hidden_island_impostor_count,
            self.max_visible_island_detail_count,
            self.max_hidden_island_detail_count,
            self.max_visible_route_beacon_count,
            self.max_weather_cloud_count,
            self.max_environment_motion_visual_count,
            json_number(self.max_environment_motion_offset_m),
            self.min_island_terrain_surface_count,
            self.min_island_terrain_mesh_vertices,
            self.min_island_terrain_color_bands,
            self.min_island_terrain_material_weight_bands,
            self.min_island_terrain_material_channels,
            self.min_island_terrain_material_regions,
            self.min_island_terrain_texture_detail_bands,
            json_number(self.min_island_terrain_relief_range_m),
            self.min_island_cliff_color_bands,
            self.min_procedural_island_body_count,
            self.max_primitive_island_body_count,
            self.min_island_body_silhouette_segments,
            json_number(self.max_avg_island_body_silhouette_segments),
            self.max_island_body_mesh_vertices,
            self.min_generated_ground_cover_patch_count,
            self.min_ground_cover_blade_count,
            self.min_ground_cover_mesh_vertices,
            self.min_generated_tree_trunk_count,
            self.min_generated_tree_canopy_count,
            self.min_tree_trunk_mesh_vertices,
            self.min_tree_canopy_mesh_vertices,
            self.min_detail_biome_palette_count,
            self.min_generated_rock_count,
            self.min_rock_mesh_vertices,
            self.min_generated_weather_cloud_count,
            self.min_generated_weather_cloud_bank_count,
            json_number(self.min_weather_cloud_bank_depth_m),
            self.min_weather_cloud_lobe_count,
            self.min_max_weather_cloud_lobe_count,
            self.min_weather_cloud_mesh_vertices,
            self.max_resident_island_visual_count,
            self.max_stream_visibility_changes_per_frame,
            self.total_stream_visibility_changes,
            self.max_catalog_island_visual_count,
            self.max_hidden_island_visual_count,
            json_number(self.max_resident_island_visual_fraction),
            self.max_stream_spawned_visuals_per_frame,
            self.max_stream_despawned_visuals_per_frame,
            self.total_stream_spawned_visuals,
            self.total_stream_despawned_visuals,
            self.max_entity_count,
            self.objective_total_count,
            self.max_completed_objective_count,
            self.final_objective_completed_count,
            json_number(self.min_objective_distance_m),
            json_number(self.final_objective_distance_m),
            self.objective_complete_samples,
            self.max_visual_asset_slot_count,
            self.max_gltf_scene_asset_slot_count,
            self.max_ready_visual_asset_slot_count,
            self.max_placeholder_visual_asset_slot_count,
            self.max_streaming_visual_asset_slot_count,
            self.max_missing_visual_asset_slot_count,
            self.max_queued_visual_asset_scene_count,
            self.max_loading_visual_asset_scene_count,
            self.max_loaded_visual_asset_scene_count,
            self.max_dependency_loaded_visual_asset_scene_count,
            self.max_preload_ready_visual_asset_scene_count,
            self.max_failed_visual_asset_scene_count,
            self.max_spawned_visual_asset_scene_count,
            self.max_ready_visual_asset_scene_count,
            self.max_visible_authored_world_fixture_count,
            self.max_always_visual_asset_slot_count,
            self.max_stream_window_visual_asset_slot_count,
            self.max_near_lod_visual_asset_slot_count,
            self.max_far_lod_visual_asset_slot_count,
            self.max_weather_visual_asset_slot_count,
            self.max_always_preload_ready_visual_asset_slot_count,
            self.max_streaming_preload_ready_visual_asset_slot_count,
            self.max_declared_animation_clip_count,
            self.max_ready_animation_clip_count,
            self.max_animation_player_count,
            self.max_animation_graph_count,
            self.max_power_up_count,
            self.min_visible_power_up_count,
            self.max_collected_power_up_count,
            self.power_up_effect_samples,
            self.total_power_up_activations,
            self.target_landing_samples,
            self.lifted_samples,
            self.readable_lift_samples,
            self.unreadable_lift_samples,
            self.gliding_samples,
            self.launching_samples,
            self.grounded_samples,
        );
        let air_brake_key = format!("{indent}  \"max_air_brake_speed_drop_mps\"");
        let rear_response_metrics = format!(
            "{indent}  \"max_backward_right_rear_response_mps\": {},\n{indent}  \"max_backward_left_rear_response_mps\": {},\n{}",
            json_number(self.max_backward_right_rear_response_mps),
            json_number(self.max_backward_left_rear_response_mps),
            air_brake_key
        );
        let json = json.replacen(&air_brake_key, &rear_response_metrics, 1);
        let post_brake_key = format!("{indent}  \"max_post_brake_forward_alignment_mps\"");
        let planar_brake_metrics = format!(
            "{indent}  \"max_air_brake_planar_speed_drop_mps\": {},\n{}",
            json_number(self.max_air_brake_planar_speed_drop_mps),
            post_brake_key
        );
        let json = json.replacen(&post_brake_key, &planar_brake_metrics, 1);
        let procedural_body_key = format!("{indent}  \"min_procedural_island_body_count\"");
        let impostor_metrics = format!(
            "{indent}  \"min_island_impostor_mesh_vertices\": {},\n{indent}  \"min_island_impostor_color_bands\": {},\n{}",
            self.min_island_impostor_mesh_vertices,
            self.min_island_impostor_color_bands,
            procedural_body_key
        );
        let json = json.replacen(&procedural_body_key, &impostor_metrics, 1);
        let body_mesh_key = format!("{indent}  \"max_island_body_mesh_vertices\"");
        let body_mesh_metrics = format!(
            "{indent}  \"min_island_body_mesh_vertices\": {},\n{}",
            self.min_island_body_mesh_vertices, body_mesh_key
        );
        let json = json.replacen(&body_mesh_key, &body_mesh_metrics, 1);
        let deferred_asset_key = format!("{indent}  \"max_queued_visual_asset_scene_count\"");
        let deferred_asset_metrics = format!(
            "{indent}  \"max_deferred_visual_asset_scene_count\": {},\n{}",
            self.max_deferred_visual_asset_scene_count, deferred_asset_key
        );
        json.replacen(&deferred_asset_key, &deferred_asset_metrics, 1)
    }
}

#[derive(Clone, Debug)]
pub struct EvalCheck {
    pub name: &'static str,
    pub passed: bool,
    pub value: f32,
    pub threshold: f32,
    pub comparator: &'static str,
    pub unit: &'static str,
}

impl EvalCheck {
    fn at_least(name: &'static str, value: f32, threshold: f32, unit: &'static str) -> Self {
        Self {
            name,
            passed: value >= threshold,
            value,
            threshold,
            comparator: ">=",
            unit,
        }
    }

    fn at_most(name: &'static str, value: f32, threshold: f32, unit: &'static str) -> Self {
        Self {
            name,
            passed: value <= threshold,
            value,
            threshold,
            comparator: "<=",
            unit,
        }
    }

    fn to_json(&self, indent: &str) -> String {
        format!(
            "{{\n{indent}  \"name\": {},\n{indent}  \"passed\": {},\n{indent}  \"value\": {},\n{indent}  \"comparator\": {},\n{indent}  \"threshold\": {},\n{indent}  \"unit\": {}\n{indent}}}",
            json_string(self.name),
            self.passed,
            json_number(self.value),
            json_string(self.comparator),
            json_number(self.threshold),
            json_string(self.unit),
        )
    }
}

#[derive(Clone, Debug)]
pub struct EvalSummary {
    pub scenario_name: &'static str,
    pub target_island_name: Option<&'static str>,
    pub passed: bool,
    pub frame_count: u32,
    pub duration_secs: f32,
    pub thresholds: EvalThresholds,
    pub metrics: EvalMetricsSummary,
    pub checks: Vec<EvalCheck>,
    pub artifacts: EvalArtifacts,
    pub final_sample: Option<EvalSample>,
}

impl EvalSummary {
    pub fn to_json(&self) -> String {
        let checks = self
            .checks
            .iter()
            .map(|check| check.to_json("      "))
            .collect::<Vec<_>>()
            .join(",\n");
        let final_sample = self
            .final_sample
            .as_ref()
            .map(EvalSample::to_json)
            .unwrap_or_else(|| "null".to_string());
        let target_island = self
            .target_island_name
            .map(json_string)
            .unwrap_or_else(|| "null".to_string());

        format!(
            "{{\n  \"scenario\": {},\n  \"target_island\": {},\n  \"passed\": {},\n  \"frame_count\": {},\n  \"duration_secs\": {},\n  \"thresholds\": {},\n  \"metrics\": {},\n  \"checks\": [\n{}\n  ],\n  \"artifacts\": {},\n  \"final_sample\": {}\n}}\n",
            json_string(self.scenario_name),
            target_island,
            self.passed,
            self.frame_count,
            json_number(self.duration_secs),
            self.thresholds.to_json("  "),
            self.metrics.to_json("  "),
            checks,
            self.artifacts.to_json("  "),
            final_sample,
        )
    }
}

fn vec3_array(value: Vec3) -> [f32; 3] {
    [value.x, value.y, value.z]
}

fn horizontal_distance(start: [f32; 3], end: [f32; 3]) -> f32 {
    let dx = end[0] - start[0];
    let dz = end[2] - start[2];
    (dx * dx + dz * dz).sqrt()
}

fn percentile(sorted_values: &[f32], percentile: f32) -> f32 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = ((sorted_values.len() as f32 * percentile).ceil() as usize)
        .saturating_sub(1)
        .min(sorted_values.len() - 1);
    sorted_values[index]
}

fn json_array3(values: [f32; 3]) -> String {
    format!(
        "[{},{},{}]",
        json_number(values[0]),
        json_number(values[1]),
        json_number(values[2])
    )
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn json_number(value: f32) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "null".to_string()
    }
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use crate::asset_pipeline::{
        ALWAYS_VISUAL_ASSET_SLOT_COUNT, DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
        FAR_LOD_VISUAL_ASSET_SLOT_COUNT, GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
        MIN_READY_VISUAL_ANIMATION_CLIP_COUNT, MIN_VISUAL_ANIMATION_GRAPH_COUNT,
        MIN_VISUAL_ANIMATION_PLAYER_COUNT, NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
        STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT, STREAMING_VISUAL_ASSET_SLOT_COUNT,
        VISUAL_ASSET_SLOT_COUNT, WEATHER_VISUAL_ASSET_SLOT_COUNT,
    };
    use crate::camera::CameraInput;
    use crate::environment::AERIAL_POWER_UP_ROUTE;
    use crate::movement::FlightInput;

    use super::*;

    #[test]
    fn baseline_route_has_scripted_launch_and_glide() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");

        assert!(scripted_input(scenario, 1).launch);
        assert!(!scripted_input(scenario, 2).launch);
        assert!(scripted_input(scenario, 60).glide);
    }

    #[test]
    fn ground_taxi_script_exercises_wasd_without_launching() {
        let scenario = scenario_named(GROUND_TAXI_CONTROL).expect("ground taxi route exists");

        assert!(scripted_input(scenario, 20).forward);
        assert!(scripted_input(scenario, 60).right);
        assert!(scripted_input(scenario, 135).backward);
        assert!(!scripted_input(scenario, 1).launch);
        assert!(!scripted_input(scenario, 60).glide);
    }

    #[test]
    fn updraft_route_steers_toward_lift_without_diving() {
        let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");

        assert!(scripted_input(scenario, 1).launch);
        assert!(scripted_input(scenario, 90).right);
        assert!(scripted_input(scenario, 180).glide);
        assert!(!scripted_input(scenario, 180).dive);
        assert_eq!(scenario.thresholds.min_completed_objective_count, 1);
    }

    #[test]
    fn island_launch_script_releases_forward_after_touchdown() {
        let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");

        assert!(scripted_input(scenario, 360).forward);
        assert!(scripted_input(scenario, 423).forward);
        assert!(!scripted_input(scenario, 430).forward);
        assert!(scenario.thresholds.require_target_landing);
    }

    #[test]
    fn branch_recovery_route_targets_named_recovery_island() {
        let scenario = scenario_named(BRANCH_RECOVERY_ROUTE).expect("branch route exists");

        assert_eq!(scenario.target_island_name, Some("sunlit terrace"));
        assert!(scenario.thresholds.require_target_landing);
        assert_eq!(scenario.thresholds.min_objective_total_count, 3);
        assert_eq!(scenario.thresholds.min_completed_objective_count, 3);
        assert!(scripted_input(scenario, 1).launch);
        assert!(scripted_input(scenario, 540).dive);
        assert!(scripted_input(scenario, 624).backward);
        assert!(!scripted_input(scenario, 750).forward);
    }

    #[test]
    fn camera_mouse_script_exercises_x_and_y_axes() {
        let scenario = scenario_named(CAMERA_MOUSE_CONTROL).expect("camera route exists");

        assert!(scripted_camera_input(scenario, 30).mouse_delta.x > 0.0);
        assert!(scripted_camera_input(scenario, 70).mouse_delta.y < 0.0);
        assert!(scripted_camera_input(scenario, 105).mouse_delta.y > 0.0);
        assert_eq!(
            scripted_input(scenario, 1),
            FlightInput::default(),
            "camera eval should not hide mouse regressions behind movement"
        );
    }

    #[test]
    fn camera_yaw_stability_script_applies_small_yaw_then_settles() {
        let scenario = scenario_named(CAMERA_YAW_STABILITY).expect("camera yaw route exists");

        assert!(scripted_camera_input(scenario, 18).mouse_delta.x > 0.0);
        assert_eq!(scripted_camera_input(scenario, 80), CameraInput::default());
        assert_eq!(
            scripted_input(scenario, 18),
            FlightInput::default(),
            "yaw stability eval should isolate mouse drift from movement"
        );
    }

    #[test]
    fn camera_turn_script_exercises_air_turns_and_air_brake() {
        let scenario = scenario_named(CAMERA_TURN_STABILITY).expect("turn route exists");

        assert!(scripted_input(scenario, 1).launch);
        assert!(scripted_input(scenario, 80).glide);
        assert!(scripted_input(scenario, 85).left);
        assert!(scripted_input(scenario, 115).right);
        assert!(scripted_input(scenario, 255).backward);
    }

    #[test]
    fn camera_strafe_script_exercises_lateral_input_without_mouse() {
        let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");

        assert!(scripted_input(scenario, 30).right);
        assert!(scripted_input(scenario, 130).left);
        assert_eq!(scripted_camera_input(scenario, 30), CameraInput::default());
        assert_eq!(scripted_camera_input(scenario, 130), CameraInput::default());
    }

    #[test]
    fn air_control_response_script_exercises_lateral_brake_and_recovery_without_mouse() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");

        assert!(scripted_input(scenario, 1).launch);
        assert!(scripted_input(scenario, 90).forward);
        assert!(scripted_input(scenario, 90).right);
        assert!(scripted_input(scenario, 140).right);
        assert!(!scripted_input(scenario, 140).forward);
        assert!(scripted_input(scenario, 210).left);
        assert!(scripted_input(scenario, 250).backward);
        assert!(scripted_input(scenario, 250).right);
        assert!(scripted_input(scenario, 310).backward);
        assert!(scripted_input(scenario, 310).left);
        assert!(scripted_input(scenario, 370).forward);
        assert_eq!(scripted_camera_input(scenario, 90), CameraInput::default());
        assert_eq!(scripted_camera_input(scenario, 210), CameraInput::default());
        assert_eq!(scripted_camera_input(scenario, 310), CameraInput::default());
        assert!(scenario.thresholds.min_gliding_samples >= 45);
    }

    #[test]
    fn long_glide_visibility_script_crosses_archipelago() {
        let scenario = scenario_named(LONG_GLIDE_VISIBILITY).expect("long glide route exists");

        assert!(scripted_input(scenario, 1).launch);
        assert!(scripted_input(scenario, 120).right);
        assert!(scripted_input(scenario, 160).left);
        assert!(scripted_input(scenario, 620).glide);
        assert!(!scripted_input(scenario, 620).dive);
        assert!(scenario.thresholds.min_sky_island_count >= 12);
        assert_eq!(scenario.thresholds.min_power_up_count, 3);
        assert_eq!(scenario.thresholds.min_collected_power_up_count, 3);
        assert!(scenario.thresholds.min_power_up_effect_samples >= 3);
    }

    #[test]
    fn scenarios_define_non_final_camera_checkpoints() {
        for name in SCENARIO_NAMES {
            let scenario = scenario_named(name).expect("scenario exists");

            assert!(!scenario.checkpoints.is_empty());
            assert!(
                scenario
                    .checkpoints
                    .iter()
                    .all(|checkpoint| checkpoint.frame < scenario.frame_count)
            );
            assert_eq!(
                scenario.checkpoint_at(scenario.checkpoints[0].frame),
                Some(scenario.checkpoints[0])
            );
        }
    }

    #[test]
    fn accumulator_summarizes_frame_time_percentiles() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut accumulator = EvalAccumulator::default();
        for frame_time_ms in [8.0, 16.0, 33.0, 50.0] {
            accumulator.observe_frame_time_ms(frame_time_ms);
        }

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );

        assert_eq!(summary.metrics.avg_frame_time_ms, 26.75);
        assert_eq!(summary.metrics.p95_frame_time_ms, 50.0);
        assert_eq!(summary.metrics.p99_frame_time_ms, 50.0);
        assert_eq!(summary.metrics.max_frame_time_ms, 50.0);
    }

    #[test]
    fn accumulator_requires_both_air_control_lateral_phases() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
        let mut accumulator = EvalAccumulator::default();

        accumulator.observe(air_control_metric_sample(
            scenario,
            0,
            Vec3::new(0.0, 0.0, -18.0),
            Vec2::new(0.0, 1.0),
            0.0,
            18.0,
            8.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            90,
            Vec3::new(20.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            20.0,
            18.0,
            4.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            210,
            Vec3::new(14.0, -2.0, -18.0),
            Vec2::new(-1.0, 0.0),
            2.0,
            18.0,
            4.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            270,
            Vec3::new(12.0, -2.0, 8.0),
            Vec2::new(1.0, -1.0),
            12.0,
            18.0,
            4.0,
        ));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let right_check = summary
            .checks
            .iter()
            .find(|check| check.name == "air_control_right_lateral_response")
            .expect("right response check exists");
        let left_check = summary
            .checks
            .iter()
            .find(|check| check.name == "air_control_left_lateral_response")
            .expect("left response check exists");

        assert!(right_check.passed);
        assert!(!left_check.passed);
        assert_eq!(summary.metrics.max_right_lateral_response_mps, 20.0);
        assert_eq!(summary.metrics.max_left_lateral_response_mps, 2.0);
    }

    #[test]
    fn accumulator_requires_backward_diagonal_air_control_response() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
        let mut accumulator = EvalAccumulator::default();

        accumulator.observe(air_control_metric_sample(
            scenario,
            0,
            Vec3::new(0.0, 0.0, -18.0),
            Vec2::new(0.0, 1.0),
            0.0,
            18.0,
            8.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            90,
            Vec3::new(20.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            20.0,
            18.0,
            4.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            210,
            Vec3::new(-20.0, -2.0, -18.0),
            Vec2::new(-1.0, 0.0),
            20.0,
            18.0,
            4.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            270,
            Vec3::new(2.0, -2.0, 8.0),
            Vec2::new(1.0, -1.0),
            2.0,
            18.0,
            4.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            320,
            Vec3::new(-2.0, -2.0, 8.0),
            Vec2::new(-1.0, -1.0),
            2.0,
            18.0,
            4.0,
        ));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let aggregate_check = named_check(&summary, "air_control_backward_lateral_response");
        let backward_right_check =
            named_check(&summary, "air_control_backward_right_lateral_response");
        let backward_left_check =
            named_check(&summary, "air_control_backward_left_lateral_response");

        assert_eq!(summary.metrics.max_backward_lateral_response_mps, 2.0);
        assert_eq!(summary.metrics.max_backward_right_lateral_response_mps, 2.0);
        assert_eq!(summary.metrics.max_backward_left_lateral_response_mps, 2.0);
        assert!(!aggregate_check.passed);
        assert!(!backward_right_check.passed);
        assert!(!backward_left_check.passed);
    }

    #[test]
    fn accumulator_requires_backward_diagonal_rear_component() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
        let mut accumulator = EvalAccumulator::default();
        let lateral_only_diagonal_alignment = 12.0 / std::f32::consts::SQRT_2;

        accumulator.observe(air_control_metric_sample(
            scenario,
            0,
            Vec3::new(0.0, 0.0, -18.0),
            Vec2::new(0.0, 1.0),
            0.0,
            20.0,
            8.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            90,
            Vec3::new(20.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            20.0,
            20.0,
            4.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            210,
            Vec3::new(-20.0, -2.0, -18.0),
            Vec2::new(-1.0, 0.0),
            20.0,
            20.0,
            4.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            270,
            Vec3::new(12.0, -2.0, 0.0),
            Vec2::new(1.0, -1.0),
            12.0,
            lateral_only_diagonal_alignment,
            4.0,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            320,
            Vec3::new(-12.0, -2.0, 0.0),
            Vec2::new(-1.0, -1.0),
            12.0,
            lateral_only_diagonal_alignment,
            4.0,
        ));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let backward_right_lateral_check =
            named_check(&summary, "air_control_backward_right_lateral_response");
        let backward_left_lateral_check =
            named_check(&summary, "air_control_backward_left_lateral_response");
        let backward_right_rear_check =
            named_check(&summary, "air_control_backward_right_rear_response");
        let backward_left_rear_check =
            named_check(&summary, "air_control_backward_left_rear_response");

        assert!(backward_right_lateral_check.passed);
        assert!(backward_left_lateral_check.passed);
        assert!(!backward_right_rear_check.passed);
        assert!(!backward_left_rear_check.passed);
        assert!(summary.metrics.max_backward_right_rear_response_mps.abs() < 0.001);
        assert!(summary.metrics.max_backward_left_rear_response_mps.abs() < 0.001);
        let summary_json = summary.to_json();
        assert!(summary_json.contains("\"max_backward_right_rear_response_mps\""));
        assert!(summary_json.contains("\"max_backward_left_rear_response_mps\""));
    }

    #[test]
    fn accumulator_gates_air_control_camera_follow_lag() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
        let mut accumulator = EvalAccumulator::default();

        for (frame, movement_axis) in [
            (0, Vec2::new(0.0, 1.0)),
            (90, Vec2::new(1.0, 0.0)),
            (210, Vec2::new(-1.0, 0.0)),
        ] {
            accumulator.observe(
                air_control_metric_sample(
                    scenario,
                    frame,
                    Vec3::new(20.0, -2.0, -18.0),
                    movement_axis,
                    20.0,
                    18.0,
                    4.0,
                )
                .with_camera_follow_metrics(72.0),
            );
        }

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let check = named_check(&summary, "air_control_avg_camera_follow_direction_error");

        assert_eq!(
            summary.metrics.avg_camera_follow_direction_error_degrees,
            72.0
        );
        assert_eq!(
            summary.metrics.max_camera_follow_direction_error_degrees,
            72.0
        );
        assert_eq!(check.value, 72.0);
        assert!(!check.passed);
    }

    #[test]
    fn accumulator_gates_air_control_follow_error_spikes() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
        let mut accumulator = EvalAccumulator::default();

        for frame in 0..20 {
            accumulator.observe(
                air_control_metric_sample(
                    scenario,
                    frame,
                    Vec3::new(16.0, -2.0, -18.0),
                    Vec2::new(0.0, 1.0),
                    0.0,
                    18.0,
                    4.0,
                )
                .with_camera_follow_metrics(0.0),
            );
        }
        for frame in [90, 210] {
            accumulator.observe(
                air_control_metric_sample(
                    scenario,
                    frame,
                    Vec3::new(20.0, -2.0, -18.0),
                    Vec2::new(1.0, 0.0),
                    20.0,
                    18.0,
                    4.0,
                )
                .with_camera_follow_metrics(90.0),
            );
        }

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let avg_check = named_check(&summary, "air_control_avg_camera_follow_direction_error");
        let p95_check = named_check(&summary, "air_control_p95_camera_follow_direction_error");

        assert!(avg_check.passed);
        assert_eq!(
            summary.metrics.p95_camera_follow_direction_error_degrees,
            90.0
        );
        assert_eq!(p95_check.value, 90.0);
        assert!(!p95_check.passed);
    }

    #[test]
    fn accumulator_gates_air_control_body_heading_spikes() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
        let mut accumulator = EvalAccumulator::default();

        for frame in 0..20 {
            accumulator.observe(air_control_metric_sample(
                scenario,
                frame,
                Vec3::new(16.0, -2.0, -18.0),
                Vec2::new(0.0, 1.0),
                0.0,
                18.0,
                3.0,
            ));
        }
        accumulator.observe(air_control_metric_sample(
            scenario,
            90,
            Vec3::new(20.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            20.0,
            18.0,
            90.0,
        ));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let avg_check = named_check(&summary, "air_control_avg_body_heading_error");
        let p95_check = named_check(&summary, "air_control_p95_body_heading_error");
        let max_check = named_check(&summary, "air_control_max_body_heading_error");
        let step_check = named_check(&summary, "air_control_max_body_yaw_error_step");

        assert!(avg_check.passed);
        assert!(p95_check.passed);
        assert_eq!(summary.metrics.max_desired_body_heading_error_degrees, 90.0);
        assert_eq!(max_check.value, 90.0);
        assert!(!max_check.passed);
        assert!(
            summary.metrics.max_body_yaw_error_step_degrees
                > AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES
        );
        assert_eq!(
            step_check.value,
            summary.metrics.max_body_yaw_error_step_degrees
        );
        assert!(!step_check.passed);
    }

    #[test]
    fn accumulator_gates_movement_only_camera_world_yaw_drift() {
        let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");
        let mut accumulator = EvalAccumulator::default();

        accumulator.observe(
            content_metric_sample(scenario, 0, 12, 0, 64).with_camera_world_yaw_metrics(0.0),
        );
        accumulator.observe(
            content_metric_sample(scenario, 60, 12, 0, 64).with_camera_world_yaw_metrics(20.0),
        );

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let check = named_check(&summary, "camera_strafe_world_yaw_drift");

        assert_eq!(summary.metrics.max_camera_world_yaw_drift_degrees, 20.0);
        assert_eq!(check.value, 20.0);
        assert!(!check.passed);
    }

    #[test]
    fn accumulator_resets_body_roll_step_across_grounded_samples() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
        let mut accumulator = EvalAccumulator::default();

        accumulator.observe(air_control_metric_sample(
            scenario,
            30,
            Vec3::new(14.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            16.0,
            18.0,
            4.0,
        ));
        let mut grounded = air_control_metric_sample(
            scenario,
            60,
            Vec3::new(0.0, 0.0, 0.0),
            Vec2::ZERO,
            0.0,
            f32::NAN,
            f32::NAN,
        );
        grounded.mode = FlightMode::Grounded.label();
        grounded.body_roll_degrees = 0.0;
        accumulator.observe(grounded);
        accumulator.observe(air_control_metric_sample(
            scenario,
            90,
            Vec3::new(-14.0, -2.0, -18.0),
            Vec2::new(-1.0, 0.0),
            16.0,
            18.0,
            -4.0,
        ));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );

        assert_eq!(summary.metrics.max_body_roll_step_degrees, 0.0);
        assert!(named_check(&summary, "air_control_max_body_roll_step").passed);
    }

    #[test]
    fn accumulator_gates_movement_only_camera_view_yaw_drift() {
        let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");
        let mut accumulator = EvalAccumulator::default();

        accumulator.observe(
            content_metric_sample(scenario, 0, 12, 0, 64).with_camera_view_yaw_metrics(0.0),
        );
        accumulator.observe(
            content_metric_sample(scenario, 60, 12, 0, 64).with_camera_view_yaw_metrics(12.0),
        );

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let check = named_check(&summary, "camera_strafe_view_yaw_drift");

        assert_eq!(summary.metrics.max_camera_view_yaw_drift_degrees, 12.0);
        assert_eq!(check.value, 12.0);
        assert!(!check.passed);
    }

    #[test]
    fn accumulator_gates_ground_strafe_directional_response() {
        let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");
        let mut accumulator = EvalAccumulator::default();

        accumulator.observe(
            content_metric_sample(scenario, 0, 12, 0, 64).with_movement_metrics(
                EvalMovementMetrics {
                    desired_body_yaw_error_degrees: f32::NAN,
                    body_roll_degrees: 0.0,
                    desired_heading_alignment_mps: f32::NAN,
                    lateral_response_mps: 9.0,
                    lateral_input_active: false,
                    movement_axis: Vec2::new(1.0, 0.0),
                },
            ),
        );
        accumulator.observe(
            content_metric_sample(scenario, 60, 12, 0, 64).with_movement_metrics(
                EvalMovementMetrics {
                    desired_body_yaw_error_degrees: f32::NAN,
                    body_roll_degrees: 0.0,
                    desired_heading_alignment_mps: f32::NAN,
                    lateral_response_mps: 3.0,
                    lateral_input_active: false,
                    movement_axis: Vec2::new(-1.0, 0.0),
                },
            ),
        );

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let right_check = named_check(&summary, "camera_strafe_right_lateral_response");
        let left_check = named_check(&summary, "camera_strafe_left_lateral_response");

        assert!(right_check.passed);
        assert_eq!(summary.metrics.max_right_lateral_response_mps, 9.0);
        assert_eq!(summary.metrics.max_left_lateral_response_mps, 3.0);
        assert!(!left_check.passed);
    }

    #[test]
    fn accumulator_gates_planar_air_brake_drop() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
        let mut accumulator = EvalAccumulator::default();

        accumulator.observe(air_control_metric_sample(
            scenario,
            240,
            Vec3::new(10.0, -52.0, 0.0),
            Vec2::new(0.0, -1.0),
            0.0,
            0.0,
            f32::NAN,
        ));
        accumulator.observe(air_control_metric_sample(
            scenario,
            245,
            Vec3::new(10.0, -8.0, 0.0),
            Vec2::new(0.0, -1.0),
            0.0,
            0.0,
            f32::NAN,
        ));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let total_speed_check = named_check(&summary, "air_control_air_brake_speed_drop");
        let planar_speed_check = named_check(&summary, "air_control_air_brake_planar_speed_drop");

        assert!(summary.metrics.max_air_brake_speed_drop_mps > 40.0);
        assert!(total_speed_check.passed);
        assert_eq!(summary.metrics.max_air_brake_planar_speed_drop_mps, 0.0);
        assert_eq!(planar_speed_check.value, 0.0);
        assert!(!planar_speed_check.passed);
        assert!(
            summary
                .to_json()
                .contains("\"max_air_brake_planar_speed_drop_mps\"")
        );
    }

    #[test]
    fn accumulator_gates_grounded_visual_foot_gap() {
        let scenario = scenario_named(GROUND_TAXI_CONTROL).expect("ground taxi route exists");
        let mut sample = content_metric_sample(scenario, 0, 12, 0, 96);
        sample.mode = FlightMode::Grounded.label();
        sample.visual_foot_gap_m = 0.18;

        let mut accumulator = EvalAccumulator::default();
        accumulator.observe(sample);

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let check = named_check(&summary, "grounded_visual_foot_gap");

        assert_eq!(summary.metrics.max_grounded_visual_foot_gap_m, 0.18);
        assert_eq!(check.value, 0.18);
        assert!(!check.passed);
    }

    fn air_control_metric_sample(
        scenario: EvalScenario,
        frame: u32,
        velocity: Vec3,
        movement_axis: Vec2,
        lateral_response_mps: f32,
        desired_alignment_mps: f32,
        yaw_error_degrees: f32,
    ) -> EvalSample {
        let objective = EvalObjectiveProgress::new(0, 2, "near route updraft", 120.0, false);
        EvalSample::new(
            frame,
            scenario.fixed_dt,
            Vec3::new(frame as f32 * 0.5, 42.0, -(frame as f32) * 0.25),
            velocity,
            FlightMode::Gliding,
            14.0,
            3.0,
            4.0,
            -18.0,
            0.0,
            0.0,
            0.2,
            1.0,
            0.0,
            0.0,
            0.0,
            0,
            0,
            3,
            0,
            0,
            1,
            140.0,
            false,
            objective,
            12,
            25,
            6,
            2,
            4,
            6,
            24,
            36,
            8,
            4,
            26,
            118,
            16,
            12,
            8,
            0.08,
            160,
            0,
            12,
            12,
            335,
            175,
            0.48,
            0,
            0,
            12,
            12,
            20,
            20,
            130,
            VISUAL_ASSET_SLOT_COUNT,
            GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            MIN_READY_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            STREAMING_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
            MIN_READY_VISUAL_ASSET_SCENE_COUNT,
            ALWAYS_VISUAL_ASSET_SLOT_COUNT,
            STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
            NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            WEATHER_VISUAL_ASSET_SLOT_COUNT,
            MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_VISUAL_ANIMATION_PLAYER_COUNT,
            MIN_VISUAL_ANIMATION_GRAPH_COUNT,
            AERIAL_POWER_UP_ROUTE.len(),
            AERIAL_POWER_UP_ROUTE.len(),
            0,
            0,
            0,
        )
        .with_content_metrics(12, 2305, 61, 0.8, 9, 12, 0, 96, 96.0, 1633, 1633)
        .with_island_impostor_metrics(146, 24)
        .with_terrain_material_metrics(36, 3, 4, 64)
        .with_generated_visual_shape_metrics(
            528, 220, 1100, 37, 37, 62, 412, 5, 60, 74, 30, 12, 4.8, 7, 14, 574,
        )
        .with_visible_authored_world_fixture_count(MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT)
        .with_movement_metrics(EvalMovementMetrics {
            desired_body_yaw_error_degrees: yaw_error_degrees,
            body_roll_degrees: -movement_axis.x.signum() * 12.0,
            desired_heading_alignment_mps: desired_alignment_mps,
            lateral_response_mps,
            lateral_input_active: movement_axis.x.abs() > f32::EPSILON,
            movement_axis,
        })
    }

    fn content_metric_sample(
        scenario: EvalScenario,
        frame: u32,
        procedural_body_count: usize,
        primitive_body_count: usize,
        silhouette_segments: usize,
    ) -> EvalSample {
        air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(12.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            14.0,
            18.0,
            8.0,
        )
        .with_content_metrics(
            12,
            2305,
            61,
            0.8,
            9,
            procedural_body_count,
            primitive_body_count,
            silhouette_segments,
            silhouette_segments as f32,
            1633,
            1633,
        )
        .with_island_impostor_metrics(146, 24)
    }

    fn named_check<'a>(summary: &'a EvalSummary, name: &str) -> &'a EvalCheck {
        summary
            .checks
            .iter()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("{name} check exists"))
    }

    #[test]
    fn accumulator_gates_authored_asset_readiness() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut sample = content_metric_sample(scenario, 0, 12, 0, 96);
        sample.ready_visual_asset_slot_count = 0;
        sample.placeholder_visual_asset_slot_count = VISUAL_ASSET_SLOT_COUNT;
        sample.missing_visual_asset_slot_count = VISUAL_ASSET_SLOT_COUNT;
        sample.deferred_visual_asset_scene_count = 1;
        sample.queued_visual_asset_scene_count = 0;
        sample.loaded_visual_asset_scene_count = 0;
        sample.dependency_loaded_visual_asset_scene_count = 0;
        sample.preload_ready_visual_asset_scene_count = 0;
        sample.spawned_visual_asset_scene_count = 0;
        sample.ready_visual_asset_scene_count = 0;
        sample.visible_authored_world_fixture_count = 0;
        sample.always_preload_ready_visual_asset_slot_count = 0;
        sample.streaming_preload_ready_visual_asset_slot_count = 0;
        sample.ready_animation_clip_count = 0;
        sample.animation_player_count = 0;
        sample.animation_graph_count = 0;

        let mut accumulator = EvalAccumulator::default();
        accumulator.observe(sample);
        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );

        for check_name in [
            "ready_visual_asset_slot_count",
            "missing_visual_asset_slot_count",
            "deferred_visual_asset_scene_count",
            "loaded_visual_asset_scene_count",
            "dependency_loaded_visual_asset_scene_count",
            "preload_ready_visual_asset_scene_count",
            "always_preload_ready_visual_asset_slot_count",
            "streaming_preload_ready_visual_asset_slot_count",
            "spawned_visual_asset_scene_count",
            "ready_visual_asset_scene_count",
            "visible_authored_world_fixture_count",
            "ready_animation_clip_count",
            "animation_player_count",
            "animation_graph_count",
        ] {
            assert!(
                !named_check(&summary, check_name).passed,
                "{check_name} should fail without a loaded authored scene"
            );
        }
    }

    #[test]
    fn accumulator_fails_when_procedural_body_count_disappears_after_startup() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut accumulator = EvalAccumulator::default();
        accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
        accumulator.observe(content_metric_sample(scenario, 10, 8, 0, 96));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let procedural_check = named_check(&summary, "procedural_island_body_count");

        assert!(!procedural_check.passed);
        assert_eq!(procedural_check.value, 8.0);
    }

    #[test]
    fn accumulator_fails_registered_primitive_or_low_silhouette_body_content() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut accumulator = EvalAccumulator::default();
        accumulator.observe(content_metric_sample(scenario, 0, 12, 1, 48));
        accumulator.observe(
            content_metric_sample(scenario, 5, 12, 0, 96)
                .with_content_metrics(12, 2305, 61, 0.8, 9, 12, 0, 96, 96.0, 900, 1633),
        );
        accumulator.observe(content_metric_sample(scenario, 10, 12, 0, 96));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let primitive_check = named_check(&summary, "primitive_island_body_count");
        let silhouette_check = named_check(&summary, "island_body_silhouette_segments");
        let mesh_check = named_check(&summary, "island_body_mesh_vertices");

        assert!(!primitive_check.passed);
        assert_eq!(primitive_check.value, 1.0);
        assert!(!silhouette_check.passed);
        assert_eq!(silhouette_check.value, 48.0);
        assert!(!mesh_check.passed);
        assert_eq!(mesh_check.value, 900.0);
    }

    #[test]
    fn accumulator_fails_low_detail_island_impostors() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut accumulator = EvalAccumulator::default();
        accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
        accumulator.observe(
            content_metric_sample(scenario, 10, 12, 0, 96).with_island_impostor_metrics(42, 4),
        );

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let mesh_check = named_check(&summary, "island_impostor_mesh_vertices");
        let color_check = named_check(&summary, "island_impostor_color_bands");

        assert!(!mesh_check.passed);
        assert_eq!(mesh_check.value, 42.0);
        assert!(!color_check.passed);
        assert_eq!(color_check.value, 4.0);
        assert!(
            summary
                .to_json()
                .contains("\"min_island_impostor_mesh_vertices\"")
        );
    }

    #[test]
    fn accumulator_fails_terrain_detail_regression() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut accumulator = EvalAccumulator::default();
        accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
        accumulator.observe(
            content_metric_sample(scenario, 10, 12, 0, 96)
                .with_content_metrics(10, 1200, 2, 0.2, 3, 12, 0, 96, 96.0, 1633, 1633)
                .with_terrain_material_metrics(4, 2, 2, 16),
        );

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let terrain_count_check = named_check(&summary, "island_terrain_surface_count");
        let terrain_vertex_check = named_check(&summary, "island_terrain_mesh_vertices");
        let terrain_color_check = named_check(&summary, "island_terrain_color_bands");
        let material_band_check = named_check(&summary, "island_terrain_material_weight_bands");
        let material_channel_check = named_check(&summary, "island_terrain_material_channels");
        let material_region_check = named_check(&summary, "island_terrain_material_regions");
        let texture_detail_check = named_check(&summary, "island_terrain_texture_detail_bands");
        let relief_check = named_check(&summary, "island_terrain_relief_range");
        let cliff_color_check = named_check(&summary, "island_cliff_color_bands");

        assert!(!terrain_count_check.passed);
        assert_eq!(terrain_count_check.value, 10.0);
        assert!(!terrain_vertex_check.passed);
        assert_eq!(terrain_vertex_check.value, 1200.0);
        assert!(!terrain_color_check.passed);
        assert_eq!(terrain_color_check.value, 2.0);
        assert!(!material_band_check.passed);
        assert_eq!(material_band_check.value, 4.0);
        assert!(!material_channel_check.passed);
        assert_eq!(material_channel_check.value, 2.0);
        assert!(!material_region_check.passed);
        assert_eq!(material_region_check.value, 2.0);
        assert!(!texture_detail_check.passed);
        assert_eq!(texture_detail_check.value, 16.0);
        assert!(!relief_check.passed);
        assert_eq!(relief_check.value, 0.2);
        assert!(!cliff_color_check.passed);
        assert_eq!(cliff_color_check.value, 3.0);
    }

    #[test]
    fn summary_json_exposes_terrain_detail_thresholds() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut accumulator = EvalAccumulator::default();
        accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let summary_json = summary.to_json();

        assert!(summary_json.contains("\"min_island_terrain_surface_count\": 12"));
        assert!(summary_json.contains("\"min_island_terrain_mesh_vertices\": 2305"));
        assert!(summary_json.contains("\"min_island_terrain_color_bands\": 61"));
        assert!(summary_json.contains("\"min_island_terrain_material_weight_bands\": 36"));
        assert!(summary_json.contains("\"min_island_terrain_material_channels\": 3"));
        assert!(summary_json.contains("\"min_island_terrain_material_regions\": 4"));
        assert!(summary_json.contains("\"min_island_terrain_texture_detail_bands\": 64"));
        assert!(summary_json.contains("\"min_island_terrain_relief_range_m\": 0.8000"));
        assert!(summary_json.contains("\"min_island_cliff_color_bands\": 9"));
        assert!(summary_json.contains("\"min_island_body_mesh_vertices\": 1633"));
        assert!(summary_json.contains("\"min_generated_ground_cover_patch_count\": 528"));
        assert!(summary_json.contains("\"min_ground_cover_blade_count\": 220"));
        assert!(summary_json.contains("\"min_ground_cover_mesh_vertices\": 1100"));
        assert!(summary_json.contains("\"min_tree_canopy_mesh_vertices\": 412"));
        assert!(summary_json.contains("\"min_detail_biome_palette_count\": 5"));
        assert!(summary_json.contains("\"min_generated_rock_count\": 60"));
        assert!(summary_json.contains("\"min_rock_mesh_vertices\": 74"));
        assert!(summary_json.contains("\"min_generated_weather_cloud_bank_count\": 12"));
        assert!(summary_json.contains("\"min_weather_cloud_bank_depth_m\": 4.8000"));
        assert!(summary_json.contains("\"min_weather_cloud_mesh_vertices\": 574"));
    }

    #[test]
    fn accumulator_fails_generated_visual_shape_regression() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut accumulator = EvalAccumulator::default();
        accumulator.observe(
            content_metric_sample(scenario, 0, 12, 0, 96).with_generated_visual_shape_metrics(
                528, 220, 1100, 12, 12, 62, 316, 5, 48, 74, 12, 12, 4.8, 6, 10, 270,
            ),
        );
        accumulator.observe(
            content_metric_sample(scenario, 10, 12, 0, 96).with_generated_visual_shape_metrics(
                10, 12, 60, 0, 0, 8, 45, 1, 1, 12, 0, 0, 0.4, 1, 1, 45,
            ),
        );

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        );
        let tree_count_check = named_check(&summary, "generated_tree_trunk_count");
        let ground_patch_check = named_check(&summary, "generated_ground_cover_patch_count");
        let ground_blade_check = named_check(&summary, "ground_cover_blade_count");
        let ground_vertex_check = named_check(&summary, "ground_cover_mesh_vertices");
        let canopy_vertex_check = named_check(&summary, "tree_canopy_mesh_vertices");
        let detail_palette_check = named_check(&summary, "detail_biome_palette_count");
        let rock_count_check = named_check(&summary, "generated_rock_count");
        let rock_vertex_check = named_check(&summary, "rock_mesh_vertices");
        let cloud_lobe_check = named_check(&summary, "weather_cloud_lobe_count");
        let cloud_bank_lobe_check = named_check(&summary, "weather_cloud_bank_lobe_count");
        let cloud_bank_count_check = named_check(&summary, "generated_weather_cloud_bank_count");
        let cloud_bank_depth_check = named_check(&summary, "weather_cloud_bank_depth");

        assert!(!ground_patch_check.passed);
        assert_eq!(ground_patch_check.value, 10.0);
        assert!(!ground_blade_check.passed);
        assert_eq!(ground_blade_check.value, 12.0);
        assert!(!ground_vertex_check.passed);
        assert_eq!(ground_vertex_check.value, 60.0);
        assert!(!tree_count_check.passed);
        assert_eq!(tree_count_check.value, 0.0);
        assert!(!canopy_vertex_check.passed);
        assert_eq!(canopy_vertex_check.value, 45.0);
        assert!(!detail_palette_check.passed);
        assert_eq!(detail_palette_check.value, 1.0);
        assert!(!rock_count_check.passed);
        assert_eq!(rock_count_check.value, 1.0);
        assert!(!rock_vertex_check.passed);
        assert_eq!(rock_vertex_check.value, 12.0);
        assert!(!cloud_lobe_check.passed);
        assert_eq!(cloud_lobe_check.value, 1.0);
        assert!(!cloud_bank_lobe_check.passed);
        assert_eq!(cloud_bank_lobe_check.value, 1.0);
        assert!(!cloud_bank_count_check.passed);
        assert_eq!(cloud_bank_count_check.value, 0.0);
        assert!(!cloud_bank_depth_check.passed);
        assert_eq!(cloud_bank_depth_check.value, 0.4);
    }

    #[test]
    fn accumulator_marks_current_baseline_shape_as_passing() {
        let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
        let mut accumulator = EvalAccumulator::default();
        let objective = EvalObjectiveProgress::new(0, 2, "near route updraft", 140.0, false);

        observe_current_content(
            &mut accumulator,
            EvalSample::new(
                0,
                scenario.fixed_dt,
                Vec3::new(0.0, 1.2, 0.0),
                Vec3::ZERO,
                FlightMode::Grounded,
                12.0,
                3.0,
                4.0,
                -20.0,
                0.0,
                0.0,
                0.2,
                2.0,
                0.0,
                0.0,
                0.0,
                0,
                0,
                3,
                0,
                0,
                1,
                140.0,
                false,
                objective,
                12,
                25,
                6,
                2,
                4,
                6,
                24,
                36,
                8,
                4,
                26,
                118,
                16,
                12,
                8,
                0.08,
                160,
                0,
                12,
                12,
                335,
                175,
                0.48,
                0,
                0,
                12,
                12,
                20,
                20,
                130,
                VISUAL_ASSET_SLOT_COUNT,
                GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                MIN_READY_VISUAL_ASSET_SLOT_COUNT,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                STREAMING_VISUAL_ASSET_SLOT_COUNT,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                0,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
                MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
                0,
                MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
                MIN_READY_VISUAL_ASSET_SCENE_COUNT,
                ALWAYS_VISUAL_ASSET_SLOT_COUNT,
                STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
                NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                WEATHER_VISUAL_ASSET_SLOT_COUNT,
                MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
                MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
                MIN_VISUAL_ANIMATION_PLAYER_COUNT,
                MIN_VISUAL_ANIMATION_GRAPH_COUNT,
                AERIAL_POWER_UP_ROUTE.len(),
                AERIAL_POWER_UP_ROUTE.len(),
                0,
                0,
                0,
            ),
        );
        observe_current_content(
            &mut accumulator,
            EvalSample::new(
                scenario.frame_count,
                scenario.fixed_dt,
                Vec3::new(0.0, 32.0, 140.0),
                Vec3::new(0.0, -4.0, 30.0),
                FlightMode::Gliding,
                14.0,
                3.0,
                4.0,
                -18.0,
                0.0,
                0.0,
                0.2,
                2.0,
                0.0,
                0.0,
                0.0,
                0,
                0,
                3,
                0,
                0,
                1,
                0.0,
                false,
                objective,
                12,
                25,
                6,
                2,
                4,
                6,
                24,
                36,
                8,
                4,
                26,
                118,
                16,
                12,
                8,
                0.08,
                160,
                0,
                12,
                12,
                335,
                175,
                0.48,
                0,
                0,
                12,
                12,
                20,
                20,
                130,
                VISUAL_ASSET_SLOT_COUNT,
                GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                MIN_READY_VISUAL_ASSET_SLOT_COUNT,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                STREAMING_VISUAL_ASSET_SLOT_COUNT,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                0,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
                MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
                0,
                MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
                MIN_READY_VISUAL_ASSET_SCENE_COUNT,
                ALWAYS_VISUAL_ASSET_SLOT_COUNT,
                STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
                NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                WEATHER_VISUAL_ASSET_SLOT_COUNT,
                MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
                MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
                MIN_VISUAL_ANIMATION_PLAYER_COUNT,
                MIN_VISUAL_ANIMATION_GRAPH_COUNT,
                AERIAL_POWER_UP_ROUTE.len(),
                AERIAL_POWER_UP_ROUTE.len(),
                0,
                0,
                0,
            ),
        );
        for frame in 1..=scenario.thresholds.min_gliding_samples {
            observe_current_content(
                &mut accumulator,
                EvalSample::new(
                    frame,
                    scenario.fixed_dt,
                    Vec3::new(0.0, 24.0, frame as f32 * 4.0),
                    Vec3::new(0.0, -3.0, 25.0),
                    FlightMode::Gliding,
                    13.0,
                    3.0,
                    4.0,
                    -18.0,
                    0.0,
                    0.0,
                    0.2,
                    2.0,
                    0.0,
                    0.0,
                    0.0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    1,
                    140.0 - frame as f32 * 4.0,
                    false,
                    objective,
                    12,
                    25,
                    6,
                    2,
                    4,
                    6,
                    24,
                    36,
                    8,
                    4,
                    26,
                    118,
                    16,
                    12,
                    8,
                    0.08,
                    160,
                    0,
                    12,
                    12,
                    335,
                    175,
                    0.48,
                    0,
                    0,
                    12,
                    12,
                    20,
                    20,
                    130,
                    VISUAL_ASSET_SLOT_COUNT,
                    GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                    MIN_READY_VISUAL_ASSET_SLOT_COUNT,
                    MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                    STREAMING_VISUAL_ASSET_SLOT_COUNT,
                    MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                    MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                    0,
                    MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                    MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
                    MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
                    0,
                    MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
                    MIN_READY_VISUAL_ASSET_SCENE_COUNT,
                    ALWAYS_VISUAL_ASSET_SLOT_COUNT,
                    STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
                    NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                    FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                    WEATHER_VISUAL_ASSET_SLOT_COUNT,
                    MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                    MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                    DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
                    MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
                    MIN_VISUAL_ANIMATION_PLAYER_COUNT,
                    MIN_VISUAL_ANIMATION_GRAPH_COUNT,
                    AERIAL_POWER_UP_ROUTE.len(),
                    AERIAL_POWER_UP_ROUTE.len(),
                    0,
                    0,
                    0,
                ),
            );
        }

        let summary = accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: vec!["checkpoints/glide_midroute.png".to_string()],
                checkpoint_marker_metadata: vec![
                    "checkpoints/glide_midroute.markers.json".to_string(),
                ],
            },
        );

        assert!(summary.passed);
        assert_eq!(summary.metrics.objective_total_count, 2);
        assert_eq!(summary.metrics.max_completed_objective_count, 0);
        assert!(summary.to_json().contains("\"passed\": true"));
        assert!(summary.to_json().contains("\"objective\":"));
        assert!(
            summary
                .to_json()
                .contains("\"checkpoint_screenshots\": [\"checkpoints/glide_midroute.png\"]")
        );
        assert!(summary.to_json().contains(
            "\"checkpoint_marker_metadata\": [\"checkpoints/glide_midroute.markers.json\"]"
        ));
    }

    fn observe_current_content(accumulator: &mut EvalAccumulator, sample: EvalSample) {
        accumulator.observe(
            sample
                .with_content_metrics(12, 2305, 61, 0.8, 9, 12, 0, 96, 96.0, 1633, 1633)
                .with_island_impostor_metrics(146, 24)
                .with_terrain_material_metrics(36, 3, 4, 64)
                .with_generated_visual_shape_metrics(
                    528, 220, 1100, 37, 37, 62, 412, 5, 60, 74, 30, 12, 4.8, 7, 14, 574,
                )
                .with_visible_authored_world_fixture_count(
                    MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT,
                ),
        );
    }
}
