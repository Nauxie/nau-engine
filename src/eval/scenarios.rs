mod checkpoints;
mod control_scenarios;
mod input;
mod traversal_scenarios;

use super::thresholds::EvalThresholds;

pub use input::{scripted_camera_input, scripted_input};

pub const BASELINE_ROUTE: &str = "baseline_route";
pub const ISLAND_LAUNCH_TO_LANDING: &str = "island_launch_to_landing";
pub const GROUND_TAXI_CONTROL: &str = "ground_taxi_control";
pub const WORLD_COLLISION_CONTACT: &str = "world_collision_contact";
pub const TERRAIN_RIM_COLLISION_CONTACT: &str = "terrain_rim_collision_contact";
pub const TERRAIN_BODY_COLLISION_CONTACT: &str = "terrain_body_collision_contact";
pub const UPDRAFT_ROUTE: &str = "updraft_route";
pub const CAMERA_MOUSE_CONTROL: &str = "camera_mouse_control";
pub const CAMERA_YAW_STABILITY: &str = "camera_yaw_stability";
pub const CAMERA_TURN_STABILITY: &str = "camera_turn_stability";
pub const CAMERA_STRAFE_STABILITY: &str = "camera_strafe_stability";
pub const AIR_CONTROL_RESPONSE: &str = "air_control_response";
pub const POSE_STATE_COVERAGE: &str = "pose_state_coverage";
pub const LONG_GLIDE_VISIBILITY: &str = "long_glide_visibility";
pub const BRANCH_RECOVERY_ROUTE: &str = "branch_recovery_route";
pub const SCENARIO_NAMES: &[&str] = &[
    BASELINE_ROUTE,
    ISLAND_LAUNCH_TO_LANDING,
    GROUND_TAXI_CONTROL,
    WORLD_COLLISION_CONTACT,
    TERRAIN_RIM_COLLISION_CONTACT,
    TERRAIN_BODY_COLLISION_CONTACT,
    UPDRAFT_ROUTE,
    BRANCH_RECOVERY_ROUTE,
    CAMERA_MOUSE_CONTROL,
    CAMERA_YAW_STABILITY,
    CAMERA_TURN_STABILITY,
    CAMERA_STRAFE_STABILITY,
    AIR_CONTROL_RESPONSE,
    POSE_STATE_COVERAGE,
    LONG_GLIDE_VISIBILITY,
];
pub const APP_ONLY_SCENARIO_NAMES: &[&str] = &[
    WORLD_COLLISION_CONTACT,
    TERRAIN_RIM_COLLISION_CONTACT,
    TERRAIN_BODY_COLLISION_CONTACT,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvalCheckpoint {
    pub frame: u32,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct EvalScenario {
    pub name: &'static str,
    pub fixed_dt: f32,
    pub frame_count: u32,
    pub sample_stride: u32,
    pub target_island_name: Option<&'static str>,
    pub checkpoints: &'static [EvalCheckpoint],
    pub thresholds: EvalThresholds,
}

impl EvalScenario {
    pub fn duration_secs(self) -> f32 {
        self.frame_count as f32 * self.fixed_dt
    }

    pub fn should_sample(self, frame: u32) -> bool {
        frame == 0 || frame >= self.frame_count || frame.is_multiple_of(self.sample_stride)
    }

    pub fn checkpoint_at(self, frame: u32) -> Option<EvalCheckpoint> {
        self.checkpoints
            .iter()
            .copied()
            .find(|checkpoint| checkpoint.frame == frame)
    }
}

pub fn scenario_named(name: &str) -> Option<EvalScenario> {
    match name {
        BASELINE_ROUTE | "baseline" => Some(traversal_scenarios::baseline_route()),
        ISLAND_LAUNCH_TO_LANDING | "island" => {
            Some(traversal_scenarios::island_launch_to_landing())
        }
        GROUND_TAXI_CONTROL | "ground_taxi" | "taxi" => {
            Some(control_scenarios::ground_taxi_control())
        }
        WORLD_COLLISION_CONTACT | "collision_contact" | "asset_collision" => {
            Some(control_scenarios::world_collision_contact())
        }
        TERRAIN_RIM_COLLISION_CONTACT | "terrain_rim_contact" | "rim_collision" => {
            Some(control_scenarios::terrain_rim_collision_contact())
        }
        TERRAIN_BODY_COLLISION_CONTACT
        | "terrain_body_contact"
        | "body_collision"
        | "cliff_collision" => Some(control_scenarios::terrain_body_collision_contact()),
        UPDRAFT_ROUTE | "updraft" => Some(traversal_scenarios::updraft_route()),
        BRANCH_RECOVERY_ROUTE | "branch_recovery" | "recovery_route" => {
            Some(traversal_scenarios::branch_recovery_route())
        }
        CAMERA_MOUSE_CONTROL | "camera_mouse" | "mouse_camera" => {
            Some(control_scenarios::camera_mouse_control())
        }
        CAMERA_YAW_STABILITY | "camera_yaw" | "yaw_stability" => {
            Some(control_scenarios::camera_yaw_stability())
        }
        CAMERA_TURN_STABILITY | "camera_turn" | "turn_stability" => {
            Some(control_scenarios::camera_turn_stability())
        }
        CAMERA_STRAFE_STABILITY | "camera_strafe" | "strafe_stability" => {
            Some(control_scenarios::camera_strafe_stability())
        }
        AIR_CONTROL_RESPONSE | "air_control" | "air_response" => {
            Some(control_scenarios::air_control_response())
        }
        POSE_STATE_COVERAGE | "pose_state" | "pose_coverage" => {
            Some(control_scenarios::pose_state_coverage())
        }
        LONG_GLIDE_VISIBILITY | "long_glide" | "glide_visibility" => {
            Some(traversal_scenarios::long_glide_visibility())
        }
        _ => None,
    }
}
