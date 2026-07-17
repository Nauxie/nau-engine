mod checkpoints;
mod control_scenarios;
mod input;
mod traversal_scenarios;

use super::thresholds::EvalThresholds;
use crate::world::{ISLAND_REVIEW_VIEWS_PER_ISLAND, SKY_ROUTE_ISLAND_COUNT};

pub use input::{scripted_camera_input, scripted_input, scripted_playtest_reset_requested};

pub const BASELINE_ROUTE: &str = "baseline_route";
pub const ISLAND_LAUNCH_TO_LANDING: &str = "island_launch_to_landing";
pub const GROUND_TAXI_CONTROL: &str = "ground_taxi_control";
pub const PLAYTEST_RESET: &str = "playtest_reset";
pub const WORLD_COLLISION_CONTACT: &str = "world_collision_contact";
pub const TERRAIN_RIM_COLLISION_CONTACT: &str = "terrain_rim_collision_contact";
pub const TERRAIN_BODY_COLLISION_CONTACT: &str = "terrain_body_collision_contact";
pub const TERRAIN_EDGE_WALKOFF: &str = "terrain_edge_walkoff";
pub const UPDRAFT_ROUTE: &str = "updraft_route";
pub const CAMERA_MOUSE_CONTROL: &str = "camera_mouse_control";
pub const CAMERA_YAW_STABILITY: &str = "camera_yaw_stability";
pub const CAMERA_TURN_STABILITY: &str = "camera_turn_stability";
pub const CAMERA_STRAFE_STABILITY: &str = "camera_strafe_stability";
pub const AIR_CONTROL_RESPONSE: &str = "air_control_response";
pub const POSE_STATE_COVERAGE: &str = "pose_state_coverage";
pub const LONG_GLIDE_VISIBILITY: &str = "long_glide_visibility";
pub const BRANCH_RECOVERY_ROUTE: &str = "branch_recovery_route";
pub const GREAT_SKY_PLATEAU_ROUTE: &str = "great_sky_plateau_route";
pub const GREAT_SKY_PLATEAU_VISTAS: &str = "great_sky_plateau_vistas";
pub const ISLAND_SURFACE_REVIEW: &str = "island_surface_review";
pub const ISLAND_HERO_GALLERY: &str = "island_hero_gallery";
pub const RETURN_DESCENT_ROUTE: &str = "return_descent_route";
pub const PLATEAU_ARRIVAL_CAMERA: &str = "plateau_arrival_camera";
pub const UNDERBRIDGE_UNDER_ROUTE: &str = "underbridge_under_route";
pub const ISLAND_HERO_GALLERY_SETTLE_FRAMES: u32 = 32;
pub const ISLAND_HERO_GALLERY_HOLD_FRAMES: u32 = 4;
pub const ISLAND_HERO_GALLERY_CAPTURE_FRAME_OFFSET: u32 = ISLAND_HERO_GALLERY_SETTLE_FRAMES;
pub const ISLAND_HERO_GALLERY_FRAMES_PER_VIEW: u32 =
    ISLAND_HERO_GALLERY_CAPTURE_FRAME_OFFSET + ISLAND_HERO_GALLERY_HOLD_FRAMES;
pub const ISLAND_HERO_GALLERY_CAPTURE_COUNT: usize =
    SKY_ROUTE_ISLAND_COUNT * ISLAND_REVIEW_VIEWS_PER_ISLAND;
pub const ISLAND_HERO_GALLERY_FRAME_COUNT: u32 =
    ISLAND_HERO_GALLERY_CAPTURE_COUNT as u32 * ISLAND_HERO_GALLERY_FRAMES_PER_VIEW - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IslandHeroGalleryTiming {
    pub settle_frames: u32,
    pub hold_frames: u32,
    pub frames_per_view: u32,
    pub capture_count: usize,
    pub frame_count: u32,
}

impl IslandHeroGalleryTiming {
    pub const fn capture_frame(self, capture_index: usize) -> Option<u32> {
        if capture_index >= self.capture_count {
            return None;
        }
        Some(
            capture_index as u32 * self.frames_per_view + (self.frames_per_view - self.hold_frames),
        )
    }

    pub const fn expected_grounded_samples(self) -> u32 {
        (self.capture_count / ISLAND_REVIEW_VIEWS_PER_ISLAND) as u32 * self.frames_per_view
    }
}

type ScenarioFactory = fn() -> EvalScenario;

#[derive(Clone, Copy)]
struct ScenarioRegistration {
    name: &'static str,
    aliases: &'static [&'static str],
    app_only: bool,
    factory: ScenarioFactory,
}

const SCENARIO_COUNT: usize = 24;
const APP_ONLY_SCENARIO_COUNT: usize = 9;
const SCENARIO_REGISTRY: [ScenarioRegistration; SCENARIO_COUNT] = [
    ScenarioRegistration {
        name: BASELINE_ROUTE,
        aliases: &["baseline"],
        app_only: false,
        factory: traversal_scenarios::baseline_route,
    },
    ScenarioRegistration {
        name: ISLAND_LAUNCH_TO_LANDING,
        aliases: &["island"],
        app_only: false,
        factory: traversal_scenarios::island_launch_to_landing,
    },
    ScenarioRegistration {
        name: GROUND_TAXI_CONTROL,
        aliases: &["ground_taxi", "taxi"],
        app_only: false,
        factory: control_scenarios::ground_taxi_control,
    },
    ScenarioRegistration {
        name: PLAYTEST_RESET,
        aliases: &["reset", "central_reset"],
        app_only: true,
        factory: control_scenarios::playtest_reset,
    },
    ScenarioRegistration {
        name: WORLD_COLLISION_CONTACT,
        aliases: &["collision_contact", "asset_collision"],
        app_only: true,
        factory: control_scenarios::world_collision_contact,
    },
    ScenarioRegistration {
        name: TERRAIN_RIM_COLLISION_CONTACT,
        aliases: &["terrain_rim_contact", "rim_collision"],
        app_only: true,
        factory: control_scenarios::terrain_rim_collision_contact,
    },
    ScenarioRegistration {
        name: TERRAIN_BODY_COLLISION_CONTACT,
        aliases: &["terrain_body_contact", "body_collision", "cliff_collision"],
        app_only: true,
        factory: control_scenarios::terrain_body_collision_contact,
    },
    ScenarioRegistration {
        name: TERRAIN_EDGE_WALKOFF,
        aliases: &["edge_walkoff", "edge_collision_truth"],
        app_only: true,
        factory: control_scenarios::terrain_edge_walkoff,
    },
    ScenarioRegistration {
        name: UPDRAFT_ROUTE,
        aliases: &["updraft"],
        app_only: false,
        factory: traversal_scenarios::updraft_route,
    },
    ScenarioRegistration {
        name: BRANCH_RECOVERY_ROUTE,
        aliases: &["branch_recovery", "recovery_route"],
        app_only: false,
        factory: traversal_scenarios::branch_recovery_route,
    },
    ScenarioRegistration {
        name: CAMERA_MOUSE_CONTROL,
        aliases: &["camera_mouse", "mouse_camera"],
        app_only: false,
        factory: control_scenarios::camera_mouse_control,
    },
    ScenarioRegistration {
        name: CAMERA_YAW_STABILITY,
        aliases: &["camera_yaw", "yaw_stability"],
        app_only: false,
        factory: control_scenarios::camera_yaw_stability,
    },
    ScenarioRegistration {
        name: CAMERA_TURN_STABILITY,
        aliases: &["camera_turn", "turn_stability"],
        app_only: false,
        factory: control_scenarios::camera_turn_stability,
    },
    ScenarioRegistration {
        name: CAMERA_STRAFE_STABILITY,
        aliases: &["camera_strafe", "strafe_stability"],
        app_only: false,
        factory: control_scenarios::camera_strafe_stability,
    },
    ScenarioRegistration {
        name: AIR_CONTROL_RESPONSE,
        aliases: &["air_control", "air_response"],
        app_only: false,
        factory: control_scenarios::air_control_response,
    },
    ScenarioRegistration {
        name: POSE_STATE_COVERAGE,
        aliases: &["pose_state", "pose_coverage"],
        app_only: false,
        factory: control_scenarios::pose_state_coverage,
    },
    ScenarioRegistration {
        name: LONG_GLIDE_VISIBILITY,
        aliases: &["long_glide", "glide_visibility"],
        app_only: false,
        factory: traversal_scenarios::long_glide_visibility,
    },
    ScenarioRegistration {
        name: GREAT_SKY_PLATEAU_ROUTE,
        aliases: &["great_sky_plateau", "plateau_route"],
        app_only: false,
        factory: traversal_scenarios::great_sky_plateau_route,
    },
    ScenarioRegistration {
        name: GREAT_SKY_PLATEAU_VISTAS,
        aliases: &["plateau_vistas", "plateau_showcase"],
        app_only: true,
        factory: traversal_scenarios::great_sky_plateau_vistas,
    },
    ScenarioRegistration {
        name: ISLAND_SURFACE_REVIEW,
        aliases: &["surface_review", "island_details"],
        app_only: true,
        factory: traversal_scenarios::island_surface_review,
    },
    ScenarioRegistration {
        name: ISLAND_HERO_GALLERY,
        aliases: &["hero_gallery", "all_islands"],
        app_only: true,
        factory: traversal_scenarios::island_hero_gallery,
    },
    ScenarioRegistration {
        name: RETURN_DESCENT_ROUTE,
        aliases: &["return_descent", "descent_route", "long_descent"],
        app_only: true,
        factory: traversal_scenarios::return_descent_route,
    },
    ScenarioRegistration {
        name: PLATEAU_ARRIVAL_CAMERA,
        aliases: &["plateau_camera", "plateau_arrival"],
        app_only: false,
        factory: traversal_scenarios::plateau_arrival_camera,
    },
    ScenarioRegistration {
        name: UNDERBRIDGE_UNDER_ROUTE,
        aliases: &["underbridge_route", "under_route"],
        app_only: false,
        factory: traversal_scenarios::underbridge_under_route,
    },
];

const fn scenario_names() -> [&'static str; SCENARIO_COUNT] {
    let mut names = [""; SCENARIO_COUNT];
    let mut index = 0;
    while index < SCENARIO_COUNT {
        names[index] = SCENARIO_REGISTRY[index].name;
        index += 1;
    }
    names
}

const fn app_only_scenario_names() -> [&'static str; APP_ONLY_SCENARIO_COUNT] {
    let mut names = [""; APP_ONLY_SCENARIO_COUNT];
    let mut registry_index = 0;
    let mut output_index = 0;
    while registry_index < SCENARIO_COUNT {
        if SCENARIO_REGISTRY[registry_index].app_only {
            if output_index >= APP_ONLY_SCENARIO_COUNT {
                panic!("APP_ONLY_SCENARIO_COUNT is too small");
            }
            names[output_index] = SCENARIO_REGISTRY[registry_index].name;
            output_index += 1;
        }
        registry_index += 1;
    }
    if output_index != APP_ONLY_SCENARIO_COUNT {
        panic!("APP_ONLY_SCENARIO_COUNT is too large");
    }
    names
}

const SCENARIO_NAMES_STORAGE: [&str; SCENARIO_COUNT] = scenario_names();
const APP_ONLY_SCENARIO_NAMES_STORAGE: [&str; APP_ONLY_SCENARIO_COUNT] = app_only_scenario_names();
pub const SCENARIO_NAMES: &[&str] = &SCENARIO_NAMES_STORAGE;
pub const APP_ONLY_SCENARIO_NAMES: &[&str] = &APP_ONLY_SCENARIO_NAMES_STORAGE;

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

    pub fn expected_sample_count(self) -> u32 {
        let interval_samples = self.frame_count / self.sample_stride + 1;
        interval_samples + u32::from(!self.frame_count.is_multiple_of(self.sample_stride))
    }

    pub fn checkpoint_at(self, frame: u32) -> Option<EvalCheckpoint> {
        self.checkpoints
            .iter()
            .copied()
            .find(|checkpoint| checkpoint.frame == frame)
    }

    pub fn island_hero_gallery_timing(self) -> Option<IslandHeroGalleryTiming> {
        if self.name != ISLAND_HERO_GALLERY {
            return None;
        }
        Some(IslandHeroGalleryTiming {
            settle_frames: ISLAND_HERO_GALLERY_SETTLE_FRAMES,
            hold_frames: ISLAND_HERO_GALLERY_HOLD_FRAMES,
            frames_per_view: ISLAND_HERO_GALLERY_FRAMES_PER_VIEW,
            capture_count: ISLAND_HERO_GALLERY_CAPTURE_COUNT,
            frame_count: ISLAND_HERO_GALLERY_FRAME_COUNT,
        })
    }
}

pub fn scenario_named(name: &str) -> Option<EvalScenario> {
    SCENARIO_REGISTRY
        .iter()
        .find(|registration| registration.name == name || registration.aliases.contains(&name))
        .map(|registration| {
            let mut scenario = (registration.factory)();
            scenario.thresholds.min_samples = scenario.expected_sample_count();
            if let Some(timing) = scenario.island_hero_gallery_timing() {
                scenario.thresholds.min_grounded_samples = timing.expected_grounded_samples();
            }
            scenario
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn scenario_registry_is_complete_unique_and_resolvable() {
        let mut lookup_names = BTreeSet::new();
        let canonical_names = SCENARIO_REGISTRY
            .iter()
            .map(|registration| registration.name)
            .collect::<Vec<_>>();
        let app_only_names = SCENARIO_REGISTRY
            .iter()
            .filter(|registration| registration.app_only)
            .map(|registration| registration.name)
            .collect::<Vec<_>>();

        assert_eq!(canonical_names.as_slice(), SCENARIO_NAMES);
        assert_eq!(app_only_names.as_slice(), APP_ONLY_SCENARIO_NAMES);
        for registration in SCENARIO_REGISTRY {
            assert!(
                lookup_names.insert(registration.name),
                "duplicate scenario name: {}",
                registration.name
            );
            assert_eq!(
                scenario_named(registration.name)
                    .expect("registered scenario resolves")
                    .name,
                registration.name
            );
            for alias in registration.aliases {
                assert!(
                    lookup_names.insert(alias),
                    "duplicate scenario alias: {alias}"
                );
                assert_eq!(
                    scenario_named(alias)
                        .expect("registered scenario alias resolves")
                        .name,
                    registration.name
                );
            }
        }

        for name in SCENARIO_NAMES {
            let scenario = scenario_named(name).expect("registered scenario resolves");
            assert_eq!(
                scenario.thresholds.min_samples,
                scenario.expected_sample_count(),
                "{name} should require its exact deterministic sample count"
            );
        }

        let air_control = scenario_named(AIR_CONTROL_RESPONSE).expect("air control scenario");
        assert!(air_control.sample_stride > 1);
        assert!(!1_u32.is_multiple_of(air_control.sample_stride));
        assert!(!air_control.should_sample(1));
    }

    #[test]
    fn return_descent_route_targets_authored_handrail_regression() {
        let scenario = scenario_named(RETURN_DESCENT_ROUTE).expect("return descent scenario");
        let alias = scenario_named("long_descent").expect("return descent alias");

        assert_eq!(alias.name, RETURN_DESCENT_ROUTE);
        assert!(APP_ONLY_SCENARIO_NAMES.contains(&RETURN_DESCENT_ROUTE));
        assert_eq!(scenario.target_island_name, Some("upper crown"));
        assert_eq!(scenario.checkpoints[0].name, "return_descent_handrail");
        assert!(
            scenario
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.name == "upper_crown_descent_view")
        );
        assert!(scenario.frame_count >= 1_020);
        assert!(scenario.thresholds.min_horizontal_distance_m >= 260.0);
        assert!(scenario.thresholds.min_gliding_samples >= 100);
        assert!(scenario.thresholds.min_objective_total_count >= 11);
        assert_eq!(scenario.thresholds.max_camera_obstruction_snap_count, 0);
        assert!(scripted_input(scenario, 1).launch);
        assert!(scripted_input(scenario, 60).glide);
        assert!(scripted_input(scenario, 120).forward);
        assert!(!scripted_input(scenario, 120).left);
        assert!(scripted_input(scenario, 360).left);
        assert!(scripted_input(scenario, 720).right);
        assert!(scripted_input(scenario, 780).right);
    }

    #[test]
    fn plateau_arrival_camera_targets_close_geometry_regression() {
        let scenario = scenario_named(PLATEAU_ARRIVAL_CAMERA).expect("plateau camera scenario");
        let alias = scenario_named("plateau_camera").expect("plateau camera alias");

        assert_eq!(alias.name, PLATEAU_ARRIVAL_CAMERA);
        assert_eq!(scenario.target_island_name, Some("great sky plateau"));
        assert_eq!(
            scenario.checkpoints[1].name,
            "plateau_spire_camera_obstruction"
        );
        assert_eq!(scenario.sample_stride, 1);
        assert!(scenario.frame_count >= 420);
        assert!(scenario.thresholds.min_samples >= 360);
        assert!(scenario.thresholds.min_camera_obstruction_adjustment_m >= 4.0);
        assert!(scenario.thresholds.min_camera_obstructed_distance_m >= 5.0);
        assert!(scenario.thresholds.max_camera_step_distance_m <= 0.75);
        assert!(scenario.thresholds.max_camera_rotation_delta_degrees <= 1.5);
        assert!(scenario.thresholds.max_camera_player_angle_degrees <= 2.0);
        assert_eq!(scenario.thresholds.max_camera_obstruction_snap_count, 0);
        assert!(scenario.thresholds.min_abs_camera_yaw_degrees >= 10.0);
        assert!(scripted_input(scenario, 30).forward);
        assert!(scripted_input(scenario, 130).right);
        assert!(scripted_input(scenario, 230).left);
        assert!(scripted_input(scenario, 320).right);
        assert!(scripted_camera_input(scenario, 20).mouse_delta.x > 0.0);
        assert!(scripted_camera_input(scenario, 60).mouse_delta.x < 0.0);
        assert!(scripted_camera_input(scenario, 110).mouse_delta.x > 0.0);
        assert!(scripted_camera_input(scenario, 155).mouse_delta.x < 0.0);
        assert!(scripted_camera_input(scenario, 205).mouse_delta.x > 0.0);
        assert!(scripted_camera_input(scenario, 260).mouse_delta.x < 0.0);
        assert_eq!(
            scripted_camera_input(scenario, 360).mouse_delta,
            bevy::prelude::Vec2::ZERO
        );
        assert!(!scripted_input(scenario, 1).launch);
    }

    #[test]
    fn terrain_edge_walkoff_targets_invisible_barrier_regression() {
        let scenario = scenario_named(TERRAIN_EDGE_WALKOFF).expect("edge walkoff scenario");
        let alias = scenario_named("edge_collision_truth").expect("edge collision alias");

        assert_eq!(alias.name, TERRAIN_EDGE_WALKOFF);
        assert!(APP_ONLY_SCENARIO_NAMES.contains(&TERRAIN_EDGE_WALKOFF));
        assert!(scripted_input(scenario, 30).right);
        assert!(!scripted_input(scenario, 30).launch);
        assert_eq!(scenario.thresholds.max_camera_obstruction_snap_count, 0);
        assert!(scenario.thresholds.min_grounded_samples >= 12);
    }

    #[test]
    fn island_hero_gallery_timing_is_single_source_of_capture_truth() {
        let scenario = scenario_named(ISLAND_HERO_GALLERY).expect("gallery scenario");
        let timing = scenario
            .island_hero_gallery_timing()
            .expect("gallery timing");

        assert_eq!(timing.settle_frames, 32);
        assert_eq!(timing.hold_frames, 4);
        assert_eq!(timing.frames_per_view, 36);
        assert_eq!(timing.capture_count, 123);
        assert_eq!(timing.frame_count, scenario.frame_count);
        assert_eq!(timing.capture_frame(0), Some(32));
        assert_eq!(timing.capture_frame(1), Some(68));
        assert_eq!(timing.capture_frame(122), Some(4_424));
        assert_eq!(timing.capture_frame(123), None);
        assert_eq!(
            scenario.thresholds.min_grounded_samples,
            timing.expected_grounded_samples()
        );
        assert_eq!(timing.expected_grounded_samples(), 1_476);
    }
}
