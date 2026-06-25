use super::random_unit;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const TREE_CANOPY_LATITUDE_SEGMENTS: usize = 6;
pub(crate) const TREE_CANOPY_LONGITUDE_SEGMENTS: usize = 12;
pub(crate) const TREE_CANOPY_CARD_COUNT: usize = 12;
pub(crate) const TREE_TRUNK_SEGMENTS: usize = 8;
pub(crate) const TREE_BRANCH_COUNT: usize = 3;
pub(crate) const TREE_BRANCH_SEGMENTS: usize = 6;
pub(crate) const ROCK_MESH_SEGMENTS: usize = 12;
pub(crate) const ROCK_MESH_RINGS: usize = 6;
pub(crate) const CLOUD_BANK_LOBES: usize = 14;
pub(crate) const CLOUD_VEIL_LOBES: usize = 7;
pub(crate) const CLOUD_WISP_CARDS_PER_LOBE: usize = 2;
#[cfg(test)]
pub(crate) const DETAIL_CARD_VERTICES: usize = 8;

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
pub(crate) fn append_tapered_limb(
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

pub(crate) fn cloud_cluster_mesh(seed: u32, lobe_count: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for lobe in 0..lobe_count {
        let phase = lobe as f32 / lobe_count as f32 * std::f32::consts::TAU
            + random_unit(seed, lobe as u32, 5) * 0.8;
        let layer = lobe % 4;
        let layer_height = match layer {
            0 => -0.34,
            1 => -0.08,
            2 => 0.18,
            _ => 0.40,
        };
        let layer_spread = match layer {
            0 => 0.72,
            1 => 0.56,
            2 => 0.40,
            _ => 0.24,
        };
        let radius =
            0.36 + random_unit(seed, lobe as u32, 19) * 0.27 + if layer == 0 { 0.08 } else { 0.0 };
        let center = Vec3::new(
            phase.cos() * (0.18 + layer_spread * random_unit(seed, lobe as u32, 29)),
            layer_height + (random_unit(seed, lobe as u32, 41) - 0.5) * 0.16,
            phase.sin() * (0.14 + layer_spread * random_unit(seed, lobe as u32, 53) * 0.82),
        );
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            Vec3::new(
                radius * (1.20 + layer as f32 * 0.04),
                radius * (0.54 + layer as f32 * 0.03),
                radius * (0.82 + layer as f32 * 0.05),
            ),
            5,
            10,
            seed.wrapping_add(lobe as u32 * 101),
            0.15,
        );

        for card in 0..CLOUD_WISP_CARDS_PER_LOBE {
            let card_phase = phase
                + card as f32 / CLOUD_WISP_CARDS_PER_LOBE as f32 * 1.9
                + random_unit(seed, lobe as u32, 211 + card as u32) * 0.45;
            let outward = Vec3::new(
                card_phase.cos(),
                0.10 + layer as f32 * 0.025,
                card_phase.sin(),
            )
            .normalize();
            let tangent = Vec3::new(-card_phase.sin(), 0.0, card_phase.cos()).normalize();
            let up = (Vec3::Y * 0.78 + outward * 0.22).normalize();
            let card_center = center
                + outward
                    * radius
                    * (0.58 + random_unit(seed, lobe as u32, 223 + card as u32) * 0.22);
            let half_width =
                radius * (0.62 + random_unit(seed, lobe as u32, 229 + card as u32) * 0.22);
            let half_height =
                radius * (0.20 + random_unit(seed, lobe as u32, 233 + card as u32) * 0.10);
            append_double_sided_detail_card(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                card_center,
                tangent,
                up,
                half_width,
                half_height,
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
pub(crate) fn append_double_sided_detail_card(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    tangent: Vec3,
    up: Vec3,
    half_width: f32,
    half_height: f32,
) {
    let tangent = tangent.normalize_or_zero();
    let up = up.normalize_or_zero();
    if tangent.length_squared() <= 0.0001 || up.length_squared() <= 0.0001 {
        return;
    }
    let normal = tangent.cross(up).normalize_or_zero();
    if normal.length_squared() <= 0.0001 {
        return;
    }

    let side = tangent * half_width;
    let vertical = up * half_height;
    let card_positions = [
        center - side,
        center + vertical,
        center + side,
        center - vertical,
    ];
    let card_uvs = [[0.0, 0.5], [0.5, 0.0], [1.0, 0.5], [0.5, 1.0]];
    let start = positions.len() as u32;

    for position in card_positions {
        positions.push(position.to_array());
        normals.push(normal.to_array());
    }
    uvs.extend(card_uvs);
    for position in card_positions {
        positions.push(position.to_array());
        normals.push((-normal).to_array());
    }
    uvs.extend(card_uvs);

    indices.extend([
        start,
        start + 1,
        start + 2,
        start,
        start + 2,
        start + 3,
        start + 6,
        start + 5,
        start + 4,
        start + 7,
        start + 6,
        start + 4,
    ]);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_ellipsoid_lobe(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    radii: Vec3,
    latitude_segments: usize,
    longitude_segments: usize,
    seed: u32,
    noise_strength: f32,
) {
    let start = positions.len() as u32;

    for lat in 0..=latitude_segments {
        let theta = lat as f32 / latitude_segments as f32 * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        for lon in 0..=longitude_segments {
            let phi = lon as f32 / longitude_segments as f32 * std::f32::consts::TAU;
            let unit = Vec3::new(sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin());
            let noise =
                (random_unit(seed, lat as u32 * 31 + lon as u32, 83) - 0.5) * noise_strength;
            let position = center + unit * radii * (1.0 + noise);
            let normal =
                Vec3::new(unit.x / radii.x, unit.y / radii.y, unit.z / radii.z).normalize_or_zero();
            positions.push(position.to_array());
            normals.push(normal.to_array());
            uvs.push([
                lon as f32 / longitude_segments as f32,
                lat as f32 / latitude_segments as f32,
            ]);
        }
    }

    let stride = longitude_segments + 1;
    for lat in 0..latitude_segments {
        for lon in 0..longitude_segments {
            let a = start + (lat * stride + lon) as u32;
            let b = start + (lat * stride + lon + 1) as u32;
            let c = start + ((lat + 1) * stride + lon) as u32;
            let d = start + ((lat + 1) * stride + lon + 1) as u32;
            indices.extend([a, c, b, b, c, d]);
        }
    }
}

pub(crate) fn updraft_ribbon_mesh(radius: f32, height: f32, phase: f32) -> Mesh {
    const SEGMENTS: usize = 44;
    const STRANDS: f32 = 1.45;

    let width = (radius * 0.03).clamp(0.32, 0.65);
    let ribbon_radius = radius * 0.42;
    let mut positions = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut normals = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut uvs = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut indices = Vec::with_capacity(SEGMENTS * 6);

    for segment in 0..=SEGMENTS {
        let t = segment as f32 / SEGMENTS as f32;
        let angle = phase + t * std::f32::consts::TAU * STRANDS;
        let y = -height * 0.5 + t * height;
        let breathing = 1.0 + 0.08 * (angle * 2.0 + phase).sin();
        let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
        let center = radial * ribbon_radius * breathing + Vec3::Y * y;
        let side = radial * width;
        let normal = Vec3::new(radial.x * 0.32, 0.78, radial.z * 0.32).normalize();

        positions.extend([(center - side).to_array(), (center + side).to_array()]);
        normals.extend([normal.to_array(), normal.to_array()]);
        uvs.extend([[0.0, t], [1.0, t]]);

        if segment < SEGMENTS {
            let start = (segment * 2) as u32;
            indices.extend([start, start + 1, start + 2, start + 1, start + 3, start + 2]);
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

pub(crate) fn glider_airflow_trail_mesh() -> Mesh {
    let positions = vec![
        [-0.5, 0.0, -0.5],
        [0.5, 0.0, -0.5],
        [-0.14, 0.0, 0.5],
        [0.14, 0.0, 0.5],
    ];
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    let indices = vec![0, 1, 2, 1, 3, 2];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
