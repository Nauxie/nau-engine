use crate::environment::GAMEPLAY_LIFT_ROUTE;
use crate::movement::{FlightMode, FlightState};
use bevy::prelude::{Resource, Vec2, Vec3};

use super::{
    GROUND_CONTACT_EPSILON, GROUND_CONTACT_HORIZONTAL_DAMPING, GroundSurface, LodBand,
    PLAYER_STANDING_OFFSET, RouteObjective, START_FLOOR_Y, SkyIsland, StreamChunkCoord,
    StreamingLodStats,
};

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
                    Vec2::new(34.0, 28.0),
                    11.0,
                    false,
                ),
                SkyIsland::new(
                    "midpoint shelf",
                    Vec3::new(-12.0, 44.0, -128.0),
                    Vec2::new(28.0, 24.0),
                    9.0,
                    false,
                ),
                SkyIsland::new(
                    "landing garden",
                    Vec3::new(-38.0, 52.0, -263.0),
                    Vec2::new(46.0, 36.0),
                    12.0,
                    true,
                ),
                SkyIsland::new(
                    "distant crown",
                    Vec3::new(82.0, 62.0, -356.0),
                    Vec2::new(38.0, 32.0),
                    14.0,
                    false,
                ),
                SkyIsland::new(
                    "wind overlook",
                    Vec3::new(-112.0, 52.0, -204.0),
                    Vec2::new(30.0, 26.0),
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
                    Vec2::new(54.0, 30.0),
                    13.0,
                    false,
                ),
                SkyIsland::new(
                    "western refuge",
                    Vec3::new(-150.0, 70.0, -432.0),
                    Vec2::new(38.0, 30.0),
                    12.0,
                    false,
                ),
                SkyIsland::new(
                    "storm porch",
                    Vec3::new(-74.0, 76.0, -548.0),
                    Vec2::new(42.0, 28.0),
                    15.0,
                    false,
                ),
                SkyIsland::new(
                    "high orchard",
                    Vec3::new(18.0, 82.0, -662.0),
                    Vec2::new(58.0, 38.0),
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
                    Vec2::new(46.0, 34.0),
                    16.0,
                    false,
                ),
            ],
        }
    }
}

impl SkyRoute {
    pub fn islands(&self) -> &[SkyIsland] {
        &self.islands
    }

    pub fn route_objectives(&self, island_name: Option<&str>) -> Vec<RouteObjective> {
        let mut objectives = vec![RouteObjective::fly_through(GAMEPLAY_LIFT_ROUTE[0])];
        if self
            .tracked_target_island(island_name)
            .is_some_and(|island| !island.is_target)
        {
            objectives.push(RouteObjective::fly_through(GAMEPLAY_LIFT_ROUTE[1]));
        }
        if let Some(target) = self.tracked_target_island(island_name) {
            objectives.push(RouteObjective::land_on(target));
        }

        objectives
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
