use super::EvalCheckpoint;

pub(super) const BASELINE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 90,
        name: "launch_clear",
    },
    EvalCheckpoint {
        frame: 260,
        name: "glide_midroute",
    },
];
pub(super) const ISLAND_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 35,
        name: "launch_terrain_read",
    },
    EvalCheckpoint {
        frame: 85,
        name: "launch_updraft_entry",
    },
    EvalCheckpoint {
        frame: 120,
        name: "outbound_glide",
    },
    EvalCheckpoint {
        frame: 220,
        name: "midroute_lift_view",
    },
    EvalCheckpoint {
        frame: 320,
        name: "landing_approach",
    },
];
pub(super) const GROUND_TAXI_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 60,
        name: "ground_turn",
    },
    EvalCheckpoint {
        frame: 150,
        name: "reverse_check",
    },
];
pub(super) const WORLD_COLLISION_CONTACT_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 80,
        name: "approach_tree",
    },
    EvalCheckpoint {
        frame: 150,
        name: "blocked_by_tree",
    },
];
pub(super) const TERRAIN_RIM_COLLISION_CONTACT_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 90,
        name: "approach_rim",
    },
    EvalCheckpoint {
        frame: 180,
        name: "blocked_by_rim",
    },
];
pub(super) const TERRAIN_BODY_COLLISION_CONTACT_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 120,
        name: "approach_cliff_body",
    },
    EvalCheckpoint {
        frame: 260,
        name: "blocked_by_cliff_body",
    },
];
pub(super) const UPDRAFT_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 150,
        name: "updraft_entry",
    },
    EvalCheckpoint {
        frame: 280,
        name: "high_glide",
    },
];
pub(super) const BRANCH_RECOVERY_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 180,
        name: "branch_choice",
    },
    EvalCheckpoint {
        frame: 500,
        name: "recovery_approach",
    },
    EvalCheckpoint {
        frame: 580,
        name: "branch_landing_approach",
    },
];
pub(super) const CAMERA_MOUSE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 5,
        name: "launch_obstruction",
    },
    EvalCheckpoint {
        frame: 50,
        name: "yaw_check",
    },
    EvalCheckpoint {
        frame: 120,
        name: "pitch_check",
    },
    EvalCheckpoint {
        frame: 180,
        name: "settled_view",
    },
];
pub(super) const CAMERA_YAW_STABILITY_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 30,
        name: "small_yaw_input",
    },
    EvalCheckpoint {
        frame: 180,
        name: "yaw_settle",
    },
    EvalCheckpoint {
        frame: 260,
        name: "drift_check",
    },
];
pub(super) const CAMERA_TURN_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 90,
        name: "first_turn",
    },
    EvalCheckpoint {
        frame: 180,
        name: "counter_turn",
    },
    EvalCheckpoint {
        frame: 300,
        name: "air_brake",
    },
];
pub(super) const CAMERA_STRAFE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 60,
        name: "right_strafe",
    },
    EvalCheckpoint {
        frame: 150,
        name: "left_strafe",
    },
    EvalCheckpoint {
        frame: 230,
        name: "settled_strafe",
    },
];
pub(super) const AIR_CONTROL_RESPONSE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 90,
        name: "diagonal_air_steer",
    },
    EvalCheckpoint {
        frame: 165,
        name: "right_air_steer",
    },
    EvalCheckpoint {
        frame: 245,
        name: "left_air_recovery",
    },
    EvalCheckpoint {
        frame: 335,
        name: "air_brake_recovery",
    },
];
pub(super) const POSE_STATE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 55,
        name: "grounded_walk_coast",
    },
    EvalCheckpoint {
        frame: 210,
        name: "launch_to_fall",
    },
    EvalCheckpoint {
        frame: 290,
        name: "glide_recovery",
    },
];
pub(super) const LONG_GLIDE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 180,
        name: "far_route_entry",
    },
    EvalCheckpoint {
        frame: 420,
        name: "archipelago_midroute",
    },
    EvalCheckpoint {
        frame: 640,
        name: "distant_islands",
    },
];
pub(super) const GREAT_SKY_PLATEAU_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 360,
        name: "upper_thermal_chain",
    },
    EvalCheckpoint {
        frame: 780,
        name: "stratos_to_summit_climb",
    },
    EvalCheckpoint {
        frame: 1260,
        name: "high_archipelago_crossing",
    },
    EvalCheckpoint {
        frame: 1740,
        name: "plateau_approach",
    },
];

pub(super) const UNDERBRIDGE_UNDER_ROUTE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 105,
        name: "underbridge_low_setup",
    },
    EvalCheckpoint {
        frame: 170,
        name: "under_route_camera_obstruction",
    },
    EvalCheckpoint {
        frame: 250,
        name: "underbridge_lift_recovery",
    },
];
