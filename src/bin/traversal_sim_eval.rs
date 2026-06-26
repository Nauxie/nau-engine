#![recursion_limit = "512"]

#[path = "traversal_sim_eval/cli.rs"]
mod cli;
#[path = "traversal_sim_eval/metrics.rs"]
mod metrics;
#[path = "traversal_sim_eval/sample.rs"]
mod sample;
#[path = "traversal_sim_eval/simulation.rs"]
mod simulation;
#[path = "traversal_sim_eval/state.rs"]
mod state;

use cli::{SimOptions, run_and_write, usage};

pub(crate) use sample::{SimSample, round4, round4_f64, vec3_json};
pub(crate) use simulation::run_simulation;

const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 2.2;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;
const AIR_CONTROL_RESPONSE_THRESHOLD_MPS: f32 = 4.0;
const AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS: f32 = 18.0;
const AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS: f32 = 10.0;
const AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS: f32 = 10.0;
const AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS: f32 = 20.0;
const AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES: f32 = 8.0;
const AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES: f32 = 22.0;
const AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES: f32 = 36.0;
const AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES: f32 = 36.0;
const AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS: f32 = 4.0;
const AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES: f32 = 8.0;
const AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES: f32 = 12.0;
const AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES: f32 = 0.01;
const AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES: f32 = 2.0;
const AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES: f32 = 2.0;
const AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS: f32 = 0.20;
const AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS: f32 = 12.0;
const AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS: f32 = 12.0;
const AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS: f32 = 14.0;
const AIR_CONTROL_MIN_POSE_TORSO_PITCH_DEGREES: f32 = 45.0;
const AIR_CONTROL_MIN_POSE_ARM_SPREAD_DEGREES: f32 = 100.0;
const AIR_CONTROL_MIN_POSE_LEG_TUCK_DEGREES: f32 = 35.0;
const AIR_CONTROL_MIN_POSE_LATERAL_LEAN_DEGREES: f32 = 8.0;
const AIR_CONTROL_MIN_POSE_WING_AIRFLOW_STRENGTH: f32 = 0.25;
const LANDING_MIN_POSE_CROUCH_M: f32 = 0.05;
const AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES: f32 = 8.0;
const CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS: f32 = 8.0;
const CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES: f32 = 2.0;
const MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES: f32 = 2.0;
const GROUND_VISUAL_FOOT_GAP_M: f32 = 0.0;

fn main() {
    let options = match SimOptions::from_env() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };

    if let Err(error) = run_and_write(options) {
        eprintln!("traversal simulation eval failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
#[path = "traversal_sim_eval/tests.rs"]
mod tests;
