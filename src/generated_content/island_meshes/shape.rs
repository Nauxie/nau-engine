use bevy::prelude::*;
use nau_engine::world::SkyIsland;

pub(crate) fn island_visual_surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let x = island.center.x + island.half_extents.x * normalized_offset.x;
    let z = island.center.z + island.half_extents.y * normalized_offset.y;

    Vec3::new(x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z)
}

pub(crate) fn island_silhouette_scale(island_index: usize, angle: f32) -> f32 {
    let phase = island_index as f32 * 0.73;
    (1.0 + 0.09 * (angle * 3.0 + phase).sin()
        + 0.055 * (angle * 7.0 - phase * 0.4).cos()
        + 0.032 * (angle * 11.0 + phase * 1.7).sin())
    .clamp(0.82, 1.18)
}

pub(crate) fn island_playable_silhouette_scale(island_index: usize, angle: f32) -> f32 {
    island_silhouette_scale(island_index, angle).min(1.0)
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
