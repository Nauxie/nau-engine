use crate::environment::GAMEPLAY_LIFT_ROUTE;
use crate::movement::{FlightMode, FlightState};
use bevy::prelude::{Resource, Vec2, Vec3};

use super::{
    GROUND_CONTACT_EPSILON, GROUND_CONTACT_HORIZONTAL_DAMPING, GroundSurface, IslandPlateauRegion,
    IslandUnderRouteSegment, LodBand, PLAYER_STANDING_OFFSET, RouteObjective, START_FLOOR_Y,
    SkyIsland, StreamChunkCoord, StreamingLodStats,
};

pub const SKY_ROUTE_ISLAND_COUNT: usize = 41;
pub const PLAYTEST_RESET_ISLAND_NAME: &str = "great sky plateau";
const UNDER_ROUTE_GROUND_CLEARANCE_PADDING_M: f32 = 10.0;
const UNDER_ROUTE_TOP_SURFACE_CLEARANCE_FRACTION: f32 = 0.18;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstExpeditionBeatKind {
    Launch,
    FirstGlide,
    LowRecovery,
    UnderRoutePass,
    LakeWaterfallLandmark,
    HighClimb,
    PlateauApproach,
    PlateauArrival,
}

impl FirstExpeditionBeatKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::FirstGlide => "first_glide",
            Self::LowRecovery => "low_recovery",
            Self::UnderRoutePass => "under_route_pass",
            Self::LakeWaterfallLandmark => "lake_waterfall_landmark",
            Self::HighClimb => "high_climb",
            Self::PlateauApproach => "plateau_approach",
            Self::PlateauArrival => "plateau_arrival",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum FirstExpeditionAltitudeBand {
    Low,
    Mid,
    High,
    Plateau,
}

impl FirstExpeditionAltitudeBand {
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Mid => "mid",
            Self::High => "high",
            Self::Plateau => "plateau",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstExpeditionTraversalMode {
    GroundLaunch,
    OpenGlide,
    RecoveryLift,
    UnderIslandGlide,
    LandmarkGlide,
    SustainedClimb,
    ApproachGlide,
    ArrivalLanding,
}

impl FirstExpeditionTraversalMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::GroundLaunch => "ground_launch",
            Self::OpenGlide => "open_glide",
            Self::RecoveryLift => "recovery_lift",
            Self::UnderIslandGlide => "under_island_glide",
            Self::LandmarkGlide => "landmark_glide",
            Self::SustainedClimb => "sustained_climb",
            Self::ApproachGlide => "approach_glide",
            Self::ArrivalLanding => "arrival_landing",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstExpeditionDetourKind {
    LowAltitudeRecoveryLoop,
    HighRiskUpperPath,
}

impl FirstExpeditionDetourKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::LowAltitudeRecoveryLoop => "low_altitude_recovery_loop",
            Self::HighRiskUpperPath => "high_risk_upper_path",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstExpeditionDetourRisk {
    Recovery,
    HighRiskHighReward,
}

impl FirstExpeditionDetourRisk {
    pub fn label(self) -> &'static str {
        match self {
            Self::Recovery => "recovery",
            Self::HighRiskHighReward => "high_risk_high_reward",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstExpeditionNavigationLandmarkKind {
    LaunchBeacon,
    RouteCairn,
    RecoveryUpdraft,
    CaveMouth,
    WaterfallLakeVista,
    GardenRing,
    PlateauRim,
    PlateauArrivalRuin,
    DetourMarker,
}

impl FirstExpeditionNavigationLandmarkKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::LaunchBeacon => "launch_beacon",
            Self::RouteCairn => "route_cairn",
            Self::RecoveryUpdraft => "recovery_updraft",
            Self::CaveMouth => "cave_mouth",
            Self::WaterfallLakeVista => "waterfall_lake_vista",
            Self::GardenRing => "garden_ring",
            Self::PlateauRim => "plateau_rim",
            Self::PlateauArrivalRuin => "plateau_arrival_ruin",
            Self::DetourMarker => "detour_marker",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstExpeditionNavigationContext {
    RequiredBeat(FirstExpeditionBeatKind),
    OptionalDetour(FirstExpeditionDetourKind),
}

impl FirstExpeditionNavigationContext {
    pub fn label(self) -> &'static str {
        match self {
            Self::RequiredBeat(_) => "required_beat",
            Self::OptionalDetour(_) => "optional_detour",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstExpeditionRecoveryAffordance {
    LaunchLift {
        lift_name: &'static str,
    },
    LandingSurface {
        island_name: &'static str,
    },
    RecoveryUpdraft {
        lift_name: &'static str,
    },
    UnderRouteRecovery {
        lift_name: &'static str,
    },
    PlateauReset {
        island_name: &'static str,
        lift_name: &'static str,
    },
}

impl FirstExpeditionRecoveryAffordance {
    pub fn label(self) -> &'static str {
        match self {
            Self::LaunchLift { .. } => "launch_lift",
            Self::LandingSurface { .. } => "landing_surface",
            Self::RecoveryUpdraft { .. } => "recovery_updraft",
            Self::UnderRouteRecovery { .. } => "under_route_recovery",
            Self::PlateauReset { .. } => "plateau_reset",
        }
    }

    pub fn lift_name(self) -> Option<&'static str> {
        match self {
            Self::LaunchLift { lift_name }
            | Self::RecoveryUpdraft { lift_name }
            | Self::UnderRouteRecovery { lift_name }
            | Self::PlateauReset { lift_name, .. } => Some(lift_name),
            Self::LandingSurface { .. } => None,
        }
    }

    pub fn island_name(self) -> Option<&'static str> {
        match self {
            Self::LandingSurface { island_name } | Self::PlateauReset { island_name, .. } => {
                Some(island_name)
            }
            Self::LaunchLift { .. }
            | Self::RecoveryUpdraft { .. }
            | Self::UnderRouteRecovery { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FirstExpeditionRouteBeat {
    pub label: &'static str,
    pub kind: FirstExpeditionBeatKind,
    pub anchor_island_name: &'static str,
    pub position: Vec3,
    pub altitude_band: FirstExpeditionAltitudeBand,
    pub traversal_mode: FirstExpeditionTraversalMode,
    pub landmark_anchor: &'static str,
    pub recovery_affordance: FirstExpeditionRecoveryAffordance,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FirstExpeditionOptionalDetour {
    pub label: &'static str,
    pub kind: FirstExpeditionDetourKind,
    pub risk: FirstExpeditionDetourRisk,
    pub entry_beat_kind: FirstExpeditionBeatKind,
    pub reconnect_beat_kind: FirstExpeditionBeatKind,
    pub altitude_band: FirstExpeditionAltitudeBand,
    pub island_names: &'static [&'static str],
    pub lift_names: &'static [&'static str],
    pub reconnect_lift_name: &'static str,
    pub landmark_anchor: &'static str,
    pub route_positions: Vec<Vec3>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FirstExpeditionNavigationLandmark {
    pub label: &'static str,
    pub kind: FirstExpeditionNavigationLandmarkKind,
    pub context: FirstExpeditionNavigationContext,
    pub island_name: &'static str,
    pub visual_anchor: &'static str,
    pub position: Vec3,
    pub altitude_band: FirstExpeditionAltitudeBand,
    pub readable_radius_m: f32,
    pub required_route: bool,
}

#[derive(Clone, Copy, Debug)]
struct FirstExpeditionRouteBeatSpec {
    label: &'static str,
    kind: FirstExpeditionBeatKind,
    anchor_island_name: &'static str,
    position: FirstExpeditionBeatPosition,
    altitude_band: FirstExpeditionAltitudeBand,
    traversal_mode: FirstExpeditionTraversalMode,
    landmark_anchor: &'static str,
    recovery_affordance: FirstExpeditionRecoveryAffordance,
}

#[derive(Clone, Copy, Debug)]
enum FirstExpeditionBeatPosition {
    IslandSurface,
    LiftNode(&'static str),
    UnderRouteMidpoint,
    PlateauRegion(IslandPlateauRegion),
}

#[derive(Clone, Copy, Debug)]
struct FirstExpeditionOptionalDetourSpec {
    label: &'static str,
    kind: FirstExpeditionDetourKind,
    risk: FirstExpeditionDetourRisk,
    entry_beat_kind: FirstExpeditionBeatKind,
    reconnect_beat_kind: FirstExpeditionBeatKind,
    altitude_band: FirstExpeditionAltitudeBand,
    island_names: &'static [&'static str],
    lift_names: &'static [&'static str],
    reconnect_lift_name: &'static str,
    landmark_anchor: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct FirstExpeditionNavigationLandmarkSpec {
    label: &'static str,
    kind: FirstExpeditionNavigationLandmarkKind,
    context: FirstExpeditionNavigationContext,
    island_name: &'static str,
    visual_anchor: &'static str,
    position: FirstExpeditionNavigationLandmarkPosition,
    altitude_band: FirstExpeditionAltitudeBand,
    readable_radius_m: f32,
    required_route: bool,
}

#[derive(Clone, Copy, Debug)]
enum FirstExpeditionNavigationLandmarkPosition {
    IslandSurfaceOffset {
        normalized_offset: [f32; 2],
        height_offset_m: f32,
    },
    LiftNode(&'static str),
    UnderRouteEntry,
}

const LOW_ALTITUDE_RECOVERY_LOOP_ISLANDS: [&str; 3] =
    ["low reef", "lowwind shelf", "western refuge"];
const LOW_ALTITUDE_RECOVERY_LOOP_LIFTS: [&str; 3] = [
    "low reef updraft",
    "western catch updraft",
    "distant recovery updraft",
];
const HIGH_RISK_UPPER_PATH_ISLANDS: [&str; 3] =
    ["bluevault basin", "outer switchback", "far horizon perch"];
const HIGH_RISK_UPPER_PATH_LIFTS: [&str; 3] = [
    "bluevault shoulder recovery updraft",
    "plateau west rim recovery updraft",
    "great sky plateau updraft",
];

const FIRST_EXPEDITION_ROUTE_BEAT_SPECS: [FirstExpeditionRouteBeatSpec; 8] = [
    FirstExpeditionRouteBeatSpec {
        label: "launch mesa takeoff",
        kind: FirstExpeditionBeatKind::Launch,
        anchor_island_name: "launch mesa",
        position: FirstExpeditionBeatPosition::IslandSurface,
        altitude_band: FirstExpeditionAltitudeBand::Low,
        traversal_mode: FirstExpeditionTraversalMode::GroundLaunch,
        landmark_anchor: "launch beacon",
        recovery_affordance: FirstExpeditionRecoveryAffordance::LaunchLift {
            lift_name: "launch terrace updraft",
        },
    },
    FirstExpeditionRouteBeatSpec {
        label: "midpoint shelf first glide",
        kind: FirstExpeditionBeatKind::FirstGlide,
        anchor_island_name: "midpoint shelf",
        position: FirstExpeditionBeatPosition::IslandSurface,
        altitude_band: FirstExpeditionAltitudeBand::Low,
        traversal_mode: FirstExpeditionTraversalMode::OpenGlide,
        landmark_anchor: "route cairn line",
        recovery_affordance: FirstExpeditionRecoveryAffordance::LandingSurface {
            island_name: "landing garden",
        },
    },
    FirstExpeditionRouteBeatSpec {
        label: "low reef recovery thermal",
        kind: FirstExpeditionBeatKind::LowRecovery,
        anchor_island_name: "low reef",
        position: FirstExpeditionBeatPosition::LiftNode("low reef updraft"),
        altitude_band: FirstExpeditionAltitudeBand::Low,
        traversal_mode: FirstExpeditionTraversalMode::RecoveryLift,
        landmark_anchor: "low reef wind ribbons",
        recovery_affordance: FirstExpeditionRecoveryAffordance::RecoveryUpdraft {
            lift_name: "low reef updraft",
        },
    },
    FirstExpeditionRouteBeatSpec {
        label: "underbridge cay under-route",
        kind: FirstExpeditionBeatKind::UnderRoutePass,
        anchor_island_name: "underbridge cay",
        position: FirstExpeditionBeatPosition::UnderRouteMidpoint,
        altitude_band: FirstExpeditionAltitudeBand::Low,
        traversal_mode: FirstExpeditionTraversalMode::UnderIslandGlide,
        landmark_anchor: "under-route cave mouth arch",
        recovery_affordance: FirstExpeditionRecoveryAffordance::UnderRouteRecovery {
            lift_name: "underbridge cay updraft",
        },
    },
    FirstExpeditionRouteBeatSpec {
        label: "cloudfall lake and waterfall sightline",
        kind: FirstExpeditionBeatKind::LakeWaterfallLandmark,
        anchor_island_name: "cloudfall meadow",
        position: FirstExpeditionBeatPosition::IslandSurface,
        altitude_band: FirstExpeditionAltitudeBand::Mid,
        traversal_mode: FirstExpeditionTraversalMode::LandmarkGlide,
        landmark_anchor: "route edge waterfall and bluevault basin lake",
        recovery_affordance: FirstExpeditionRecoveryAffordance::RecoveryUpdraft {
            lift_name: "cloudfall meadow updraft",
        },
    },
    FirstExpeditionRouteBeatSpec {
        label: "sunspire garden high climb",
        kind: FirstExpeditionBeatKind::HighClimb,
        anchor_island_name: "sunspire garden",
        position: FirstExpeditionBeatPosition::LiftNode("sunspire garden updraft"),
        altitude_band: FirstExpeditionAltitudeBand::High,
        traversal_mode: FirstExpeditionTraversalMode::SustainedClimb,
        landmark_anchor: "sunspire garden ring",
        recovery_affordance: FirstExpeditionRecoveryAffordance::RecoveryUpdraft {
            lift_name: "sunspire garden updraft",
        },
    },
    FirstExpeditionRouteBeatSpec {
        label: "cloudbreak stair plateau approach",
        kind: FirstExpeditionBeatKind::PlateauApproach,
        anchor_island_name: "cloudbreak stair",
        position: FirstExpeditionBeatPosition::LiftNode("cloudbreak stair recovery updraft"),
        altitude_band: FirstExpeditionAltitudeBand::High,
        traversal_mode: FirstExpeditionTraversalMode::ApproachGlide,
        landmark_anchor: "great sky plateau west rim silhouette",
        recovery_affordance: FirstExpeditionRecoveryAffordance::RecoveryUpdraft {
            lift_name: "cloudbreak stair recovery updraft",
        },
    },
    FirstExpeditionRouteBeatSpec {
        label: "great sky plateau meadow arrival",
        kind: FirstExpeditionBeatKind::PlateauArrival,
        anchor_island_name: PLAYTEST_RESET_ISLAND_NAME,
        position: FirstExpeditionBeatPosition::PlateauRegion(IslandPlateauRegion::MeadowPlateau),
        altitude_band: FirstExpeditionAltitudeBand::Plateau,
        traversal_mode: FirstExpeditionTraversalMode::ArrivalLanding,
        landmark_anchor: "plateau lake waterfall vista and cave mouth",
        recovery_affordance: FirstExpeditionRecoveryAffordance::PlateauReset {
            island_name: PLAYTEST_RESET_ISLAND_NAME,
            lift_name: "great sky plateau updraft",
        },
    },
];

const FIRST_EXPEDITION_OPTIONAL_DETOUR_SPECS: [FirstExpeditionOptionalDetourSpec; 2] = [
    FirstExpeditionOptionalDetourSpec {
        label: "low reef missed-glide recovery loop",
        kind: FirstExpeditionDetourKind::LowAltitudeRecoveryLoop,
        risk: FirstExpeditionDetourRisk::Recovery,
        entry_beat_kind: FirstExpeditionBeatKind::FirstGlide,
        reconnect_beat_kind: FirstExpeditionBeatKind::LowRecovery,
        altitude_band: FirstExpeditionAltitudeBand::Low,
        island_names: &LOW_ALTITUDE_RECOVERY_LOOP_ISLANDS,
        lift_names: &LOW_ALTITUDE_RECOVERY_LOOP_LIFTS,
        reconnect_lift_name: "distant recovery updraft",
        landmark_anchor: "low reef wind ribbons and western catch cairn",
    },
    FirstExpeditionOptionalDetourSpec {
        label: "bluevault west-rim upper challenge path",
        kind: FirstExpeditionDetourKind::HighRiskUpperPath,
        risk: FirstExpeditionDetourRisk::HighRiskHighReward,
        entry_beat_kind: FirstExpeditionBeatKind::HighClimb,
        reconnect_beat_kind: FirstExpeditionBeatKind::PlateauArrival,
        altitude_band: FirstExpeditionAltitudeBand::High,
        island_names: &HIGH_RISK_UPPER_PATH_ISLANDS,
        lift_names: &HIGH_RISK_UPPER_PATH_LIFTS,
        reconnect_lift_name: "plateau west rim recovery updraft",
        landmark_anchor: "bluevault lake shoulder and plateau west rim thermal",
    },
];

const FIRST_EXPEDITION_NAVIGATION_LANDMARK_SPECS: [FirstExpeditionNavigationLandmarkSpec; 10] = [
    FirstExpeditionNavigationLandmarkSpec {
        label: "launch beacon",
        kind: FirstExpeditionNavigationLandmarkKind::LaunchBeacon,
        context: FirstExpeditionNavigationContext::RequiredBeat(FirstExpeditionBeatKind::Launch),
        island_name: "launch mesa",
        visual_anchor: "launch beacon",
        position: FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
            normalized_offset: [-0.42, 0.38],
            height_offset_m: 1.6,
        },
        altitude_band: FirstExpeditionAltitudeBand::Low,
        readable_radius_m: 90.0,
        required_route: true,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "midpoint cairn line",
        kind: FirstExpeditionNavigationLandmarkKind::RouteCairn,
        context: FirstExpeditionNavigationContext::RequiredBeat(
            FirstExpeditionBeatKind::FirstGlide,
        ),
        island_name: "midpoint shelf",
        visual_anchor: "route cairn line",
        position: FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
            normalized_offset: [-0.18, 0.22],
            height_offset_m: 2.25,
        },
        altitude_band: FirstExpeditionAltitudeBand::Low,
        readable_radius_m: 105.0,
        required_route: true,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "low reef wind ribbons",
        kind: FirstExpeditionNavigationLandmarkKind::RecoveryUpdraft,
        context: FirstExpeditionNavigationContext::RequiredBeat(
            FirstExpeditionBeatKind::LowRecovery,
        ),
        island_name: "low reef",
        visual_anchor: "low reef wind ribbons",
        position: FirstExpeditionNavigationLandmarkPosition::LiftNode("low reef updraft"),
        altitude_band: FirstExpeditionAltitudeBand::Low,
        readable_radius_m: 130.0,
        required_route: true,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "underbridge cave mouth arch",
        kind: FirstExpeditionNavigationLandmarkKind::CaveMouth,
        context: FirstExpeditionNavigationContext::RequiredBeat(
            FirstExpeditionBeatKind::UnderRoutePass,
        ),
        island_name: "underbridge cay",
        visual_anchor: "under-route cave mouth arch",
        position: FirstExpeditionNavigationLandmarkPosition::UnderRouteEntry,
        altitude_band: FirstExpeditionAltitudeBand::Low,
        readable_radius_m: 95.0,
        required_route: true,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "cloudfall waterfall lake sightline",
        kind: FirstExpeditionNavigationLandmarkKind::WaterfallLakeVista,
        context: FirstExpeditionNavigationContext::RequiredBeat(
            FirstExpeditionBeatKind::LakeWaterfallLandmark,
        ),
        island_name: "cloudfall meadow",
        visual_anchor: "route edge waterfall and bluevault basin lake",
        position: FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
            normalized_offset: [0.54, 0.10],
            height_offset_m: 1.2,
        },
        altitude_band: FirstExpeditionAltitudeBand::Mid,
        readable_radius_m: 185.0,
        required_route: true,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "sunspire garden ring",
        kind: FirstExpeditionNavigationLandmarkKind::GardenRing,
        context: FirstExpeditionNavigationContext::RequiredBeat(FirstExpeditionBeatKind::HighClimb),
        island_name: "sunspire garden",
        visual_anchor: "sunspire garden ring",
        position: FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
            normalized_offset: [0.0, 0.0],
            height_offset_m: 0.35,
        },
        altitude_band: FirstExpeditionAltitudeBand::High,
        readable_radius_m: 145.0,
        required_route: true,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "cloudbreak plateau rim silhouette",
        kind: FirstExpeditionNavigationLandmarkKind::PlateauRim,
        context: FirstExpeditionNavigationContext::RequiredBeat(
            FirstExpeditionBeatKind::PlateauApproach,
        ),
        island_name: "cloudbreak stair",
        visual_anchor: "great sky plateau west rim silhouette",
        position: FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
            normalized_offset: [-0.18, 0.22],
            height_offset_m: 3.0,
        },
        altitude_band: FirstExpeditionAltitudeBand::High,
        readable_radius_m: 210.0,
        required_route: true,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "great sky plateau arrival ruin",
        kind: FirstExpeditionNavigationLandmarkKind::PlateauArrivalRuin,
        context: FirstExpeditionNavigationContext::RequiredBeat(
            FirstExpeditionBeatKind::PlateauArrival,
        ),
        island_name: PLAYTEST_RESET_ISLAND_NAME,
        visual_anchor: "plateau arrival ruin marker",
        position: FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
            normalized_offset: [-0.16, 0.12],
            height_offset_m: 10.0,
        },
        altitude_band: FirstExpeditionAltitudeBand::Plateau,
        readable_radius_m: 220.0,
        required_route: true,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "western refuge recovery cairn",
        kind: FirstExpeditionNavigationLandmarkKind::DetourMarker,
        context: FirstExpeditionNavigationContext::OptionalDetour(
            FirstExpeditionDetourKind::LowAltitudeRecoveryLoop,
        ),
        island_name: "western refuge",
        visual_anchor: "western catch cairn",
        position: FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
            normalized_offset: [-0.18, 0.22],
            height_offset_m: 2.8,
        },
        altitude_band: FirstExpeditionAltitudeBand::Low,
        readable_radius_m: 115.0,
        required_route: false,
    },
    FirstExpeditionNavigationLandmarkSpec {
        label: "bluevault lake shoulder marker",
        kind: FirstExpeditionNavigationLandmarkKind::DetourMarker,
        context: FirstExpeditionNavigationContext::OptionalDetour(
            FirstExpeditionDetourKind::HighRiskUpperPath,
        ),
        island_name: "bluevault basin",
        visual_anchor: "bluevault lake shoulder",
        position: FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
            normalized_offset: [0.08, -0.10],
            height_offset_m: 0.8,
        },
        altitude_band: FirstExpeditionAltitudeBand::High,
        readable_radius_m: 150.0,
        required_route: false,
    },
];

#[derive(Resource, Clone, Debug)]
pub struct SkyRoute {
    pub fallback_floor_y: f32,
    islands: Vec<SkyIsland>,
}

impl Default for SkyRoute {
    fn default() -> Self {
        Self {
            fallback_floor_y: PLAYER_STANDING_OFFSET,
            islands: vec![
                SkyIsland::new(
                    "launch mesa",
                    Vec3::new(0.0, START_FLOOR_Y, 0.0),
                    Vec2::new(40.0, 32.0),
                    11.0,
                    false,
                ),
                SkyIsland::new(
                    "midpoint shelf",
                    Vec3::new(-12.0, 44.0, -128.0),
                    Vec2::new(34.0, 28.0),
                    9.0,
                    false,
                ),
                SkyIsland::new(
                    "landing garden",
                    Vec3::new(-38.0, 52.0, -263.0),
                    Vec2::new(58.0, 42.0),
                    12.0,
                    true,
                ),
                SkyIsland::new(
                    "distant crown",
                    Vec3::new(82.0, 62.0, -356.0),
                    Vec2::new(44.0, 36.0),
                    14.0,
                    false,
                ),
                SkyIsland::new(
                    "wind overlook",
                    Vec3::new(-112.0, 52.0, -204.0),
                    Vec2::new(36.0, 30.0),
                    10.0,
                    false,
                ),
                SkyIsland::new(
                    "copper stair",
                    Vec3::new(36.0, 58.0, -332.0),
                    Vec2::new(26.0, 22.0),
                    9.0,
                    false,
                ),
                SkyIsland::new(
                    "sunlit terrace",
                    Vec3::new(42.0, 64.0, -444.0),
                    Vec2::new(68.0, 38.0),
                    13.0,
                    false,
                ),
                SkyIsland::new(
                    "western refuge",
                    Vec3::new(-150.0, 70.0, -432.0),
                    Vec2::new(46.0, 34.0),
                    12.0,
                    false,
                ),
                SkyIsland::new(
                    "storm porch",
                    Vec3::new(-74.0, 76.0, -548.0),
                    Vec2::new(50.0, 34.0),
                    15.0,
                    false,
                ),
                SkyIsland::new(
                    "high orchard",
                    Vec3::new(18.0, 82.0, -662.0),
                    Vec2::new(72.0, 46.0),
                    14.0,
                    false,
                ),
                SkyIsland::new(
                    "far needle",
                    Vec3::new(142.0, 92.0, -742.0),
                    Vec2::new(24.0, 22.0),
                    18.0,
                    false,
                ),
                SkyIsland::new(
                    "sapphire basin",
                    Vec3::new(-58.0, 88.0, -818.0),
                    Vec2::new(56.0, 40.0),
                    16.0,
                    false,
                ),
                SkyIsland::new(
                    "broken stair",
                    Vec3::new(-176.0, 98.0, -708.0),
                    Vec2::new(32.0, 44.0),
                    17.0,
                    false,
                ),
                SkyIsland::new(
                    "mist arch",
                    Vec3::new(82.0, 104.0, -926.0),
                    Vec2::new(78.0, 34.0),
                    20.0,
                    false,
                ),
                SkyIsland::new(
                    "cloud gate",
                    Vec3::new(204.0, 112.0, -1048.0),
                    Vec2::new(50.0, 42.0),
                    19.0,
                    false,
                ),
                SkyIsland::new(
                    "launch spur",
                    Vec3::new(58.0, 34.0, -72.0),
                    Vec2::new(21.0, 16.0),
                    9.0,
                    false,
                ),
                SkyIsland::new(
                    "garden apron",
                    Vec3::new(-102.0, 56.0, -306.0),
                    Vec2::new(28.0, 18.0),
                    10.0,
                    false,
                ),
                SkyIsland::new(
                    "storm shard",
                    Vec3::new(-132.0, 84.0, -614.0),
                    Vec2::new(24.0, 18.0),
                    16.0,
                    false,
                ),
                SkyIsland::new(
                    "orchard spur",
                    Vec3::new(82.0, 88.0, -638.0),
                    Vec2::new(28.0, 18.0),
                    12.0,
                    false,
                ),
                SkyIsland::new(
                    "mist stepping stone",
                    Vec3::new(148.0, 108.0, -990.0),
                    Vec2::new(22.0, 18.0),
                    15.0,
                    false,
                ),
                SkyIsland::new(
                    "underbridge cay",
                    Vec3::new(-64.0, 18.0, -92.0),
                    Vec2::new(18.0, 14.0),
                    9.0,
                    false,
                ),
                SkyIsland::new(
                    "low reef",
                    Vec3::new(92.0, 22.0, -188.0),
                    Vec2::new(34.0, 20.0),
                    9.0,
                    false,
                ),
                SkyIsland::new(
                    "quiet lower garden",
                    Vec3::new(-188.0, 38.0, -238.0),
                    Vec2::new(40.0, 30.0),
                    9.0,
                    false,
                ),
                SkyIsland::new(
                    "lowwind shelf",
                    Vec3::new(178.0, 24.0, -412.0),
                    Vec2::new(38.0, 24.0),
                    9.0,
                    false,
                ),
                SkyIsland::new(
                    "upper thermal ring",
                    Vec3::new(122.0, 138.0, -520.0),
                    Vec2::new(42.0, 32.0),
                    18.0,
                    false,
                ),
                SkyIsland::new(
                    "needle crownlet",
                    Vec3::new(250.0, 148.0, -832.0),
                    Vec2::new(20.0, 18.0),
                    21.0,
                    false,
                ),
                SkyIsland::new(
                    "skyhook basin",
                    Vec3::new(-238.0, 128.0, -882.0),
                    Vec2::new(66.0, 44.0),
                    22.0,
                    false,
                ),
                SkyIsland::new(
                    "stratos shelf",
                    Vec3::new(-22.0, 156.0, -1138.0),
                    Vec2::new(86.0, 52.0),
                    24.0,
                    false,
                ),
                SkyIsland::new(
                    "cloudfall meadow",
                    Vec3::new(-144.0, 142.0, -1208.0),
                    Vec2::new(74.0, 54.0),
                    20.0,
                    false,
                ),
                SkyIsland::new(
                    "highgate stair",
                    Vec3::new(260.0, 172.0, -1210.0),
                    Vec2::new(30.0, 24.0),
                    25.0,
                    false,
                ),
                SkyIsland::new(
                    "thin air roost",
                    Vec3::new(54.0, 196.0, -1355.0),
                    Vec2::new(24.0, 20.0),
                    28.0,
                    false,
                ),
                SkyIsland::new(
                    "summit anvil",
                    Vec3::new(-18.0, 218.0, -1510.0),
                    Vec2::new(90.0, 46.0),
                    30.0,
                    false,
                ),
                SkyIsland::new(
                    "upper sky shelf",
                    Vec3::new(-210.0, 285.0, -1720.0),
                    Vec2::new(86.0, 54.0),
                    32.0,
                    false,
                ),
                SkyIsland::new(
                    "east windchain",
                    Vec3::new(300.0, 318.0, -1695.0),
                    Vec2::new(34.0, 26.0),
                    28.0,
                    false,
                ),
                SkyIsland::new(
                    "bluevault basin",
                    Vec3::new(90.0, 365.0, -1905.0),
                    Vec2::new(84.0, 54.0),
                    34.0,
                    false,
                ),
                SkyIsland::new(
                    "outer switchback",
                    Vec3::new(-380.0, 430.0, -2050.0),
                    Vec2::new(42.0, 30.0),
                    32.0,
                    false,
                ),
                SkyIsland::new(
                    "sunspire garden",
                    Vec3::new(420.0, 505.0, -2240.0),
                    Vec2::new(72.0, 46.0),
                    36.0,
                    false,
                ),
                SkyIsland::new(
                    "cloudbreak stair",
                    Vec3::new(160.0, 580.0, -2410.0),
                    Vec2::new(36.0, 30.0),
                    38.0,
                    false,
                ),
                SkyIsland::new(
                    "great sky plateau",
                    Vec3::new(-120.0, 690.0, -2600.0),
                    Vec2::new(230.0, 155.0),
                    72.0,
                    false,
                ),
                SkyIsland::new(
                    "far horizon perch",
                    Vec3::new(520.0, 820.0, -2860.0),
                    Vec2::new(64.0, 44.0),
                    46.0,
                    false,
                ),
                SkyIsland::new(
                    "upper crown",
                    Vec3::new(-360.0, 1040.0, -3200.0),
                    Vec2::new(82.0, 52.0),
                    50.0,
                    false,
                ),
            ],
        }
    }
}

fn lift_route_node_count_for_target(target: SkyIsland) -> usize {
    let mut count = if target.is_target { 1 } else { 2 };
    let target_depth = -target.center.z;
    for (index, node) in GAMEPLAY_LIFT_ROUTE.iter().enumerate().skip(2) {
        if !is_route_objective_lift_node(node.name) {
            continue;
        }
        let node_depth = -node.center.z;
        if target.center.y >= node.center.y - 36.0 || target_depth >= node_depth - 40.0 {
            count = index + 1;
        }
    }

    count.min(GAMEPLAY_LIFT_ROUTE.len())
}

fn is_route_objective_lift_node(name: &str) -> bool {
    !matches!(
        name,
        "low reef updraft"
            | "western catch updraft"
            | "skyhook basin updraft"
            | "cloudfall meadow updraft"
            | "underbridge cay updraft"
            | "bluevault shoulder recovery updraft"
            | "cloudbreak stair recovery updraft"
            | "plateau west rim recovery updraft"
    )
}

impl SkyRoute {
    pub fn islands(&self) -> &[SkyIsland] {
        &self.islands
    }

    pub fn under_island_route_segments(&self) -> Vec<IslandUnderRouteSegment> {
        self.islands
            .iter()
            .copied()
            .filter_map(SkyIsland::under_route_segment)
            .collect()
    }

    pub fn first_expedition_route_beats(&self) -> Vec<FirstExpeditionRouteBeat> {
        FIRST_EXPEDITION_ROUTE_BEAT_SPECS
            .iter()
            .filter_map(|spec| self.first_expedition_route_beat(*spec))
            .collect()
    }

    pub fn first_expedition_optional_detours(&self) -> Vec<FirstExpeditionOptionalDetour> {
        FIRST_EXPEDITION_OPTIONAL_DETOUR_SPECS
            .iter()
            .filter_map(|spec| self.first_expedition_optional_detour(*spec))
            .collect()
    }

    pub fn first_expedition_navigation_landmarks(&self) -> Vec<FirstExpeditionNavigationLandmark> {
        FIRST_EXPEDITION_NAVIGATION_LANDMARK_SPECS
            .iter()
            .filter_map(|spec| self.first_expedition_navigation_landmark(*spec))
            .collect()
    }

    fn first_expedition_route_beat(
        &self,
        spec: FirstExpeditionRouteBeatSpec,
    ) -> Option<FirstExpeditionRouteBeat> {
        let anchor_island = self.island_named(spec.anchor_island_name)?;
        let position = match spec.position {
            FirstExpeditionBeatPosition::IslandSurface => {
                let mut position = anchor_island.center;
                position.y = self.ground_at(position).floor_y;
                position
            }
            FirstExpeditionBeatPosition::LiftNode(lift_name) => {
                lift_route_node_named(lift_name)?.center
            }
            FirstExpeditionBeatPosition::UnderRouteMidpoint => {
                anchor_island.under_route_segment()?.midpoint
            }
            FirstExpeditionBeatPosition::PlateauRegion(region) => {
                anchor_island.plateau_region_position(region)?
            }
        };

        Some(FirstExpeditionRouteBeat {
            label: spec.label,
            kind: spec.kind,
            anchor_island_name: spec.anchor_island_name,
            position,
            altitude_band: spec.altitude_band,
            traversal_mode: spec.traversal_mode,
            landmark_anchor: spec.landmark_anchor,
            recovery_affordance: spec.recovery_affordance,
        })
    }

    fn first_expedition_optional_detour(
        &self,
        spec: FirstExpeditionOptionalDetourSpec,
    ) -> Option<FirstExpeditionOptionalDetour> {
        let mut route_positions = Vec::new();
        for island_name in spec.island_names {
            let island = self.island_named(island_name)?;
            let mut position = island.center;
            position.y = self.ground_at(position).floor_y;
            route_positions.push(position);
        }
        for lift_name in spec.lift_names {
            route_positions.push(lift_route_node_named(lift_name)?.center);
        }
        lift_route_node_named(spec.reconnect_lift_name)?;

        Some(FirstExpeditionOptionalDetour {
            label: spec.label,
            kind: spec.kind,
            risk: spec.risk,
            entry_beat_kind: spec.entry_beat_kind,
            reconnect_beat_kind: spec.reconnect_beat_kind,
            altitude_band: spec.altitude_band,
            island_names: spec.island_names,
            lift_names: spec.lift_names,
            reconnect_lift_name: spec.reconnect_lift_name,
            landmark_anchor: spec.landmark_anchor,
            route_positions,
        })
    }

    fn first_expedition_navigation_landmark(
        &self,
        spec: FirstExpeditionNavigationLandmarkSpec,
    ) -> Option<FirstExpeditionNavigationLandmark> {
        let island = self.island_named(spec.island_name)?;
        let position = match spec.position {
            FirstExpeditionNavigationLandmarkPosition::IslandSurfaceOffset {
                normalized_offset,
                height_offset_m,
            } => {
                let x = island.center.x + normalized_offset[0] * island.half_extents.x;
                let z = island.center.z + normalized_offset[1] * island.half_extents.y;
                Vec3::new(
                    x,
                    island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) + height_offset_m,
                    z,
                )
            }
            FirstExpeditionNavigationLandmarkPosition::LiftNode(lift_name) => {
                lift_route_node_named(lift_name)?.center
            }
            FirstExpeditionNavigationLandmarkPosition::UnderRouteEntry => {
                island.under_route_segment()?.entry
            }
        };

        Some(FirstExpeditionNavigationLandmark {
            label: spec.label,
            kind: spec.kind,
            context: spec.context,
            island_name: spec.island_name,
            visual_anchor: spec.visual_anchor,
            position,
            altitude_band: spec.altitude_band,
            readable_radius_m: spec.readable_radius_m,
            required_route: spec.required_route,
        })
    }

    pub fn route_objectives(&self, island_name: Option<&str>) -> Vec<RouteObjective> {
        let Some(target) = self.tracked_target_island(island_name) else {
            return Vec::new();
        };

        let lift_node_count = lift_route_node_count_for_target(target);
        let mut objectives = GAMEPLAY_LIFT_ROUTE
            .iter()
            .copied()
            .take(lift_node_count)
            .map(RouteObjective::fly_through)
            .collect::<Vec<_>>();
        objectives.push(RouteObjective::land_on(target));

        objectives
    }

    pub fn playtest_reset_position(&self) -> Vec3 {
        let island = self
            .island_named(PLAYTEST_RESET_ISLAND_NAME)
            .or_else(|| self.target_island())
            .expect("default route should include a reset island");
        let mut position = island.center;
        position.y = self.ground_at(position).floor_y;
        position
    }

    pub fn streaming_lod_stats(&self, position: Vec3) -> StreamingLodStats {
        let player_chunk = StreamChunkCoord::from_world(position);
        let active_chunk_width = super::STREAM_ACTIVE_CHUNK_RADIUS * 2 + 1;
        let mut stats = StreamingLodStats {
            player_chunk,
            active_chunk_count: (active_chunk_width * active_chunk_width) as usize,
            ..Default::default()
        };

        for island in &self.islands {
            if island.stream_activation(position).is_active() {
                stats.active_island_count += 1;
            }

            match island.lod_band(position) {
                LodBand::Near => stats.near_lod_islands += 1,
                LodBand::Mid => stats.mid_lod_islands += 1,
                LodBand::Far => stats.far_lod_islands += 1,
            }
        }

        stats
    }

    pub fn ground_at(&self, position: Vec3) -> GroundSurface {
        self.islands
            .iter()
            .copied()
            .filter(|island| island.contains_horizontal(position))
            .filter(|island| !position_is_inside_under_route_clearance(*island, position))
            .map(|island| GroundSurface::from_island_at(island, position))
            .max_by(|a, b| a.floor_y.total_cmp(&b.floor_y))
            .unwrap_or(GroundSurface {
                floor_y: self.fallback_floor_y,
                is_target: false,
                island_name: None,
            })
    }

    pub fn is_grounded_at(&self, position: Vec3) -> bool {
        let ground = self.ground_at(position);
        position.y <= ground.floor_y + GROUND_CONTACT_EPSILON
    }

    pub fn resolve_ground_contact(&self, state: FlightState) -> FlightState {
        self.resolve_ground_contact_with_landing(state, true)
    }

    pub fn resolve_ground_contact_after_step(
        &self,
        state: FlightState,
        was_grounded: bool,
    ) -> FlightState {
        self.resolve_ground_contact_with_landing(state, !was_grounded)
    }

    pub fn resolve_grounded_after_horizontal_correction(
        &self,
        mut state: FlightState,
    ) -> FlightState {
        if state.controller.mode != FlightMode::Grounded {
            return state;
        }

        let ground = self.ground_at(state.position);
        if state.position.y <= ground.floor_y + GROUND_CONTACT_EPSILON {
            state.position.y = ground.floor_y;
            state.velocity.y = state.velocity.y.max(0.0);
            state.controller.launch_timer = 0.0;
            state.controller.launch_available = true;
            state.controller.bank_degrees = 0.0;
        }

        state
    }

    fn resolve_ground_contact_with_landing(
        &self,
        mut state: FlightState,
        apply_landing_damping: bool,
    ) -> FlightState {
        let ground = self.ground_at(state.position);
        if state.position.y <= ground.floor_y + GROUND_CONTACT_EPSILON {
            let impact_speed_mps = (-state.velocity.y).max(0.0);
            state.position.y = ground.floor_y;
            if apply_landing_damping {
                state.velocity.x *= GROUND_CONTACT_HORIZONTAL_DAMPING;
                state.velocity.z *= GROUND_CONTACT_HORIZONTAL_DAMPING;
                state.controller.record_landing_impact(impact_speed_mps);
            }
            state.velocity.y = state.velocity.y.max(0.0);
            state.controller.launch_timer = 0.0;
            state.controller.launch_available = true;
            state.controller.mode = FlightMode::Grounded;
            state.controller.bank_degrees = 0.0;
        } else if state.controller.mode == FlightMode::Grounded {
            state.controller.mode = FlightMode::Airborne;
            state.controller.launch_timer = 0.0;
        }

        state
    }

    pub fn target_distance(&self, position: Vec3) -> f32 {
        self.target_distance_to(position, None)
    }

    pub fn target_distance_to(&self, position: Vec3, island_name: Option<&str>) -> f32 {
        self.tracked_target_island(island_name)
            .map(|island| island.horizontal_distance(position))
            .unwrap_or(0.0)
    }

    pub fn on_landing_target(&self, position: Vec3, mode: FlightMode) -> bool {
        self.on_landing_target_named(position, mode, None)
    }

    pub fn on_landing_target_named(
        &self,
        position: Vec3,
        mode: FlightMode,
        island_name: Option<&str>,
    ) -> bool {
        let ground = self.ground_at(position);
        self.tracked_target_island(island_name)
            .is_some_and(|island| ground.island_name == Some(island.name))
            && mode == FlightMode::Grounded
            && (position.y - ground.floor_y).abs() <= 0.1
    }

    pub fn target_island(&self) -> Option<SkyIsland> {
        self.islands.iter().copied().find(|island| island.is_target)
    }

    pub fn island_named(&self, name: &str) -> Option<SkyIsland> {
        self.islands
            .iter()
            .copied()
            .find(|island| island.name == name)
    }

    fn tracked_target_island(&self, island_name: Option<&str>) -> Option<SkyIsland> {
        island_name
            .and_then(|name| self.island_named(name))
            .or_else(|| self.target_island())
    }
}

fn lift_route_node_named(name: &str) -> Option<crate::environment::LiftRouteNode> {
    GAMEPLAY_LIFT_ROUTE
        .iter()
        .copied()
        .find(|node| node.name == name)
}

fn position_is_inside_under_route_clearance(island: SkyIsland, position: Vec3) -> bool {
    let Some(segment) = island.under_route_segment() else {
        return false;
    };
    let under_top_surface = position.y
        < island.mesh_top_y() - island.thickness * UNDER_ROUTE_TOP_SURFACE_CLEARANCE_FRACTION;

    under_top_surface
        && segment.contains_clearance(position, UNDER_ROUTE_GROUND_CLEARANCE_PADDING_M)
}
