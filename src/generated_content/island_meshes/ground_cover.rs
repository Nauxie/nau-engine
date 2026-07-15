use super::super::random_unit;
use super::constants::{
    GROUND_COVER_BLADES_PER_PATCH, INDICES_PER_GROUND_BLADE, VERTICES_PER_GROUND_BLADE,
};
use super::shape::island_playable_normalized_offset;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

pub(crate) fn island_ground_cover_mesh(
    island_index: usize,
    island: SkyIsland,
    patch_count: usize,
) -> Mesh {
    let blade_count = patch_count * GROUND_COVER_BLADES_PER_PATCH;
    let mut positions = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut normals = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut uvs = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut indices = Vec::with_capacity(blade_count * INDICES_PER_GROUND_BLADE);
    let seed = island_index as u32 * 41 + 503;

    for patch in 0..patch_count {
        let base_angle = random_unit(seed, patch as u32, 3) * std::f32::consts::TAU;
        let radius = random_unit(seed, patch as u32, 11).sqrt() * 0.90;
        let jitter = Vec2::new(
            (random_unit(seed, patch as u32, 17) - 0.5) * 0.08,
            (random_unit(seed, patch as u32, 23) - 0.5) * 0.08,
        );
        let normalized_offset = island_playable_normalized_offset(
            island,
            Vec2::new(base_angle.cos(), base_angle.sin()) * radius + jitter,
        );
        let x = island.center.x + normalized_offset.x * island.half_extents.x;
        let z = island.center.z + normalized_offset.y * island.half_extents.y;
        let surface_y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) + 0.08;

        for blade in 0..GROUND_COVER_BLADES_PER_PATCH {
            let blade_phase = base_angle
                + blade as f32 * std::f32::consts::TAU / GROUND_COVER_BLADES_PER_PATCH as f32;
            let width = 0.14 + random_unit(seed, patch as u32, 31 + blade as u32) * 0.15;
            let height = 0.72 + random_unit(seed, patch as u32, 43 + blade as u32) * 0.86;
            let lean = Vec3::new(blade_phase.cos(), 0.0, blade_phase.sin())
                * (0.1 + random_unit(seed, patch as u32, 53 + blade as u32) * 0.24);
            push_ground_cover_blade(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                Vec3::new(x, surface_y, z),
                blade_phase,
                width,
                height,
                lean,
                patch,
            );
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_ground_cover_blade(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    origin: Vec3,
    angle: f32,
    width: f32,
    height: f32,
    lean: Vec3,
    patch: usize,
) {
    let right = Vec3::new(angle.cos(), 0.0, angle.sin());
    let cross = Vec3::new(-angle.sin(), 0.0, angle.cos());
    let side = right * (width * 0.5);
    let mid_side = right * (width * 0.26);
    let mid = origin + Vec3::Y * (height * 0.54) + lean * 0.42;
    let tip = origin + Vec3::Y * height + lean;
    let leaflet_sign = if patch.is_multiple_of(2) { 1.0 } else { -1.0 };
    let leaflet = mid
        + cross * (leaflet_sign * width * 0.95)
        + right * (width * 0.10)
        + Vec3::Y * (height * 0.08);
    let blade_normal = Vec3::new(right.z * 0.35, 0.8, -right.x * 0.35).normalize();
    let leaflet_normal = (blade_normal + cross * (leaflet_sign * 0.18)).normalize();
    let start = positions.len() as u32;

    positions.extend([
        (origin - side).to_array(),
        (origin + side).to_array(),
        (mid - mid_side).to_array(),
        (mid + mid_side).to_array(),
        tip.to_array(),
        leaflet.to_array(),
    ]);
    normals.extend([
        blade_normal.to_array(),
        blade_normal.to_array(),
        blade_normal.to_array(),
        blade_normal.to_array(),
        blade_normal.to_array(),
        leaflet_normal.to_array(),
    ]);
    let uv_offset = if patch.is_multiple_of(2) { 0.0 } else { 0.5 };
    uvs.extend([
        [uv_offset, 1.0],
        [uv_offset + 0.42, 1.0],
        [uv_offset + 0.10, 0.46],
        [uv_offset + 0.32, 0.46],
        [uv_offset + 0.21, 0.0],
        [uv_offset + 0.44, 0.34],
    ]);
    indices.extend([
        start,
        start + 1,
        start + 2,
        start + 1,
        start + 3,
        start + 2,
        start + 2,
        start + 3,
        start + 4,
        start + 2,
        start + 3,
        start + 5,
    ]);
}
