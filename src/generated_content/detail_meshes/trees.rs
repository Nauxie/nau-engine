use super::{
    super::random_unit,
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const TREE_CANOPY_LATITUDE_SEGMENTS: usize = 6;
pub(crate) const TREE_CANOPY_LONGITUDE_SEGMENTS: usize = 12;
pub(crate) const TREE_CANOPY_CARD_COUNT: usize = 18;
pub(crate) const TREE_TRUNK_SEGMENTS: usize = 10;
pub(crate) const TREE_TRUNK_RING_COUNT: usize = 5;
pub(crate) const TREE_BRANCH_COUNT: usize = 4;
pub(crate) const TREE_BRANCH_SEGMENTS: usize = 8;
pub(crate) const TREE_ROOT_FLARE_COUNT: usize = 5;
pub(crate) const TREE_ROOT_FLARE_SEGMENTS: usize = 8;

pub(crate) fn tree_trunk_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let trunk_vertices = TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT + 2;
    let branch_vertices = TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2;
    let root_vertices = TREE_ROOT_FLARE_COUNT * TREE_ROOT_FLARE_SEGMENTS * 2;
    let mut positions = Vec::with_capacity(trunk_vertices + branch_vertices + root_vertices);
    let mut normals = Vec::with_capacity(trunk_vertices + branch_vertices + root_vertices);
    let mut uvs = Vec::with_capacity(trunk_vertices + branch_vertices + root_vertices);
    let mut indices = Vec::with_capacity(
        TREE_TRUNK_SEGMENTS * 6 * (TREE_TRUNK_RING_COUNT + 1)
            + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 6
            + TREE_ROOT_FLARE_COUNT * TREE_ROOT_FLARE_SEGMENTS * 6,
    );
    let bend = Vec2::new(
        (random_unit(seed, 3, 11) - 0.5) * radius * 0.95,
        (random_unit(seed, 7, 17) - 0.5) * radius * 0.95,
    );
    let rings = [
        (-0.5, radius * 1.42, Vec2::ZERO),
        (-0.24, radius * 1.06, bend * 0.22),
        (0.04, radius * 0.84, bend * 0.48),
        (0.30, radius * 0.66, bend * 0.74),
        (0.5, radius * 0.48, bend),
    ];

    for (ring_index, (height_factor, ring_radius, center_offset)) in rings.into_iter().enumerate() {
        for segment in 0..TREE_TRUNK_SEGMENTS {
            let phase = segment as f32 / TREE_TRUNK_SEGMENTS as f32 * std::f32::consts::TAU;
            let bark_noise = 0.88 + random_unit(seed, segment as u32, ring_index as u32) * 0.24;
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

    for ring in 0..TREE_TRUNK_RING_COUNT - 1 {
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

    let top_start = ((TREE_TRUNK_RING_COUNT - 1) * TREE_TRUNK_SEGMENTS) as u32;
    for segment in 0..TREE_TRUNK_SEGMENTS {
        let next = ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
        indices.extend([bottom_center, segment as u32, next]);
        indices.extend([top_center, top_start + next, top_start + segment as u32]);
    }

    for branch in 0..TREE_BRANCH_COUNT {
        let height_factor = -0.03 + branch as f32 * 0.14;
        let branch_phase = branch as f32 / TREE_BRANCH_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, branch as u32, 89) * 0.55;
        let start = Vec3::new(
            bend.x * (height_factor + 0.5).clamp(0.0, 1.0),
            height * height_factor,
            bend.y * (height_factor + 0.5).clamp(0.0, 1.0),
        );
        let reach = radius * (3.0 + random_unit(seed, branch as u32, 97) * 1.05);
        let lift = height * (0.10 + random_unit(seed, branch as u32, 107) * 0.06);
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
            TREE_BRANCH_SEGMENTS,
        );
    }

    for root in 0..TREE_ROOT_FLARE_COUNT {
        let root_phase = root as f32 / TREE_ROOT_FLARE_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, root as u32, 211) * 0.28;
        let direction = Vec3::new(root_phase.cos(), -0.18, root_phase.sin()).normalize();
        let start = Vec3::new(
            root_phase.cos() * radius * 0.48,
            -height * 0.44,
            root_phase.sin() * radius * 0.48,
        );
        let reach = radius * (2.10 + random_unit(seed, root as u32, 223) * 0.70);
        let end = start + direction * reach + Vec3::Y * (-height * 0.035);
        append_tapered_limb(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            start,
            end,
            radius * 0.30,
            radius * 0.08,
            TREE_ROOT_FLARE_SEGMENTS,
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
        let lower_skirt = card >= TREE_CANOPY_CARD_COUNT.saturating_sub(6);
        let phase = card as f32 / TREE_CANOPY_CARD_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, card as u32, 151) * 0.24;
        let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
        let tangent = Vec3::new(-phase.sin(), 0.0, phase.cos()).normalize();
        let up = if lower_skirt {
            (Vec3::Y * 0.62 - outward * 0.26).normalize()
        } else {
            (Vec3::Y + outward * 0.16).normalize()
        };
        let center = Vec3::new(
            outward.x
                * radius
                * if lower_skirt {
                    0.74 + random_unit(seed, card as u32, 163) * 0.16
                } else {
                    0.58 + random_unit(seed, card as u32, 163) * 0.22
                },
            radius
                * if lower_skirt {
                    -0.34 + random_unit(seed, card as u32, 167) * 0.16
                } else {
                    -0.08 + random_unit(seed, card as u32, 167) * 0.34
                },
            outward.z
                * radius
                * if lower_skirt {
                    0.70 + random_unit(seed, card as u32, 173) * 0.16
                } else {
                    0.54 + random_unit(seed, card as u32, 173) * 0.20
                },
        );
        let half_width = radius
            * if lower_skirt {
                0.18 + random_unit(seed, card as u32, 179) * 0.07
            } else {
                0.20 + random_unit(seed, card as u32, 179) * 0.08
            };
        let half_height = radius
            * if lower_skirt {
                0.36 + random_unit(seed, card as u32, 181) * 0.14
            } else {
                0.28 + random_unit(seed, card as u32, 181) * 0.12
            };
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
    radial_segments: usize,
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
        for segment in 0..radial_segments {
            let phase = segment as f32 / radial_segments as f32 * std::f32::consts::TAU;
            let radial = side * phase.cos() + bitangent * phase.sin();
            positions.push((center + radial * radius).to_array());
            normals.push(radial.normalize().to_array());
            uvs.push([segment as f32 / radial_segments as f32, ring as f32]);
        }
    }

    for segment in 0..radial_segments {
        let a = first + segment as u32;
        let b = first + ((segment + 1) % radial_segments) as u32;
        let c = first + radial_segments as u32 + segment as u32;
        let d = first + radial_segments as u32 + ((segment + 1) % radial_segments) as u32;
        indices.extend([a, c, b, b, c, d]);
    }
}
