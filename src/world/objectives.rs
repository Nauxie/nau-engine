use crate::environment::LiftRouteNode;
use crate::movement::FlightMode;
use bevy::prelude::{Vec2, Vec3};

use super::{RECOVERY_BRANCH_ISLANDS, SkyIsland, SkyRoute};

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
