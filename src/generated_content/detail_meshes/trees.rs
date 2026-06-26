use super::{
    super::random_unit,
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const TREE_CANOPY_LATITUDE_SEGMENTS: usize = 6;
pub(crate) const TREE_CANOPY_LONGITUDE_SEGMENTS: usize = 12;
pub(crate) const TREE_CANOPY_CARD_COUNT: usize = 12;
pub(crate) const TREE_TRUNK_SEGMENTS: usize = 8;
pub(crate) const TREE_BRANCH_COUNT: usize = 3;
pub(crate) const TREE_BRANCH_SEGMENTS: usize = 6;

pub(crate) fn tree_trunk_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let trunk_vertices = TREE_TRUNK_SEGMENTS * 3 + 2;
    let branch_vertices = TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2;
    let mut positions = Vec::with_capacity(trunk_vertices + branch_vertices);
    let mut normals = Vec::with_capacity(trunk_vertices + branch_vertices);
    let mut uvs = Vec::with_capacity(trunk_vertices + branch_vertices);
    let mut indices =
        Vec::with_capacity(TREE_TRUNK_SEGMENTS * 18 + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 6);
    let bend = Vec2::new(
        (random_unit(seed, 3, 11) - 0.5) * radius * 0.95,
        (random_unit(seed, 7, 17) - 0.5) * radius * 0.95,
    );
    let rings = [
        (-0.5, radius * 1.18, Vec2::ZERO),
        (0.0, radius * 0.96, bend * 0.42),
        (0.5, radius * 0.68, bend),
    ];

    for (ring_index, (height_factor, ring_radius, center_offset)) in rings.into_iter().enumerate() {
        for segment in 0..TREE_TRUNK_SEGMENTS {
            let phase = segment as f32 / TREE_TRUNK_SEGMENTS as f32 * std::f32::consts::TAU;
            let bark_noise = 0.9 + random_unit(seed, segment as u32, ring_index as u32) * 0.2;
            let x = center_offset.x + phase.cos() * ring_radius * bark_noise;
            let z = center_offset.y + phase.sin() * ring_radius * bark_noise;
            positions.push([x, height * height_factor, z]);
            normals.push(
                Vec3::new(phase.cos(), 0.16, phase.sin())
                    .normalize()
                    .to_array(),
            );
            uvs.push([
                segment as f32 / TREE_TRUNK_SEGMENTS as f32,
                ring_index as f32 / 2.0,
            ]);
        }
    }

    for ring in 0..2 {
        let start = (ring * TREE_TRUNK_SEGMENTS) as u32;
        let next = ((ring + 1) * TREE_TRUNK_SEGMENTS) as u32;
        for segment in 0..TREE_TRUNK_SEGMENTS {
            let a = start + segment as u32;
            let b = start + ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
            let c = next + segment as u32;
            let d = next + ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
            indices.extend([a, c, b, b, c, d]);
        }
    }

    let bottom_center = positions.len() as u32;
    positions.push([0.0, -height * 0.5, 0.0]);
    normals.push(Vec3::NEG_Y.to_array());
    uvs.push([0.5, 0.5]);
    let top_center = positions.len() as u32;
    positions.push([bend.x, height * 0.5, bend.y]);
    normals.push(Vec3::Y.to_array());
    uvs.push([0.5, 0.5]);

    for segment in 0..TREE_TRUNK_SEGMENTS {
        let next = ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
        indices.extend([bottom_center, segment as u32, next]);
        let top_start = (2 * TREE_TRUNK_SEGMENTS) as u32;
        indices.extend([top_center, top_start + next, top_start + segment as u32]);
    }

    for branch in 0..TREE_BRANCH_COUNT {
        let height_factor = -0.08 + branch as f32 * 0.23;
        let branch_phase = branch as f32 / TREE_BRANCH_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, branch as u32, 89) * 0.55;
        let start = Vec3::new(
            bend.x * (height_factor + 0.5),
            height * height_factor,
            bend.y * (height_factor + 0.5),
        );
        let reach = radius * (2.6 + random_unit(seed, branch as u32, 97) * 0.85);
        let lift = height * (0.08 + random_unit(seed, branch as u32, 107) * 0.05);
        let end = start + Vec3::new(branch_phase.cos() * reach, lift, branch_phase.sin() * reach);
        append_tapered_limb(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            start,
            end,
            radius * 0.34,
            radius * 0.12,
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

pub(crate) fn tree_canopy_mesh(radius: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    append_ellipsoid_lobe(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::ZERO,
        Vec3::new(radius * 1.08, radius * 0.82, radius),
        TREE_CANOPY_LATITUDE_SEGMENTS,
        TREE_CANOPY_LONGITUDE_SEGMENTS,
        seed,
        0.22,
    );

    for lobe in 0..5 {
        let phase =
            lobe as f32 / 5.0 * std::f32::consts::TAU + random_unit(seed, lobe as u32, 71) * 0.45;
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(
                phase.cos() * radius * (0.30 + random_unit(seed, lobe as u32, 83) * 0.12),
                radius * (-0.02 + lobe as f32 * 0.035),
                phase.sin() * radius * (0.26 + random_unit(seed, lobe as u32, 97) * 0.10),
            ),
            Vec3::new(radius * 0.58, radius * 0.50, radius * 0.54),
            4,
            8,
            seed.wrapping_add(100 + lobe as u32 * 17),
            0.18,
        );
    }

    for card in 0..TREE_CANOPY_CARD_COUNT {
        let phase = card as f32 / TREE_CANOPY_CARD_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, card as u32, 151) * 0.24;
        let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
        let tangent = Vec3::new(-phase.sin(), 0.0, phase.cos()).normalize();
        let up = (Vec3::Y + outward * 0.16).normalize();
        let center = Vec3::new(
            outward.x * radius * (0.58 + random_unit(seed, card as u32, 163) * 0.22),
            radius * (-0.08 + random_unit(seed, card as u32, 167) * 0.34),
            outward.z * radius * (0.54 + random_unit(seed, card as u32, 173) * 0.20),
        );
        let half_width = radius * (0.20 + random_unit(seed, card as u32, 179) * 0.08);
        let half_height = radius * (0.28 + random_unit(seed, card as u32, 181) * 0.12);
        append_double_sided_detail_card(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            tangent,
            up,
            half_width,
            half_height,
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

#[allow(clippy::too_many_arguments)]
fn append_tapered_limb(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    start: Vec3,
    end: Vec3,
    base_radius: f32,
    tip_radius: f32,
) {
    let axis = (end - start).normalize_or_zero();
    if axis.length_squared() <= 0.0001 {
        return;
    }
    let side_seed = if axis.dot(Vec3::Y).abs() > 0.92 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let side = axis.cross(side_seed).normalize();
    let bitangent = side.cross(axis).normalize();
    let first = positions.len() as u32;

    for (ring, (center, radius)) in [(start, base_radius), (end, tip_radius)]
        .into_iter()
        .enumerate()
    {
        for segment in 0..TREE_BRANCH_SEGMENTS {
            let phase = segment as f32 / TREE_BRANCH_SEGMENTS as f32 * std::f32::consts::TAU;
            let radial = side * phase.cos() + bitangent * phase.sin();
            positions.push((center + radial * radius).to_array());
            normals.push(radial.normalize().to_array());
            uvs.push([segment as f32 / TREE_BRANCH_SEGMENTS as f32, ring as f32]);
        }
    }

    for segment in 0..TREE_BRANCH_SEGMENTS {
        let a = first + segment as u32;
        let b = first + ((segment + 1) % TREE_BRANCH_SEGMENTS) as u32;
        let c = first + TREE_BRANCH_SEGMENTS as u32 + segment as u32;
        let d = first + TREE_BRANCH_SEGMENTS as u32 + ((segment + 1) % TREE_BRANCH_SEGMENTS) as u32;
        indices.extend([a, c, b, b, c, d]);
    }
}
