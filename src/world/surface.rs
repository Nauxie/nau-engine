use bevy::prelude::Vec3;

use super::SkyIsland;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundSurface {
    pub floor_y: f32,
    pub is_target: bool,
    pub island_name: Option<&'static str>,
}

impl GroundSurface {
    pub(super) fn from_island_at(island: SkyIsland, position: Vec3) -> Self {
        Self {
            floor_y: island.terrain_surface_y_at(position),
            is_target: island.is_target,
            island_name: Some(island.name),
        }
    }
}
