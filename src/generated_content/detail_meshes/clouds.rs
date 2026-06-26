use super::{
    super::random_unit,
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const CLOUD_BANK_LOBES: usize = 18;
pub(crate) const CLOUD_VEIL_LOBES: usize = 9;
pub(crate) const CLOUD_WISP_CARDS_PER_LOBE: usize = 3;
pub(crate) const CLOUD_FILAMENT_RIBBONS_PER_LOBE: usize = 3;
const CLOUD_FILAMENT_RIBBON_SEGMENTS: usize = 5;
#[cfg(test)]
pub(crate) const CLOUD_FILAMENT_RIBBON_VERTICES: usize = (CLOUD_FILAMENT_RIBBON_SEGMENTS + 1) * 4;

pub(crate) fn cloud_filament_ribbon_detail_count(lobe_count: usize) -> usize {
    lobe_count * CLOUD_FILAMENT_RIBBONS_PER_LOBE
}

pub(crate) fn cloud_cluster_mesh(seed: u32, lobe_count: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for lobe in 0..lobe_count {
        let phase = lobe as f32 / lobe_count as f32 * std::f32::consts::TAU
            + random_unit(seed, lobe as u32, 5) * 0.8;
        let layer = lobe % 5;
        let layer_height = match layer {
            0 => -0.52,
            1 => -0.22,
            2 => 0.08,
            3 => 0.34,
            _ => 0.60,
        };
        let layer_spread = match layer {
            0 => 0.86,
            1 => 0.68,
            2 => 0.52,
            3 => 0.36,
            _ => 0.22,
        };
        let radius =
            0.36 + random_unit(seed, lobe as u32, 19) * 0.27 + if layer <= 1 { 0.08 } else { 0.0 };
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
                radius * (1.24 + layer as f32 * 0.035),
                radius * (0.54 + layer as f32 * 0.035),
                radius * (0.84 + layer as f32 * 0.05),
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

        for ribbon in 0..CLOUD_FILAMENT_RIBBONS_PER_LOBE {
            append_cloud_filament_ribbon(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                center,
                phase,
                radius,
                seed,
                lobe as u32,
                ribbon as u32,
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
fn append_cloud_filament_ribbon(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    phase: f32,
    radius: f32,
    seed: u32,
    lobe: u32,
    ribbon: u32,
) {
    let sweep_phase = phase
        + ribbon as f32 / CLOUD_FILAMENT_RIBBONS_PER_LOBE as f32 * std::f32::consts::PI
        + random_unit(seed, lobe, 307 + ribbon) * 0.42;
    let tangent = Vec3::new(-sweep_phase.sin(), 0.08, sweep_phase.cos()).normalize_or_zero();
    let outward = Vec3::new(sweep_phase.cos(), 0.0, sweep_phase.sin()).normalize_or_zero();
    let normal = tangent.cross(Vec3::Y).normalize_or_zero();
    if tangent.length_squared() <= 0.0001
        || outward.length_squared() <= 0.0001
        || normal.length_squared() <= 0.0001
    {
        return;
    }

    let start = positions.len() as u32;
    for segment in 0..=CLOUD_FILAMENT_RIBBON_SEGMENTS {
        let t = segment as f32 / CLOUD_FILAMENT_RIBBON_SEGMENTS as f32;
        let centered_t = t - 0.5;
        let curl = (t * std::f32::consts::TAU + random_unit(seed, lobe, 331 + ribbon) * 1.4).sin();
        let local_center = center
            + tangent * centered_t * radius * (1.15 + random_unit(seed, lobe, 337 + ribbon) * 0.35)
            + outward * curl * radius * 0.14
            + Vec3::Y * (curl * radius * 0.05 + centered_t * radius * 0.10);
        let width = radius
            * (0.030 + random_unit(seed, lobe + segment as u32, 347 + ribbon) * 0.012)
            * (1.0 - centered_t.abs() * 0.35);
        let side = outward * width;
        let uv_y = t;

        positions.extend([
            (local_center - side).to_array(),
            (local_center + side).to_array(),
        ]);
        normals.extend([normal.to_array(), normal.to_array()]);
        uvs.extend([[0.0, uv_y], [1.0, uv_y]]);
    }

    for segment in 0..CLOUD_FILAMENT_RIBBON_SEGMENTS {
        let a = start + (segment * 2) as u32;
        let b = a + 1;
        let c = a + 2;
        let d = a + 3;
        indices.extend([a, c, b, b, c, d]);
    }

    let reverse_start = positions.len() as u32;
    let original_vertices = (CLOUD_FILAMENT_RIBBON_SEGMENTS + 1) * 2;
    for index in 0..original_vertices {
        positions.push(positions[start as usize + index]);
        normals.push((-normal).to_array());
        uvs.push(uvs[start as usize + index]);
    }
    for segment in 0..CLOUD_FILAMENT_RIBBON_SEGMENTS {
        let a = reverse_start + (segment * 2) as u32;
        let b = a + 1;
        let c = a + 2;
        let d = a + 3;
        indices.extend([b, c, a, d, c, b]);
    }
}
