use bevy::prelude::*;
use nau_engine::world::SkyIsland;

pub(crate) fn island_visual_surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let normalized_offset = island_playable_normalized_offset(island, normalized_offset);
    let x = island.center.x + island.half_extents.x * normalized_offset.x;
    let z = island.center.z + island.half_extents.y * normalized_offset.y;

    Vec3::new(x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z)
}

pub(crate) fn island_playable_normalized_offset(
    island: SkyIsland,
    normalized_offset: Vec2,
) -> Vec2 {
    let radius = normalized_offset.length();
    if radius <= f32::EPSILON {
        return Vec2::ZERO;
    }

    let angle = normalized_offset.y.atan2(normalized_offset.x);
    let max_radius = island.playable_silhouette_scale(angle) * 0.94;
    normalized_offset / radius * radius.min(max_radius)
}

pub(crate) fn island_silhouette_scale(island: SkyIsland, angle: f32) -> f32 {
    island.visual_silhouette_scale(angle)
}

pub(crate) fn island_polar_position(
    island: SkyIsland,
    angle: f32,
    radius_scale: f32,
    y: f32,
) -> [f32; 3] {
    [
        island.center.x + angle.cos() * island.half_extents.x * radius_scale,
        y,
        island.center.z + angle.sin() * island.half_extents.y * radius_scale,
    ]
}
