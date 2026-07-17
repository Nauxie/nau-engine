#[path = "checks/assets.rs"]
mod assets;
#[path = "checks/content.rs"]
mod content;
#[path = "checks/control.rs"]
mod control;

use super::super::EvalAccumulator;
use super::derived::SummaryDerivedMetrics;
use crate::{
    environment::{GAMEPLAY_LIFT_ROUTE, VISUAL_CROSSWIND_FIELD_COUNT},
    eval::{
        scenarios::{
            CAMERA_MOUSE_CONTROL, CAMERA_STRAFE_STABILITY, CAMERA_TURN_STABILITY,
            CAMERA_YAW_STABILITY, EvalScenario, GROUND_TAXI_CONTROL, ISLAND_HERO_GALLERY,
            PLATEAU_ARRIVAL_CAMERA, RETURN_DESCENT_ROUTE, TERRAIN_BODY_COLLISION_CONTACT,
            TERRAIN_EDGE_WALKOFF, TERRAIN_RIM_COLLISION_CONTACT, UPDRAFT_ROUTE,
            WORLD_COLLISION_CONTACT,
        },
        summary::EvalCheck,
        thresholds::*,
    },
};

const PLATEAU_CAMERA_MIN_TREE_WORLD_COLLISION_PROXY_COUNT: usize = 4;
const PLATEAU_CAMERA_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT: usize = 5;
const WORLD_CONTACT_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT: usize = 12;
const CLOSE_CAMERA_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT: usize = 12;
const EDGE_WALKOFF_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT: usize = 5;
const EDGE_WALKOFF_MIN_NEAR_ISLAND_EDGE_SAMPLES: u32 = 16;
const EDGE_WALKOFF_MIN_OUTSIDE_ISLAND_FOOTPRINT_SAMPLES: u32 = 8;
const EDGE_WALKOFF_MIN_OUTSIDE_NEAR_ISLAND_EDGE_SAMPLES: u32 = 4;
const TERRAIN_CONTACT_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT: usize = 8;
const SHORT_CONTACT_SKIPPED_CHECKS: &[&str] = &[
    "updraft_visual_rise",
    "crosswind_guide_flow_displacement",
    "crosswind_ribbon_flow_displacement",
    "sustained_wind_visual_flow_samples",
    "sustained_updraft_visual_flow_samples",
    "sustained_crosswind_visual_flow_samples",
];
const ISLAND_HERO_GALLERY_SKIPPED_CHECKS: &[&str] = &[
    "world_collision_proxy_count",
    "terrain_rim_collision_proxy_count",
    "terrain_body_collision_proxy_count",
    "solid_world_collision_proxy_count",
    "tree_world_collision_proxy_count",
    "rock_world_collision_proxy_count",
    "landmark_world_collision_proxy_count",
];

impl EvalAccumulator {
    pub fn reclassify_latest_runtime_frame_as_eval_artifact(&mut self) -> bool {
        let Some(frame_time_ms) = self.runtime_frame_times_ms.pop() else {
            return false;
        };
        self.eval_artifact_frame_times_ms.push(frame_time_ms);
        true
    }
}

pub(super) fn build_checks(
    acc: &EvalAccumulator,
    scenario: EvalScenario,
    derived: &SummaryDerivedMetrics,
) -> Vec<EvalCheck> {
    let thresholds = scenario.thresholds;
    let min_tree_world_collision_proxy_count =
        if matches!(scenario.name, PLATEAU_ARRIVAL_CAMERA | RETURN_DESCENT_ROUTE) {
            PLATEAU_CAMERA_MIN_TREE_WORLD_COLLISION_PROXY_COUNT
        } else {
            MIN_TREE_WORLD_COLLISION_PROXY_COUNT
        };
    let min_rock_world_collision_proxy_count = match scenario.name {
        PLATEAU_ARRIVAL_CAMERA | RETURN_DESCENT_ROUTE => {
            PLATEAU_CAMERA_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT
        }
        WORLD_COLLISION_CONTACT => WORLD_CONTACT_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT,
        CAMERA_MOUSE_CONTROL
        | CAMERA_YAW_STABILITY
        | CAMERA_TURN_STABILITY
        | CAMERA_STRAFE_STABILITY => CLOSE_CAMERA_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT,
        TERRAIN_EDGE_WALKOFF => EDGE_WALKOFF_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT,
        TERRAIN_RIM_COLLISION_CONTACT | TERRAIN_BODY_COLLISION_CONTACT => {
            TERRAIN_CONTACT_MIN_ROCK_WORLD_COLLISION_PROXY_COUNT
        }
        _ => MIN_ROCK_WORLD_COLLISION_PROXY_COUNT,
    };

    let mut checks = vec![
        EvalCheck::at_least(
            "sample_count",
            acc.sample_count as f32,
            thresholds.min_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "horizontal_distance",
            derived.horizontal_distance_m,
            thresholds.min_horizontal_distance_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_altitude",
            acc.max_altitude_m,
            thresholds.min_max_altitude_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_speed",
            acc.max_speed_mps,
            thresholds.min_max_speed_mps,
            "m/s",
        ),
        EvalCheck::at_least(
            "gliding_samples",
            acc.gliding_samples as f32,
            thresholds.min_gliding_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "grounded_samples",
            acc.grounded_samples as f32,
            thresholds.min_grounded_samples as f32,
            "samples",
        ),
        EvalCheck::at_most(
            "grounded_visual_foot_gap",
            acc.max_grounded_visual_foot_gap_m,
            MAX_GROUNDED_VISUAL_FOOT_GAP_M,
            "m",
        ),
        EvalCheck::at_least(
            "lifted_samples",
            acc.lifted_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "sky_island_count",
            acc.max_sky_island_count as f32,
            thresholds.min_sky_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "active_island_count",
            acc.max_active_island_count as f32,
            thresholds.min_active_island_count as f32,
            "islands",
        ),
        EvalCheck::at_most(
            "active_chunk_count",
            acc.max_active_chunk_count as f32,
            thresholds.max_active_chunk_count as f32,
            "chunks",
        ),
        EvalCheck::at_least(
            "near_lod_island_count",
            acc.max_near_lod_islands as f32,
            thresholds.min_near_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "mid_lod_island_count",
            acc.max_mid_lod_islands as f32,
            thresholds.min_mid_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "far_lod_island_count",
            acc.max_far_lod_islands as f32,
            thresholds.min_far_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_most(
            "visible_island_terrain_count",
            acc.max_visible_island_terrain_count as f32,
            thresholds.max_visible_island_terrain_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "hidden_island_terrain_count",
            acc.max_hidden_island_terrain_count as f32,
            thresholds.min_hidden_island_terrain_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "visible_island_impostor_count",
            acc.max_visible_island_impostor_count as f32,
            thresholds.min_visible_island_impostor_count as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "visible_island_detail_count",
            acc.max_visible_island_detail_count as f32,
            thresholds.max_visible_island_detail_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "hidden_island_detail_count",
            acc.max_hidden_island_detail_count as f32,
            thresholds.min_hidden_island_detail_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "visible_route_beacon_count",
            acc.max_visible_route_beacon_count as f32,
            thresholds.min_visible_route_beacon_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "weather_cloud_count",
            acc.max_weather_cloud_count as f32,
            thresholds.min_weather_cloud_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "environment_motion_visual_count",
            acc.max_environment_motion_visual_count as f32,
            thresholds.min_environment_motion_visual_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "environment_motion_offset",
            acc.max_environment_motion_offset_m,
            thresholds.min_environment_motion_offset_m,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_guide_visual_count",
            acc.max_updraft_guide_visual_count as f32,
            MIN_UPDRAFT_GUIDE_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "updraft_ribbon_visual_count",
            acc.max_updraft_ribbon_visual_count as f32,
            MIN_UPDRAFT_RIBBON_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "crosswind_guide_visual_count",
            acc.max_crosswind_guide_visual_count as f32,
            MIN_CROSSWIND_GUIDE_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "crosswind_ribbon_visual_count",
            acc.max_crosswind_ribbon_visual_count as f32,
            MIN_CROSSWIND_RIBBON_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "updraft_field_count",
            acc.max_updraft_field_count as f32,
            GAMEPLAY_LIFT_ROUTE.len() as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "updraft_fields_with_guides",
            acc.max_updraft_fields_with_guides_count as f32,
            GAMEPLAY_LIFT_ROUTE.len() as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "updraft_fields_with_ribbons",
            acc.max_updraft_fields_with_ribbons_count as f32,
            GAMEPLAY_LIFT_ROUTE.len() as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "updraft_fields_with_guides_and_ribbons",
            acc.max_updraft_fields_with_guides_and_ribbons_count as f32,
            GAMEPLAY_LIFT_ROUTE.len() as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "updraft_flow_coherent_field_count",
            acc.max_updraft_flow_coherent_field_count as f32,
            GAMEPLAY_LIFT_ROUTE.len() as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "crosswind_field_count",
            acc.max_crosswind_field_count as f32,
            VISUAL_CROSSWIND_FIELD_COUNT as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "crosswind_fields_with_guides",
            acc.max_crosswind_fields_with_guides_count as f32,
            VISUAL_CROSSWIND_FIELD_COUNT as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "crosswind_fields_with_ribbons",
            acc.max_crosswind_fields_with_ribbons_count as f32,
            VISUAL_CROSSWIND_FIELD_COUNT as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "crosswind_fields_with_guides_and_ribbons",
            acc.max_crosswind_fields_with_guides_and_ribbons_count as f32,
            VISUAL_CROSSWIND_FIELD_COUNT as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "crosswind_flow_coherent_field_count",
            acc.max_crosswind_flow_coherent_field_count as f32,
            VISUAL_CROSSWIND_FIELD_COUNT as f32,
            "fields",
        ),
        EvalCheck::at_least(
            "updraft_visual_motion",
            acc.max_updraft_visual_motion_m,
            MIN_UPDRAFT_VISUAL_MOTION_M,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_visual_rise",
            acc.max_updraft_visual_rise_m,
            MIN_UPDRAFT_VISUAL_RISE_M,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_visual_swirl_displacement",
            acc.max_updraft_visual_swirl_displacement_m,
            MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_visual_depth_span",
            acc.max_updraft_visual_depth_span_m,
            MIN_UPDRAFT_VISUAL_DEPTH_SPAN_M,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_visual_scale_pulse",
            acc.max_updraft_visual_scale_pulse,
            MIN_UPDRAFT_VISUAL_SCALE_PULSE,
            "scale",
        ),
        EvalCheck::at_least(
            "crosswind_visual_motion",
            acc.max_crosswind_visual_motion_m,
            MIN_CROSSWIND_VISUAL_MOTION_M,
            "m",
        ),
        EvalCheck::at_least(
            "crosswind_guide_flow_displacement",
            acc.max_crosswind_guide_flow_displacement_m,
            MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_least(
            "crosswind_ribbon_flow_displacement",
            acc.max_crosswind_ribbon_flow_displacement_m,
            MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_least(
            "crosswind_visual_lane_depth_span",
            acc.max_crosswind_visual_lane_depth_span_m,
            MIN_CROSSWIND_VISUAL_LANE_DEPTH_SPAN_M,
            "m",
        ),
        EvalCheck::at_least(
            "crosswind_visual_scale_pulse",
            acc.max_crosswind_visual_scale_pulse,
            MIN_CROSSWIND_VISUAL_SCALE_PULSE,
            "scale",
        ),
        EvalCheck::at_least(
            "updraft_flow_coherent_visual_count",
            acc.max_updraft_flow_coherent_visual_count as f32,
            MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "crosswind_flow_coherent_visual_count",
            acc.max_crosswind_flow_coherent_visual_count as f32,
            MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "crosswind_ribbon_flow_coherent_sample_count",
            acc.max_crosswind_ribbon_flow_coherent_sample_count as f32,
            MIN_CROSSWIND_RIBBON_FLOW_COHERENT_SAMPLE_COUNT as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "updraft_visual_flow_alignment",
            acc.max_updraft_visual_flow_alignment,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            "dot",
        ),
        EvalCheck::at_least(
            "crosswind_visual_flow_alignment",
            acc.max_crosswind_visual_flow_alignment,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            "dot",
        ),
        EvalCheck::at_least(
            "crosswind_ribbon_visual_flow_alignment",
            acc.max_crosswind_ribbon_visual_flow_alignment,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            "dot",
        ),
        EvalCheck::at_least(
            "observed_updraft_flow_coherent_visual_count",
            acc.max_observed_updraft_flow_coherent_visual_count as f32,
            MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "observed_crosswind_flow_coherent_visual_count",
            acc.max_observed_crosswind_flow_coherent_visual_count as f32,
            MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "observed_crosswind_ribbon_flow_coherent_sample_count",
            acc.max_observed_crosswind_ribbon_flow_coherent_sample_count as f32,
            MIN_CROSSWIND_RIBBON_FLOW_COHERENT_SAMPLE_COUNT as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "observed_updraft_visual_frame_motion",
            acc.max_observed_updraft_visual_frame_motion_m,
            MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_MOTION_M,
            "m",
        ),
        EvalCheck::at_least(
            "observed_updraft_visual_frame_rise",
            acc.max_observed_updraft_visual_frame_rise_m,
            MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_RISE_M,
            "m",
        ),
        EvalCheck::at_least(
            "observed_updraft_visual_frame_swirl_displacement",
            acc.max_observed_updraft_visual_frame_swirl_displacement_m,
            MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_SWIRL_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_least(
            "observed_crosswind_visual_frame_motion",
            acc.max_observed_crosswind_visual_frame_motion_m,
            MIN_OBSERVED_CROSSWIND_VISUAL_FRAME_MOTION_M,
            "m",
        ),
        EvalCheck::at_least(
            "observed_crosswind_guide_frame_flow_displacement",
            acc.max_observed_crosswind_guide_frame_flow_displacement_m,
            MIN_OBSERVED_CROSSWIND_GUIDE_FRAME_FLOW_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_least(
            "observed_crosswind_ribbon_frame_flow_displacement",
            acc.max_observed_crosswind_ribbon_frame_flow_displacement_m,
            MIN_OBSERVED_CROSSWIND_RIBBON_FRAME_FLOW_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_most(
            "observed_updraft_visual_speed",
            acc.max_observed_updraft_visual_speed_mps,
            MAX_OBSERVED_UPDRAFT_VISUAL_SPEED_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "observed_crosswind_visual_speed",
            acc.max_observed_crosswind_visual_speed_mps,
            MAX_OBSERVED_CROSSWIND_VISUAL_SPEED_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "observed_wind_visual_acceleration",
            acc.max_observed_wind_visual_acceleration_mps2,
            MAX_OBSERVED_WIND_VISUAL_ACCELERATION_MPS2,
            "m/s^2",
        ),
        EvalCheck::at_most(
            "observed_wind_visual_jump_count",
            acc.observed_wind_visual_jump_count as f32,
            MAX_OBSERVED_WIND_VISUAL_JUMP_COUNT as f32,
            "jumps",
        ),
        EvalCheck::at_least(
            "observed_updraft_visual_flow_alignment",
            acc.max_observed_updraft_visual_flow_alignment,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            "dot",
        ),
        EvalCheck::at_least(
            "observed_crosswind_visual_flow_alignment",
            acc.max_observed_crosswind_visual_flow_alignment,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            "dot",
        ),
        EvalCheck::at_least(
            "observed_crosswind_ribbon_visual_flow_alignment",
            acc.max_observed_crosswind_ribbon_visual_flow_alignment,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            "dot",
        ),
        EvalCheck::at_least(
            "sustained_wind_visual_flow_samples",
            acc.sustained_wind_visual_flow_samples as f32,
            MIN_SUSTAINED_WIND_VISUAL_FLOW_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "sustained_updraft_visual_flow_samples",
            acc.sustained_updraft_visual_flow_samples as f32,
            MIN_SUSTAINED_UPDRAFT_VISUAL_FLOW_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "sustained_crosswind_visual_flow_samples",
            acc.sustained_crosswind_visual_flow_samples as f32,
            MIN_SUSTAINED_CROSSWIND_VISUAL_FLOW_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "sustained_crosswind_ribbon_advected_flow_samples",
            acc.sustained_crosswind_ribbon_advected_flow_samples as f32,
            MIN_SUSTAINED_CROSSWIND_RIBBON_ADVECTED_FLOW_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "world_collision_proxy_count",
            acc.max_world_collision_proxy_count as f32,
            MIN_WORLD_COLLISION_PROXY_COUNT as f32,
            "proxies",
        ),
        EvalCheck::at_least(
            "terrain_rim_collision_proxy_count",
            acc.max_terrain_rim_collision_proxy_count as f32,
            MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT as f32,
            "proxies",
        ),
        EvalCheck::at_least(
            "terrain_body_collision_proxy_count",
            acc.max_terrain_body_collision_proxy_count as f32,
            MIN_TERRAIN_BODY_COLLISION_PROXY_COUNT as f32,
            "proxies",
        ),
        EvalCheck::at_least(
            "solid_world_collision_proxy_count",
            acc.max_solid_world_collision_proxy_count as f32,
            MIN_SOLID_WORLD_COLLISION_PROXY_COUNT as f32,
            "proxies",
        ),
        EvalCheck::at_least(
            "tree_world_collision_proxy_count",
            acc.max_tree_world_collision_proxy_count as f32,
            min_tree_world_collision_proxy_count as f32,
            "proxies",
        ),
        EvalCheck::at_least(
            "rock_world_collision_proxy_count",
            acc.max_rock_world_collision_proxy_count as f32,
            min_rock_world_collision_proxy_count as f32,
            "proxies",
        ),
        EvalCheck::at_least(
            "landmark_world_collision_proxy_count",
            acc.max_landmark_world_collision_proxy_count as f32,
            MIN_LANDMARK_WORLD_COLLISION_PROXY_COUNT as f32,
            "proxies",
        ),
    ];

    if scenario.name == UPDRAFT_ROUTE {
        checks.extend([
            EvalCheck::at_least(
                "crosswind_neutral_drift_samples",
                acc.crosswind_neutral_drift_samples as f32,
                MIN_CROSSWIND_NEUTRAL_DRIFT_SAMPLE_COUNT as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "crosswind_neutral_horizontal_drift",
                acc.crosswind_neutral_horizontal_drift_m,
                MIN_CROSSWIND_NEUTRAL_HORIZONTAL_DRIFT_M,
                "m",
            ),
            EvalCheck::at_most(
                "crosswind_neutral_horizontal_step",
                acc.max_crosswind_neutral_horizontal_step_m,
                MAX_CROSSWIND_NEUTRAL_HORIZONTAL_STEP_M,
                "m/sample",
            ),
        ]);
    }

    if scenario.name == WORLD_COLLISION_CONTACT {
        checks.extend([
            EvalCheck::at_least(
                "world_collision_contact_samples",
                acc.world_collision_contact_samples as f32,
                MIN_WORLD_COLLISION_CONTACT_SAMPLES as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "world_collision_push",
                acc.max_world_collision_push_m,
                MIN_WORLD_COLLISION_CONTACT_PUSH_M,
                "m",
            ),
            EvalCheck::at_most(
                "world_collision_push_ceiling",
                acc.max_world_collision_push_m,
                MAX_WORLD_COLLISION_CONTACT_PUSH_M,
                "m",
            ),
        ]);
    }
    if scenario.name == TERRAIN_RIM_COLLISION_CONTACT {
        checks.extend([
            EvalCheck::at_least(
                "terrain_rim_collision_contact_samples",
                acc.terrain_rim_collision_contact_samples as f32,
                MIN_TERRAIN_RIM_COLLISION_CONTACT_SAMPLES as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "terrain_rim_collision_push",
                acc.max_terrain_rim_collision_push_m,
                MIN_WORLD_COLLISION_CONTACT_PUSH_M,
                "m",
            ),
            EvalCheck::at_most(
                "terrain_rim_collision_push_ceiling",
                acc.max_terrain_rim_collision_push_m,
                MAX_TERRAIN_RIM_COLLISION_CONTACT_PUSH_M,
                "m",
            ),
        ]);
    }
    if scenario.name == TERRAIN_BODY_COLLISION_CONTACT {
        checks.extend([
            EvalCheck::at_least(
                "terrain_body_collision_contact_samples",
                acc.terrain_body_collision_contact_samples as f32,
                MIN_TERRAIN_BODY_COLLISION_CONTACT_SAMPLES as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "terrain_body_collision_push",
                acc.max_terrain_body_collision_push_m,
                MIN_TERRAIN_BODY_COLLISION_CONTACT_PUSH_M,
                "m",
            ),
            EvalCheck::at_most(
                "terrain_body_collision_push_ceiling",
                acc.max_terrain_body_collision_push_m,
                MAX_TERRAIN_BODY_COLLISION_CONTACT_PUSH_M,
                "m",
            ),
            EvalCheck::at_most(
                "terrain_body_collision_rim_resolved_samples",
                acc.terrain_rim_collision_resolved_samples as f32,
                0.0,
                "samples",
            ),
        ]);
    }
    if scenario.name == TERRAIN_EDGE_WALKOFF {
        checks.extend([
            EvalCheck::at_least(
                "terrain_edge_walkoff_near_island_edge_samples",
                acc.near_island_edge_samples as f32,
                EDGE_WALKOFF_MIN_NEAR_ISLAND_EDGE_SAMPLES as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "terrain_edge_walkoff_outside_island_footprint_samples",
                acc.outside_island_footprint_samples as f32,
                EDGE_WALKOFF_MIN_OUTSIDE_ISLAND_FOOTPRINT_SAMPLES as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "terrain_edge_walkoff_outside_near_island_edge_samples",
                acc.outside_near_island_edge_samples as f32,
                EDGE_WALKOFF_MIN_OUTSIDE_NEAR_ISLAND_EDGE_SAMPLES as f32,
                "samples",
            ),
            EvalCheck::at_most(
                "terrain_edge_walkoff_world_collision_resolved_samples",
                acc.world_collision_resolved_samples as f32,
                0.0,
                "samples",
            ),
            EvalCheck::at_most(
                "terrain_edge_walkoff_rim_resolved_samples",
                acc.terrain_rim_collision_resolved_samples as f32,
                0.0,
                "samples",
            ),
            EvalCheck::at_most(
                "terrain_edge_walkoff_body_resolved_samples",
                acc.terrain_body_collision_resolved_samples as f32,
                0.0,
                "samples",
            ),
            EvalCheck::at_most(
                "terrain_edge_walkoff_collision_push",
                acc.max_world_collision_push_m,
                0.0,
                "m",
            ),
            EvalCheck::at_most(
                "terrain_edge_walkoff_camera_obstruction_snap_count",
                acc.camera_obstruction_snap_count as f32,
                0.0,
                "samples",
            ),
            EvalCheck::at_most(
                "terrain_edge_walkoff_launching_samples",
                acc.launching_samples as f32,
                0.0,
                "samples",
            ),
            EvalCheck::at_most(
                "terrain_edge_walkoff_launch_upward_speed",
                acc.max_launch_upward_speed_mps,
                0.0,
                "m/s",
            ),
            EvalCheck::at_most(
                "terrain_edge_walkoff_launch_horizontal_speed",
                acc.max_launch_horizontal_speed_mps,
                0.0,
                "m/s",
            ),
        ]);
    }
    if scenario.name == GROUND_TAXI_CONTROL {
        checks.push(EvalCheck::at_most(
            "ground_taxi_terrain_rim_contact_samples",
            acc.terrain_rim_collision_resolved_samples as f32,
            0.0,
            "samples",
        ));
    }

    content::append_content_checks(&mut checks, acc, &thresholds);

    checks.extend([
        EvalCheck::at_most(
            "resident_island_visual_count",
            acc.max_resident_island_visual_count as f32,
            thresholds.max_resident_island_visual_count as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "stream_visibility_changes_per_frame",
            acc.max_stream_visibility_changes_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_least(
            "hidden_island_visual_count",
            acc.max_hidden_island_visual_count as f32,
            (thresholds.min_hidden_island_terrain_count + thresholds.min_hidden_island_detail_count)
                as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "resident_island_visual_fraction",
            acc.max_resident_island_visual_fraction,
            MAX_RESIDENT_ISLAND_VISUAL_FRACTION,
            "ratio",
        ),
        EvalCheck::at_most(
            "stream_spawned_visuals_per_frame",
            acc.max_stream_spawned_visuals_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_most(
            "stream_despawned_visuals_per_frame",
            acc.max_stream_despawned_visuals_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_most(
            "entity_count",
            acc.max_entity_count as f32,
            thresholds.max_entity_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "objective_total_count",
            acc.max_objective_total_count as f32,
            thresholds.min_objective_total_count as f32,
            "objectives",
        ),
        EvalCheck::at_least(
            "completed_objective_count",
            acc.max_completed_objective_count as f32,
            thresholds.min_completed_objective_count as f32,
            "objectives",
        ),
    ]);

    assets::append_asset_checks(&mut checks, acc, &thresholds);

    checks.extend([
        EvalCheck::at_most(
            "max_camera_distance",
            acc.max_camera_distance_m,
            thresholds.max_camera_distance_m,
            "m",
        ),
        EvalCheck::at_least(
            "min_camera_surface_clearance",
            acc.min_camera_surface_clearance_m,
            thresholds.min_camera_surface_clearance_m,
            "m",
        ),
        EvalCheck::at_most(
            "max_camera_player_angle",
            acc.max_camera_player_angle_degrees,
            thresholds.max_camera_player_angle_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_camera_step_distance",
            acc.max_camera_step_distance_m,
            thresholds.max_camera_step_distance_m,
            "m",
        ),
        EvalCheck::at_most(
            "max_camera_rotation_delta",
            acc.max_camera_rotation_delta_degrees,
            thresholds.max_camera_rotation_delta_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_camera_orbit_alignment",
            acc.max_camera_orbit_alignment_degrees,
            thresholds.max_camera_orbit_alignment_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_abs_camera_view_yaw",
            acc.max_abs_camera_view_yaw_degrees,
            thresholds.max_abs_camera_view_yaw_degrees,
            "deg",
        ),
        EvalCheck::at_least(
            "max_camera_obstruction_adjustment",
            acc.max_camera_obstruction_adjustment_m,
            thresholds.min_camera_obstruction_adjustment_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_abs_camera_yaw_offset",
            acc.max_abs_camera_yaw_offset_degrees,
            thresholds.min_abs_camera_yaw_degrees,
            "deg",
        ),
        if thresholds.min_camera_pitch_offset_degrees < 0.0 {
            EvalCheck::at_most(
                "min_camera_pitch_offset",
                acc.min_camera_pitch_offset_degrees,
                thresholds.min_camera_pitch_offset_degrees,
                "deg",
            )
        } else {
            EvalCheck::at_least(
                "min_camera_pitch_offset",
                acc.min_camera_pitch_offset_degrees,
                thresholds.min_camera_pitch_offset_degrees,
                "deg",
            )
        },
        if thresholds.max_camera_pitch_offset_degrees > 0.0 {
            EvalCheck::at_least(
                "max_camera_pitch_offset",
                acc.max_camera_pitch_offset_degrees,
                thresholds.max_camera_pitch_offset_degrees,
                "deg",
            )
        } else {
            EvalCheck::at_most(
                "max_camera_pitch_offset",
                acc.max_camera_pitch_offset_degrees,
                thresholds.max_camera_pitch_offset_degrees,
                "deg",
            )
        },
    ]);

    if thresholds.min_camera_obstruction_adjustment_m > 0.0
        || thresholds.min_camera_obstructed_distance_m > 0.0
    {
        checks.extend([
            EvalCheck::at_least(
                "min_camera_obstructed_distance",
                acc.min_camera_obstructed_distance_m.unwrap_or(0.0),
                thresholds.min_camera_obstructed_distance_m,
                "m",
            ),
            EvalCheck::at_most(
                "camera_obstruction_snap_count",
                acc.camera_obstruction_snap_count as f32,
                thresholds.max_camera_obstruction_snap_count as f32,
                "samples",
            ),
        ]);
    }

    control::append_scenario_checks(&mut checks, acc, scenario, derived, &thresholds);
    if matches!(
        scenario.name,
        TERRAIN_RIM_COLLISION_CONTACT | TERRAIN_BODY_COLLISION_CONTACT
    ) {
        checks.retain(|check| !SHORT_CONTACT_SKIPPED_CHECKS.contains(&check.name));
    }
    if scenario.name == ISLAND_HERO_GALLERY {
        checks.retain(|check| !ISLAND_HERO_GALLERY_SKIPPED_CHECKS.contains(&check.name));
    }
    checks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::scenario_named;

    #[test]
    fn artifact_frame_reclassification_excludes_runtime_timing_without_losing_total_timing() {
        let scenario = scenario_named(ISLAND_HERO_GALLERY).expect("gallery scenario");
        let mut accumulator = EvalAccumulator::default();
        accumulator.observe_frame_time_ms(240.0);

        assert!(accumulator.reclassify_latest_runtime_frame_as_eval_artifact());
        assert!(!accumulator.reclassify_latest_runtime_frame_as_eval_artifact());

        let derived = SummaryDerivedMetrics::from_accumulator(&accumulator, scenario);
        assert_eq!(derived.frame_time_stats.sample_count, 1);
        assert_eq!(derived.runtime_frame_time_stats.sample_count, 0);
        assert_eq!(derived.eval_artifact_frame_time_stats.sample_count, 1);
        assert_eq!(derived.eval_artifact_frame_time_stats.max_ms, 240.0);
    }

    #[test]
    fn gallery_rejects_the_old_grounded_foot_gap_instead_of_skipping_it() {
        let scenario = scenario_named(ISLAND_HERO_GALLERY).expect("gallery scenario");
        let mut accumulator = EvalAccumulator {
            max_grounded_visual_foot_gap_m: 0.24,
            grounded_samples: scenario.thresholds.min_grounded_samples,
            ..Default::default()
        };
        let derived = SummaryDerivedMetrics::from_accumulator(&accumulator, scenario);
        let checks = build_checks(&accumulator, scenario, &derived);
        let foot_gap = checks
            .iter()
            .find(|check| check.name == "grounded_visual_foot_gap")
            .expect("gallery foot-gap check");

        assert!(!foot_gap.passed);
        accumulator.max_grounded_visual_foot_gap_m = 0.0;
        let derived = SummaryDerivedMetrics::from_accumulator(&accumulator, scenario);
        let checks = build_checks(&accumulator, scenario, &derived);
        assert!(
            checks
                .iter()
                .find(|check| check.name == "grounded_visual_foot_gap")
                .expect("gallery foot-gap check")
                .passed
        );
    }
}
