use crate::environment::{GAMEPLAY_LIFT_ROUTE, LiftRouteNode};
use crate::movement::{FlightMode, FlightState};
use bevy::prelude::*;

pub const PLAYER_STANDING_OFFSET: f32 = 0.24;
pub const START_FLOOR_Y: f32 = 28.0;
pub const START_POSITION: Vec3 = Vec3::new(0.0, START_FLOOR_Y, 0.0);
pub const RECOVERY_BRANCH_ISLANDS: [&str; 2] = ["sunlit terrace", "western refuge"];
pub const STREAM_CHUNK_SIZE_M: f32 = 160.0;
pub const STREAM_ACTIVE_CHUNK_RADIUS: i32 = 2;
pub const LOD_NEAR_DISTANCE_M: f32 = 220.0;
pub const LOD_MID_DISTANCE_M: f32 = 520.0;
pub const TERRAIN_MAX_RISE_M: f32 = 0.45;
pub const TERRAIN_MAX_DROP_M: f32 = 0.75;
pub const TERRAIN_VISUAL_FOOTING_OFFSET_M: f32 = 0.18;
const GROUND_CONTACT_EPSILON: f32 = 0.05;
const GROUND_CONTACT_HORIZONTAL_DAMPING: f32 = 0.58;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StreamChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl StreamChunkCoord {
    pub fn from_world(position: Vec3) -> Self {
        Self {
            x: (position.x / STREAM_CHUNK_SIZE_M).floor() as i32,
            z: (position.z / STREAM_CHUNK_SIZE_M).floor() as i32,
        }
    }

    pub fn is_inside_active_window(self, center: Self) -> bool {
        (self.x - center.x).abs() <= STREAM_ACTIVE_CHUNK_RADIUS
            && (self.z - center.z).abs() <= STREAM_ACTIVE_CHUNK_RADIUS
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StreamingLodStats {
    pub player_chunk: StreamChunkCoord,
    pub active_chunk_count: usize,
    pub active_island_count: usize,
    pub near_lod_islands: usize,
    pub mid_lod_islands: usize,
    pub far_lod_islands: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LodBand {
    Near,
    Mid,
    Far,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamActivation {
    Active,
    Inactive,
}

impl StreamActivation {
    pub fn is_active(self) -> bool {
        self == Self::Active
    }
}

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
        let active_chunk_width = STREAM_ACTIVE_CHUNK_RADIUS * 2 + 1;
        let mut stats = StreamingLodStats {
            player_chunk,
            active_chunk_count: (active_chunk_width * active_chunk_width) as usize,
            ..default()
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
            state.position.y = ground.floor_y;
            if apply_landing_damping {
                state.velocity.x *= GROUND_CONTACT_HORIZONTAL_DAMPING;
                state.velocity.z *= GROUND_CONTACT_HORIZONTAL_DAMPING;
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

pub fn is_recovery_branch_island(name: &str) -> bool {
    RECOVERY_BRANCH_ISLANDS.contains(&name)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteObjectiveKind {
    FlyThrough,
    Land,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteObjective {
    pub label: &'static str,
    pub position: Vec3,
    pub radius_m: f32,
    pub kind: RouteObjectiveKind,
    pub island_name: Option<&'static str>,
}

impl RouteObjective {
    pub fn fly_through(node: LiftRouteNode) -> Self {
        Self {
            label: node.name,
            position: node.center,
            radius_m: node.half_extents.x.max(node.half_extents.z) + 8.0,
            kind: RouteObjectiveKind::FlyThrough,
            island_name: None,
        }
    }

    pub fn land_on(island: SkyIsland) -> Self {
        Self {
            label: island.name,
            position: island.center,
            radius_m: island.half_extents.x.max(island.half_extents.y),
            kind: RouteObjectiveKind::Land,
            island_name: Some(island.name),
        }
    }

    pub fn horizontal_distance(self, position: Vec3) -> f32 {
        Vec2::new(position.x - self.position.x, position.z - self.position.z).length()
    }

    pub fn is_complete(self, route: &SkyRoute, position: Vec3, mode: FlightMode) -> bool {
        match self.kind {
            RouteObjectiveKind::FlyThrough => self.horizontal_distance(position) <= self.radius_m,
            RouteObjectiveKind::Land => {
                route.on_landing_target_named(position, mode, self.island_name)
            }
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SkyIsland {
    pub name: &'static str,
    pub center: Vec3,
    pub half_extents: Vec2,
    pub thickness: f32,
    pub is_target: bool,
}

impl SkyIsland {
    pub fn new(
        name: &'static str,
        center: Vec3,
        half_extents: Vec2,
        thickness: f32,
        is_target: bool,
    ) -> Self {
        Self {
            name,
            center,
            half_extents,
            thickness: thickness.max(1.0),
            is_target,
        }
    }

    pub fn floor_y(self) -> f32 {
        self.center.y
    }

    pub fn mesh_top_y(self) -> f32 {
        self.floor_y() - PLAYER_STANDING_OFFSET
    }

    pub fn terrain_surface_y_at(self, position: Vec3) -> f32 {
        let dx = (position.x - self.center.x) / self.half_extents.x.max(0.001);
        let dz = (position.z - self.center.z) / self.half_extents.y.max(0.001);
        let radius = Vec2::new(dx, dz).length().clamp(0.0, 1.0);
        let angle = dz.atan2(dx);

        self.terrain_surface_y_at_polar(radius, angle)
    }

    pub fn terrain_surface_y_at_polar(self, radius: f32, angle: f32) -> f32 {
        self.floor_y() + self.terrain_relief_m(radius, angle)
    }

    pub fn mesh_top_y_at(self, position: Vec3) -> f32 {
        self.terrain_surface_y_at(position) - TERRAIN_VISUAL_FOOTING_OFFSET_M
    }

    pub fn mesh_top_y_at_polar(self, radius: f32, angle: f32) -> f32 {
        self.terrain_surface_y_at_polar(radius, angle) - TERRAIN_VISUAL_FOOTING_OFFSET_M
    }

    pub fn terrain_relief_m(self, radius: f32, angle: f32) -> f32 {
        let radius = radius.clamp(0.0, 1.0);
        if radius <= f32::EPSILON {
            return 0.0;
        }

        let phase = self.terrain_phase();
        let ridge = radius
            * ((angle * 3.0 + phase).sin() * 0.28 + (angle * 7.0 - phase * 0.5).cos() * 0.14);
        let shoulder = (radius * std::f32::consts::PI).sin() * 0.24;
        let center_falloff = ((1.0 - radius).powi(2) - 1.0) * 0.16;
        let edge_drop = -radius.powf(2.35) * 0.42;

        (ridge + shoulder + center_falloff + edge_drop)
            .clamp(-TERRAIN_MAX_DROP_M, TERRAIN_MAX_RISE_M)
    }

    pub fn contains_horizontal(self, position: Vec3) -> bool {
        let dx = (position.x - self.center.x) / self.half_extents.x.max(0.001);
        let dz = (position.z - self.center.z) / self.half_extents.y.max(0.001);
        dx * dx + dz * dz <= 1.0
    }

    pub fn horizontal_distance(self, position: Vec3) -> f32 {
        Vec2::new(position.x - self.center.x, position.z - self.center.z).length()
    }

    pub fn lod_band(self, position: Vec3) -> LodBand {
        let distance = self.horizontal_distance(position);
        if distance <= LOD_NEAR_DISTANCE_M {
            LodBand::Near
        } else if distance <= LOD_MID_DISTANCE_M {
            LodBand::Mid
        } else {
            LodBand::Far
        }
    }

    pub fn streaming_chunk(self) -> StreamChunkCoord {
        StreamChunkCoord::from_world(self.center)
    }

    pub fn stream_activation(self, position: Vec3) -> StreamActivation {
        let player_chunk = StreamChunkCoord::from_world(position);
        if self.streaming_chunk().is_inside_active_window(player_chunk) {
            StreamActivation::Active
        } else {
            StreamActivation::Inactive
        }
    }

    fn terrain_phase(self) -> f32 {
        self.center.x * 0.013
            + self.center.z * 0.009
            + self.half_extents.x * 0.021
            + self.half_extents.y * 0.017
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundSurface {
    pub floor_y: f32,
    pub is_target: bool,
    pub island_name: Option<&'static str>,
}

impl GroundSurface {
    fn from_island_at(island: SkyIsland, position: Vec3) -> Self {
        Self {
            floor_y: island.terrain_surface_y_at(position),
            is_target: island.is_target,
            island_name: Some(island.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement::{FlightController, FlightMode, FlightState};

    #[test]
    fn route_reports_highest_island_surface_under_player() {
        let route = SkyRoute::default();
        let launch_surface = route.ground_at(START_POSITION);

        assert_eq!(launch_surface.floor_y, START_FLOOR_Y);
        assert_eq!(launch_surface.island_name, Some("launch mesa"));
    }

    #[test]
    fn route_surface_follows_island_relief() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let ridge_position = Vec3::new(
            island.center.x + island.half_extents.x * 0.46,
            START_FLOOR_Y,
            island.center.z + island.half_extents.y * 0.18,
        );
        let edge_position = Vec3::new(
            island.center.x + island.half_extents.x * 0.84,
            START_FLOOR_Y,
            island.center.z - island.half_extents.y * 0.22,
        );

        let center_surface = route.ground_at(island.center);
        let ridge_surface = route.ground_at(ridge_position);
        let edge_surface = route.ground_at(edge_position);

        assert_eq!(center_surface.floor_y, START_FLOOR_Y);
        assert_ne!(ridge_surface.floor_y, center_surface.floor_y);
        assert_ne!(edge_surface.floor_y, center_surface.floor_y);
        assert!(
            ridge_surface.floor_y <= island.floor_y() + TERRAIN_MAX_RISE_M
                && edge_surface.floor_y >= island.floor_y() - TERRAIN_MAX_DROP_M
        );
    }

    #[test]
    fn target_distance_reaches_zero_near_landing_island_center() {
        let route = SkyRoute::default();
        let target = route.target_island().expect("target island exists");

        assert_eq!(route.target_distance(target.center), 0.0);
        assert!(route.target_distance(START_POSITION) > 200.0);
    }

    #[test]
    fn route_can_track_named_recovery_branch_islands() {
        let route = SkyRoute::default();
        let branch = route
            .island_named("sunlit terrace")
            .expect("recovery branch exists");

        assert!(is_recovery_branch_island(branch.name));
        assert_eq!(
            route.target_distance_to(branch.center, Some(branch.name)),
            0.0
        );
        assert!(
            route.target_distance_to(START_POSITION, Some(branch.name))
                > route.target_distance(START_POSITION)
        );
    }

    #[test]
    fn route_objectives_track_main_and_branch_targets() {
        let route = SkyRoute::default();
        let main = route.route_objectives(None);
        let branch = route.route_objectives(Some("sunlit terrace"));

        assert_eq!(main.len(), 2);
        assert_eq!(main[0].label, "near route updraft");
        assert_eq!(main[1].label, "landing garden");
        assert_eq!(branch.len(), 3);
        assert_eq!(branch[1].label, "distant recovery updraft");
        assert_eq!(branch[2].label, "sunlit terrace");
    }

    #[test]
    fn route_objective_completion_tracks_flythrough_and_landing() {
        let route = SkyRoute::default();
        let objectives = route.route_objectives(None);
        let target = route.target_island().expect("target island exists");

        assert!(objectives[0].is_complete(
            &route,
            GAMEPLAY_LIFT_ROUTE[0].center,
            FlightMode::Gliding
        ));
        assert!(!objectives[1].is_complete(
            &route,
            target.center + Vec3::Y * 8.0,
            FlightMode::Gliding
        ));
        assert!(objectives[1].is_complete(&route, target.center, FlightMode::Grounded));
    }

    #[test]
    fn route_has_archipelago_scale_and_distant_landmarks() {
        let route = SkyRoute::default();
        let farthest_z = route
            .islands()
            .iter()
            .map(|island| island.center.z)
            .fold(0.0_f32, f32::min);

        assert!(route.islands().len() >= 12);
        assert!(farthest_z < -800.0);
    }

    #[test]
    fn streaming_lod_stats_track_active_window_and_distance_bands() {
        let route = SkyRoute::default();
        let stats = route.streaming_lod_stats(START_POSITION);

        assert_eq!(stats.player_chunk, StreamChunkCoord { x: 0, z: 0 });
        assert_eq!(stats.active_chunk_count, 25);
        assert!(stats.active_island_count < route.islands().len());
        assert!(stats.active_island_count >= 4);
        assert!(stats.near_lod_islands >= 2);
        assert!(stats.mid_lod_islands >= 3);
        assert!(stats.far_lod_islands >= 3);
    }

    #[test]
    fn island_lod_band_uses_route_distance_thresholds() {
        let island = SkyIsland::new("test island", Vec3::ZERO, Vec2::new(10.0, 10.0), 4.0, false);

        assert_eq!(island.lod_band(Vec3::new(0.0, 0.0, 0.0)), LodBand::Near);
        assert_eq!(
            island.lod_band(Vec3::new(LOD_NEAR_DISTANCE_M + 1.0, 0.0, 0.0)),
            LodBand::Mid
        );
        assert_eq!(
            island.lod_band(Vec3::new(LOD_MID_DISTANCE_M + 1.0, 0.0, 0.0)),
            LodBand::Far
        );
    }

    #[test]
    fn island_stream_activation_uses_chunk_window() {
        let island = SkyIsland::new("test island", Vec3::ZERO, Vec2::new(10.0, 10.0), 4.0, false);

        assert_eq!(
            island.stream_activation(START_POSITION),
            StreamActivation::Active
        );
        assert_eq!(
            island.stream_activation(Vec3::new(
                0.0,
                START_FLOOR_Y,
                STREAM_CHUNK_SIZE_M * (STREAM_ACTIVE_CHUNK_RADIUS + 2) as f32,
            )),
            StreamActivation::Inactive
        );
    }

    #[test]
    fn ground_contact_marks_target_landing_as_grounded() {
        let route = SkyRoute::default();
        let target = route.target_island().expect("target island exists");
        let state = FlightState::new(
            Vec3::new(target.center.x, target.floor_y() - 2.0, target.center.z),
            Vec3::new(20.0, -10.0, 10.0),
            FlightController::default(),
        );

        let resolved = route.resolve_ground_contact(state);

        assert_eq!(resolved.position.y, target.floor_y());
        assert!(resolved.velocity.x < state.velocity.x);
        assert!(resolved.velocity.z < state.velocity.z);
        assert_eq!(resolved.controller.mode, FlightMode::Grounded);
        assert!(route.on_landing_target(resolved.position, resolved.controller.mode));
    }

    #[test]
    fn already_grounded_route_contact_does_not_damp_wasd_motion() {
        let route = SkyRoute::default();
        let state = FlightState::new(
            START_POSITION,
            Vec3::new(8.0, 0.0, -4.0),
            FlightController::default(),
        );

        let resolved = route.resolve_ground_contact_after_step(state, true);

        assert_eq!(resolved.position.y, START_FLOOR_Y);
        assert_eq!(resolved.velocity.x, state.velocity.x);
        assert_eq!(resolved.velocity.z, state.velocity.z);
        assert_eq!(resolved.controller.mode, FlightMode::Grounded);
    }

    #[test]
    fn route_landing_clears_stale_airborne_bank() {
        let route = SkyRoute::default();
        let state = FlightState::new(
            START_POSITION,
            Vec3::new(8.0, -1.0, -4.0),
            FlightController {
                mode: FlightMode::Gliding,
                bank_degrees: 18.0,
                launch_available: false,
                ..default()
            },
        );

        let resolved = route.resolve_ground_contact_after_step(state, false);

        assert_eq!(resolved.controller.mode, FlightMode::Grounded);
        assert_eq!(resolved.controller.bank_degrees, 0.0);
    }

    #[test]
    fn walking_off_island_clears_stale_grounded_mode() {
        let route = SkyRoute::default();
        let state = FlightState::new(
            Vec3::new(200.0, START_FLOOR_Y, 200.0),
            Vec3::new(6.0, 0.0, 0.0),
            FlightController::default(),
        );

        let resolved = route.resolve_ground_contact_after_step(state, true);

        assert_eq!(resolved.controller.mode, FlightMode::Airborne);
        assert_eq!(resolved.position.y, START_FLOOR_Y);
    }

    #[test]
    fn island_visual_top_stays_close_to_player_footing() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let sample = Vec3::new(
            island.center.x + island.half_extents.x * 0.35,
            island.center.y,
            island.center.z - island.half_extents.y * 0.25,
        );
        let visual_offset = island.terrain_surface_y_at(sample) - island.mesh_top_y_at(sample);

        assert!((0.15..=0.3).contains(&visual_offset));
    }
}
