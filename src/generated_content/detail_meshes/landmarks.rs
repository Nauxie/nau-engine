use super::{super::random_unit, shared::append_ellipsoid_lobe};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const ROUTE_CAIRN_STONE_COUNT: usize = 5;
pub(crate) const LAUNCH_BEACON_CRYSTAL_COUNT: usize = 4;
pub(crate) const LANDING_GARDEN_MARKER_SEGMENTS: usize = 12;
pub(crate) const POND_SURFACE_SEGMENTS: usize = 32;

const LANDMARK_LOBE_LATITUDE_SEGMENTS: usize = 4;
const LANDMARK_LOBE_LONGITUDE_SEGMENTS: usize = 9;
const CRYSTAL_RING_SEGMENTS: usize = 6;

pub(crate) fn route_cairn_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    append_cairn_stones(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        radius,
        height,
        seed,
    );

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn launch_beacon_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    append_cairn_stones(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        radius,
        height * 0.50,
        seed,
    );

    for crystal in 0..LAUNCH_BEACON_CRYSTAL_COUNT {
        let phase = crystal as f32 / LAUNCH_BEACON_CRYSTAL_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, crystal as u32, 701) * 0.42;
        let lean = Vec3::new(phase.cos(), 0.26, phase.sin()).normalize();
        let base = Vec3::new(
            phase.cos() * radius * (0.16 + random_unit(seed, crystal as u32, 709) * 0.18),
            height * (-0.05 + crystal as f32 * 0.055),
            phase.sin() * radius * (0.16 + random_unit(seed, crystal as u32, 719) * 0.18),
        );
        append_crystal_shard(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            base,
            lean,
            radius * (0.13 + random_unit(seed, crystal as u32, 727) * 0.06),
            height * (0.56 + random_unit(seed, crystal as u32, 733) * 0.14),
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn landing_garden_marker_mesh(length: f32, width: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity((LANDING_GARDEN_MARKER_SEGMENTS + 1) * 3);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(LANDING_GARDEN_MARKER_SEGMENTS * 12);

    for segment in 0..=LANDING_GARDEN_MARKER_SEGMENTS {
        let t = segment as f32 / LANDING_GARDEN_MARKER_SEGMENTS as f32;
        let centered_t = t - 0.5;
        let edge_noise = random_unit(seed, segment as u32, 811) - 0.5;
        let half_width = width * (0.44 + edge_noise * 0.10);
        let x = centered_t * length;
        let arch = (1.0 - centered_t.abs() * 1.6).max(0.0);
        let center_y = 0.10 + arch.powf(1.8) * width * 0.26;
        let edge_y = 0.04 + (random_unit(seed, segment as u32, 823) - 0.5) * width * 0.035;
        let normal = Vec3::new(
            (random_unit(seed, segment as u32, 829) - 0.5) * 0.08,
            1.0,
            (random_unit(seed, segment as u32, 839) - 0.5) * 0.08,
        )
        .normalize();

        positions.extend([
            [x, edge_y, -half_width],
            [x, center_y, 0.0],
            [x, edge_y, half_width],
        ]);
        normals.extend([normal.to_array(); 3]);
        uvs.extend([[t, 0.0], [t, 0.5], [t, 1.0]]);
    }

    for segment in 0..LANDING_GARDEN_MARKER_SEGMENTS {
        let current = (segment * 3) as u32;
        let next = current + 3;
        indices.extend([
            current,
            next,
            current + 1,
            current + 1,
            next,
            next + 1,
            current + 1,
            next + 1,
            current + 2,
            current + 2,
            next + 1,
            next + 2,
        ]);
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn pond_surface_mesh(radius_x: f32, radius_z: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity(1 + POND_SURFACE_SEGMENTS * 2);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(POND_SURFACE_SEGMENTS * 9);

    positions.push([0.0, 0.0, 0.0]);
    normals.push(Vec3::Y.to_array());
    uvs.push([0.5, 0.5]);

    for ring in [0.48_f32, 1.0] {
        for segment in 0..POND_SURFACE_SEGMENTS {
            let angle = segment as f32 / POND_SURFACE_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge = 1.0
                + (random_unit(seed, segment as u32, 907) - 0.5) * 0.15 * ring
                + 0.035 * (angle * 5.0 + seed as f32 * 0.011).sin();
            let ripple = (angle * 4.0 + seed as f32 * 0.017).sin() * 0.012 * ring;
            let x = angle.cos() * radius_x * ring * edge;
            let z = angle.sin() * radius_z * ring * edge;

            positions.push([x, ripple, z]);
            normals.push(Vec3::Y.to_array());
            uvs.push([
                0.5 + angle.cos() * ring * 0.5,
                0.5 + angle.sin() * ring * 0.5,
            ]);
        }
    }

    let inner_index = |segment: usize| -> u32 { 1 + segment as u32 % POND_SURFACE_SEGMENTS as u32 };
    let outer_index = |segment: usize| -> u32 {
        1 + POND_SURFACE_SEGMENTS as u32 + segment as u32 % POND_SURFACE_SEGMENTS as u32
    };

    for segment in 0..POND_SURFACE_SEGMENTS {
        indices.extend([0, inner_index(segment), inner_index(segment + 1)]);
        indices.extend([
            inner_index(segment),
            outer_index(segment),
            inner_index(segment + 1),
            inner_index(segment + 1),
            outer_index(segment),
            outer_index(segment + 1),
        ]);
    }

    build_mesh(positions, normals, uvs, indices)
}

fn append_cairn_stones(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    radius: f32,
    height: f32,
    seed: u32,
) {
    let stone_height = height / (ROUTE_CAIRN_STONE_COUNT as f32 - 0.3);
    for stone in 0..ROUTE_CAIRN_STONE_COUNT {
        let t = stone as f32 / (ROUTE_CAIRN_STONE_COUNT - 1) as f32;
        let phase = random_unit(seed, stone as u32, 601) * std::f32::consts::TAU;
        let layer_radius = radius * (1.04 - t * 0.46);
        let center = Vec3::new(
            phase.cos() * radius * (0.08 + t * 0.06),
            -height * 0.5 + stone_height * (0.55 + stone as f32 * 0.95),
            phase.sin() * radius * (0.08 + t * 0.06),
        );

        append_ellipsoid_lobe(
            positions,
            normals,
            uvs,
            indices,
            center,
            Vec3::new(
                layer_radius * (0.92 + random_unit(seed, stone as u32, 613) * 0.18),
                stone_height * (0.42 + random_unit(seed, stone as u32, 617) * 0.18),
                layer_radius * (0.70 + random_unit(seed, stone as u32, 619) * 0.20),
            ),
            LANDMARK_LOBE_LATITUDE_SEGMENTS,
            LANDMARK_LOBE_LONGITUDE_SEGMENTS,
            seed.wrapping_add(stone as u32 * 83),
            0.24,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_crystal_shard(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    base: Vec3,
    lean: Vec3,
    radius: f32,
    height: f32,
) {
    let axis = (Vec3::Y + lean * 0.18).normalize_or_zero();
    if axis.length_squared() <= 0.0001 {
        return;
    }
    let side_seed = if axis.dot(Vec3::Y).abs() > 0.94 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let side = axis.cross(side_seed).normalize();
    let bitangent = side.cross(axis).normalize();
    let start = positions.len() as u32;
    let waist = base + axis * height * 0.36;
    let tip = base + axis * height;

    for (ring, (center, ring_radius)) in [(base, radius), (waist, radius * 0.58)]
        .into_iter()
        .enumerate()
    {
        for segment in 0..CRYSTAL_RING_SEGMENTS {
            let phase = segment as f32 / CRYSTAL_RING_SEGMENTS as f32 * std::f32::consts::TAU;
            let radial = side * phase.cos() + bitangent * phase.sin();
            positions.push((center + radial * ring_radius).to_array());
            normals.push(radial.normalize().to_array());
            uvs.push([
                segment as f32 / CRYSTAL_RING_SEGMENTS as f32,
                ring as f32 * 0.6,
            ]);
        }
    }

    let tip_index = positions.len() as u32;
    positions.push(tip.to_array());
    normals.push(axis.to_array());
    uvs.push([0.5, 1.0]);

    let bottom_center = positions.len() as u32;
    positions.push(base.to_array());
    normals.push((-axis).to_array());
    uvs.push([0.5, 0.0]);

    for segment in 0..CRYSTAL_RING_SEGMENTS {
        let next = (segment + 1) % CRYSTAL_RING_SEGMENTS;
        let base_current = start + segment as u32;
        let base_next = start + next as u32;
        let waist_current = start + CRYSTAL_RING_SEGMENTS as u32 + segment as u32;
        let waist_next = start + CRYSTAL_RING_SEGMENTS as u32 + next as u32;
        indices.extend([
            base_current,
            waist_current,
            base_next,
            base_next,
            waist_current,
            waist_next,
            waist_current,
            tip_index,
            waist_next,
            bottom_center,
            base_next,
            base_current,
        ]);
    }
}

fn build_mesh(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
