use super::super::random_unit;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const ROCK_MESH_SEGMENTS: usize = 12;
pub(crate) const ROCK_MESH_RINGS: usize = 6;

pub(crate) fn rock_scatter_mesh(radius: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity(ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2);
    let mut normals = Vec::with_capacity(ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2);
    let mut uvs = Vec::with_capacity(ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2);
    let mut indices =
        Vec::with_capacity((ROCK_MESH_RINGS - 1) * ROCK_MESH_SEGMENTS * 6 + ROCK_MESH_SEGMENTS * 6);
    let stretch = Vec2::new(
        0.88 + random_unit(seed, 5, 17) * 0.34,
        0.76 + random_unit(seed, 7, 23) * 0.28,
    );
    let ring_profiles = [
        (-0.46, 0.72),
        (-0.28, 1.04),
        (-0.06, 1.16),
        (0.18, 0.98),
        (0.38, 0.72),
        (0.54, 0.38),
    ];

    let bottom_center = positions.len() as u32;
    positions.push([0.0, radius * -0.5, 0.0]);
    normals.push(Vec3::NEG_Y.to_array());
    uvs.push([0.5, 0.0]);

    for (ring_index, (height_factor, ring_radius)) in ring_profiles.into_iter().enumerate() {
        let phase_offset = random_unit(seed, ring_index as u32, 31) * 0.2;
        for segment in 0..ROCK_MESH_SEGMENTS {
            let phase = segment as f32 / ROCK_MESH_SEGMENTS as f32 * std::f32::consts::TAU;
            let ridge = (phase * 3.0 + phase_offset).sin() * 0.09;
            let chip = (random_unit(seed, segment as u32, ring_index as u32 + 53) - 0.5) * 0.24;
            let radial = radius * ring_radius * (1.0 + ridge + chip);
            let x = phase.cos() * radial * stretch.x;
            let z = phase.sin() * radial * stretch.y;
            let y = radius * height_factor;
            let normal = Vec3::new(
                phase.cos() / stretch.x.max(0.1),
                0.28 + height_factor * 0.35,
                phase.sin() / stretch.y.max(0.1),
            )
            .normalize();

            positions.push([x, y, z]);
            normals.push(normal.to_array());
            uvs.push([
                segment as f32 / ROCK_MESH_SEGMENTS as f32,
                ring_index as f32 / (ROCK_MESH_RINGS - 1) as f32,
            ]);
        }
    }

    let top_center = positions.len() as u32;
    let top_offset = Vec2::new(
        (random_unit(seed, 97, 11) - 0.5) * radius * 0.16,
        (random_unit(seed, 101, 13) - 0.5) * radius * 0.16,
    );
    positions.push([top_offset.x, radius * 0.68, top_offset.y]);
    normals.push(Vec3::Y.to_array());
    uvs.push([0.5, 1.0]);

    let first_ring = 1_u32;
    for segment in 0..ROCK_MESH_SEGMENTS {
        let next = ((segment + 1) % ROCK_MESH_SEGMENTS) as u32;
        indices.extend([
            bottom_center,
            first_ring + segment as u32,
            first_ring + next,
        ]);
    }

    for ring in 0..ROCK_MESH_RINGS - 1 {
        let start = 1 + (ring * ROCK_MESH_SEGMENTS) as u32;
        let next_start = 1 + ((ring + 1) * ROCK_MESH_SEGMENTS) as u32;
        for segment in 0..ROCK_MESH_SEGMENTS {
            let a = start + segment as u32;
            let b = start + ((segment + 1) % ROCK_MESH_SEGMENTS) as u32;
            let c = next_start + segment as u32;
            let d = next_start + ((segment + 1) % ROCK_MESH_SEGMENTS) as u32;
            indices.extend([a, c, b, b, c, d]);
        }
    }

    let top_ring = 1 + ((ROCK_MESH_RINGS - 1) * ROCK_MESH_SEGMENTS) as u32;
    for segment in 0..ROCK_MESH_SEGMENTS {
        let next = ((segment + 1) % ROCK_MESH_SEGMENTS) as u32;
        indices.extend([top_center, top_ring + next, top_ring + segment as u32]);
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
