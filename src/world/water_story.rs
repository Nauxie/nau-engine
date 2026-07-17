use bevy::prelude::{Vec2, Vec3};

use super::SkyIsland;

pub const ROUTE_EDGE_WATERFALL_CHANNEL_OUTLET_OFFSET: Vec2 = Vec2::new(-0.42, 0.16);
const ROUTE_EDGE_WATERFALL_LIP_OFFSET: Vec2 = Vec2::new(-0.76, 0.28);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteEdgeWaterfallPlacement {
    pub ribbon_translation: Vec3,
    pub mist_translation: Vec3,
    pub rotation_y: f32,
    pub height: f32,
    pub width: f32,
}

pub fn route_edge_waterfall_placement(island: SkyIsland) -> RouteEdgeWaterfallPlacement {
    let surface = island_water_surface_position(island, ROUTE_EDGE_WATERFALL_LIP_OFFSET);
    let source = island_water_surface_position(island, ROUTE_EDGE_WATERFALL_CHANNEL_OUTLET_OFFSET);
    let outward = ROUTE_EDGE_WATERFALL_LIP_OFFSET.normalize_or_zero();
    let outward3 = Vec3::new(outward.x, 0.0, outward.y);
    let height = (island.thickness * 1.25).clamp(18.0, 34.0);
    let width = island.half_extents.min_element() * 0.16;
    let source_drop = (source.y - surface.y).max(0.0) * 0.12;

    RouteEdgeWaterfallPlacement {
        ribbon_translation: surface + outward3 * 4.5 - Vec3::Y * (height * 0.46 + source_drop),
        mist_translation: surface + outward3 * 7.5 - Vec3::Y * (height * 0.90 + source_drop),
        rotation_y: outward.x.atan2(outward.y),
        height,
        width,
    }
}

fn island_water_surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let radius = normalized_offset.length();
    let playable_offset = if radius <= f32::EPSILON {
        Vec2::ZERO
    } else {
        let angle = normalized_offset.y.atan2(normalized_offset.x);
        let max_radius = island.playable_silhouette_scale(angle) * 0.94;
        normalized_offset / radius * radius.min(max_radius)
    };
    let x = island.center.x + island.half_extents.x * playable_offset.x;
    let z = island.center.z + island.half_extents.y * playable_offset.y;
    Vec3::new(x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z)
}
