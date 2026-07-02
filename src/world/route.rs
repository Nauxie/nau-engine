use crate::environment::GAMEPLAY_LIFT_ROUTE;
use crate::movement::{FlightMode, FlightState};
use bevy::prelude::{Resource, Vec2, Vec3};

use super::{
    GROUND_CONTACT_EPSILON, GROUND_CONTACT_HORIZONTAL_DAMPING, GroundSurface,
    IslandUnderRouteSegment, LodBand, PLAYER_STANDING_OFFSET, RouteObjective, START_FLOOR_Y,
    SkyIsland, StreamChunkCoord, StreamingLodStats,
};

pub const SKY_ROUTE_ISLAND_COUNT: usize = 41;
pub const PLAYTEST_RESET_ISLAND_NAME: &str = "great sky plateau";
const UNDER_ROUTE_GROUND_CLEARANCE_PADDING_M: f32 = 10.0;
const UNDER_ROUTE_TOP_SURFACE_CLEARANCE_FRACTION: f32 = 0.18;

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

fn position_is_inside_under_route_clearance(island: SkyIsland, position: Vec3) -> bool {
    let Some(segment) = island.under_route_segment() else {
        return false;
    };
    let under_top_surface = position.y
        < island.mesh_top_y() - island.thickness * UNDER_ROUTE_TOP_SURFACE_CLEARANCE_FRACTION;

    under_top_surface
        && segment.contains_clearance(position, UNDER_ROUTE_GROUND_CLEARANCE_PADDING_M)
}
