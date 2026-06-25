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
        frame: 120,
        name: "outbound_glide",
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
        frame: 690,
        name: "branch_landing",
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
