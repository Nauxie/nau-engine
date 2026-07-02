use super::super::random_unit;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const ROCK_MESH_SEGMENTS: usize = 12;
pub(crate) const ROCK_MESH_RINGS: usize = 6;
pub(crate) const OBSTRUCTION_SPIRE_SEGMENTS: usize = 22;
pub(crate) const OBSTRUCTION_SPIRE_RINGS: usize = 12;
pub(crate) const OBSTRUCTION_SPIRE_RIB_COUNT: usize = 7;
pub(crate) const CLIFF_TOOTH_COUNT: usize = 9;
pub(crate) const CLIFF_TOOTH_TRIANGLES_PER_TOOTH: usize = 6;

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

pub(crate) fn obstruction_spire_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity(
        OBSTRUCTION_SPIRE_RINGS * OBSTRUCTION_SPIRE_SEGMENTS + 2 + OBSTRUCTION_SPIRE_RIB_COUNT * 8,
    );
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(
        (OBSTRUCTION_SPIRE_RINGS - 1) * OBSTRUCTION_SPIRE_SEGMENTS * 6
            + OBSTRUCTION_SPIRE_SEGMENTS * 6
            + OBSTRUCTION_SPIRE_RIB_COUNT * 12,
    );
    let height = height.max(1.0);
    let radius = radius.max(0.2);
    let lean = Vec2::new(
        (random_unit(seed, 3, 11) - 0.5) * radius * 0.42,
        (random_unit(seed, 5, 13) - 0.5) * radius * 0.42,
    );
    let stretch = Vec2::new(
        0.88 + random_unit(seed, 7, 17) * 0.28,
        0.82 + random_unit(seed, 9, 19) * 0.32,
    );

    let bottom_center = positions.len() as u32;
    positions.push([0.0, 0.0, 0.0]);
    normals.push(Vec3::NEG_Y.to_array());
    uvs.push([0.5, 0.0]);

    for ring in 0..OBSTRUCTION_SPIRE_RINGS {
        let t = ring as f32 / (OBSTRUCTION_SPIRE_RINGS - 1) as f32;
        let y = height * t;
        let taper = (1.0 - t).powf(1.35) * 0.78 + 0.16;
        let shelf = 1.0 + 0.11 * (t * std::f32::consts::TAU * 2.3 + seed as f32 * 0.013).sin();
        let center = lean * t.powf(1.45);

        for segment in 0..OBSTRUCTION_SPIRE_SEGMENTS {
            let phase = segment as f32 / OBSTRUCTION_SPIRE_SEGMENTS as f32 * std::f32::consts::TAU;
            let ridge = (phase * 3.0 + t * 2.8 + seed as f32 * 0.017).sin() * 0.12;
            let fracture = (random_unit(seed, ring as u32 * 37 + segment as u32, 29) - 0.5) * 0.18;
            let notch = 1.0 - 0.10 * (phase * 5.0 - t * 4.0 + seed as f32 * 0.007).cos().max(0.0);
            let radial = radius * taper * shelf * (1.0 + ridge + fracture) * notch;
            let x = center.x + phase.cos() * radial * stretch.x;
            let z = center.y + phase.sin() * radial * stretch.y;
            let normal = Vec3::new(
                phase.cos() / stretch.x.max(0.1),
                0.14 + (1.0 - t) * 0.24,
                phase.sin() / stretch.y.max(0.1),
            )
            .normalize();

            positions.push([x, y, z]);
            normals.push(normal.to_array());
            uvs.push([
                segment as f32 / OBSTRUCTION_SPIRE_SEGMENTS as f32,
                t.clamp(0.0, 1.0),
            ]);
        }
    }

    let first_ring = 1_u32;
    for segment in 0..OBSTRUCTION_SPIRE_SEGMENTS {
        let next = ((segment + 1) % OBSTRUCTION_SPIRE_SEGMENTS) as u32;
        indices.extend([
            bottom_center,
            first_ring + segment as u32,
            first_ring + next,
        ]);
    }

    for ring in 0..OBSTRUCTION_SPIRE_RINGS - 1 {
        let start = 1 + (ring * OBSTRUCTION_SPIRE_SEGMENTS) as u32;
        let next_start = 1 + ((ring + 1) * OBSTRUCTION_SPIRE_SEGMENTS) as u32;
        for segment in 0..OBSTRUCTION_SPIRE_SEGMENTS {
            let a = start + segment as u32;
            let b = start + ((segment + 1) % OBSTRUCTION_SPIRE_SEGMENTS) as u32;
            let c = next_start + segment as u32;
            let d = next_start + ((segment + 1) % OBSTRUCTION_SPIRE_SEGMENTS) as u32;
            indices.extend([a, c, b, b, c, d]);
        }
    }

    let top_center = positions.len() as u32;
    positions.push([lean.x, height + radius * 0.24, lean.y]);
    normals.push(Vec3::Y.to_array());
    uvs.push([0.5, 1.0]);
    let top_ring = 1 + ((OBSTRUCTION_SPIRE_RINGS - 1) * OBSTRUCTION_SPIRE_SEGMENTS) as u32;
    for segment in 0..OBSTRUCTION_SPIRE_SEGMENTS {
        let next = ((segment + 1) % OBSTRUCTION_SPIRE_SEGMENTS) as u32;
        indices.extend([top_center, top_ring + next, top_ring + segment as u32]);
    }

    append_spire_ribs(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        radius,
        height,
        seed,
    );

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn cliff_tooth_ridge_mesh(width: f32, height: f32, depth: f32, seed: u32) -> Mesh {
    let vertex_count = CLIFF_TOOTH_COUNT * CLIFF_TOOTH_TRIANGLES_PER_TOOTH * 3;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(vertex_count);
    let width = width.max(2.0);
    let height = height.max(1.0);
    let depth = depth.max(0.6);
    let spacing = width / CLIFF_TOOTH_COUNT as f32;

    for tooth in 0..CLIFF_TOOTH_COUNT {
        let tooth_seed = tooth as u32;
        let t = tooth as f32 / (CLIFF_TOOTH_COUNT - 1) as f32;
        let center_x =
            (t - 0.5) * width * 0.92 + (random_unit(seed, tooth_seed, 211) - 0.5) * spacing * 0.32;
        let half_width = spacing * (0.42 + random_unit(seed, tooth_seed, 223) * 0.24);
        let base_depth = depth * (0.72 + random_unit(seed, tooth_seed, 227) * 0.52);
        let root_y = (random_unit(seed, tooth_seed, 229) - 0.5) * height * 0.04;
        let tooth_height = height * (0.78 + random_unit(seed, tooth_seed, 233) * 0.36);
        let lean = Vec2::new(
            (random_unit(seed, tooth_seed, 239) - 0.5) * spacing * 0.56,
            (random_unit(seed, tooth_seed, 241) - 0.5) * base_depth * 0.42,
        );
        let skew = (random_unit(seed, tooth_seed, 251) - 0.5) * half_width * 0.44;

        let left_front = Vec3::new(center_x - half_width - skew, root_y, -base_depth * 0.55);
        let right_front = Vec3::new(
            center_x + half_width,
            root_y + (random_unit(seed, tooth_seed, 257) - 0.5) * height * 0.018,
            -base_depth * 0.46,
        );
        let right_back = Vec3::new(
            center_x + half_width * 0.72 + skew,
            root_y + random_unit(seed, tooth_seed, 263) * height * 0.018,
            base_depth * 0.54,
        );
        let left_back = Vec3::new(
            center_x - half_width * 0.82,
            root_y - random_unit(seed, tooth_seed, 269) * height * 0.016,
            base_depth * 0.46,
        );
        let tip = Vec3::new(center_x + lean.x, root_y + tooth_height, lean.y);

        append_flat_triangle(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            [left_front, right_front, right_back],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        );
        append_flat_triangle(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            [left_front, right_back, left_back],
            [[0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        );
        append_flat_triangle(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            [left_front, tip, right_front],
            [[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]],
        );
        append_flat_triangle(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            [right_front, tip, right_back],
            [[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]],
        );
        append_flat_triangle(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            [right_back, tip, left_back],
            [[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]],
        );
        append_flat_triangle(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            [left_back, tip, left_front],
            [[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]],
        );
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

fn append_flat_triangle(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    points: [Vec3; 3],
    triangle_uvs: [[f32; 2]; 3],
) {
    let start = positions.len() as u32;
    let normal = (points[1] - points[0])
        .cross(points[2] - points[0])
        .normalize();
    for (point, uv) in points.into_iter().zip(triangle_uvs) {
        positions.push(point.to_array());
        normals.push(normal.to_array());
        uvs.push(uv);
    }
    indices.extend([start, start + 1, start + 2]);
}

fn append_spire_ribs(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    radius: f32,
    height: f32,
    seed: u32,
) {
    for rib in 0..OBSTRUCTION_SPIRE_RIB_COUNT {
        let phase = rib as f32 / OBSTRUCTION_SPIRE_RIB_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, rib as u32, 151) * 0.4;
        let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
        let tangent = Vec3::new(-phase.sin(), 0.0, phase.cos());
        let base_radius = radius * (0.72 + random_unit(seed, rib as u32, 157) * 0.18);
        let root_reach = radius * (1.10 + random_unit(seed, rib as u32, 163) * 0.26);
        let rib_height = height * (0.34 + random_unit(seed, rib as u32, 167) * 0.18);
        let half_width = radius * (0.09 + random_unit(seed, rib as u32, 173) * 0.035);
        let base = outward * root_reach;
        let upper = outward * base_radius + Vec3::Y * rib_height;
        let start = positions.len() as u32;
        let normal = outward.normalize();

        for point in [
            base - tangent * half_width,
            base + tangent * half_width,
            upper - tangent * half_width * 0.42,
            upper + tangent * half_width * 0.42,
        ] {
            positions.push(point.to_array());
            normals.push(normal.to_array());
        }
        uvs.extend([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
        for point in [
            base - tangent * half_width,
            base + tangent * half_width,
            upper - tangent * half_width * 0.42,
            upper + tangent * half_width * 0.42,
        ] {
            positions.push(point.to_array());
            normals.push((-normal).to_array());
        }
        uvs.extend([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
        indices.extend([
            start,
            start + 2,
            start + 1,
            start + 1,
            start + 2,
            start + 3,
            start + 5,
            start + 6,
            start + 4,
            start + 7,
            start + 6,
            start + 5,
        ]);
    }
}
