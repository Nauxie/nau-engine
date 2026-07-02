use super::{
    super::random_unit,
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const CLOUD_BANK_LOBES: usize = 22;
pub(crate) const CLOUD_VEIL_LOBES: usize = 12;
pub(crate) const CLOUD_WISP_CARDS_PER_LOBE: usize = 5;
pub(crate) const CLOUD_FILAMENT_RIBBONS_PER_LOBE: usize = 4;
const CLOUD_FILAMENT_RIBBON_SEGMENTS: usize = 6;
const CLOUD_VERTICAL_LAYERS: usize = 7;
#[cfg(test)]
pub(crate) const CLOUD_FILAMENT_RIBBON_VERTICES: usize = (CLOUD_FILAMENT_RIBBON_SEGMENTS + 1) * 4;

pub(crate) fn cloud_filament_ribbon_detail_count(lobe_count: usize) -> usize {
    lobe_count * CLOUD_FILAMENT_RIBBONS_PER_LOBE
}

pub(crate) fn cloud_cluster_mesh(seed: u32, lobe_count: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    for lobe in 0..lobe_count {
        let phase = lobe as f32 / lobe_count as f32 * std::f32::consts::TAU
            + random_unit(seed, lobe as u32, 5) * 0.8;
        let layer = lobe % CLOUD_VERTICAL_LAYERS;
        let layer_height = match layer {
            0 => -0.68,
            1 => -0.42,
            2 => -0.16,
            3 => 0.10,
            4 => 0.34,
            5 => 0.56,
            _ => 0.74,
        };
        let layer_spread = match layer {
            0 => 0.98,
            1 => 0.82,
            2 => 0.66,
            3 => 0.52,
            4 => 0.38,
            5 => 0.27,
            _ => 0.18,
        };
        let radius =
            0.40 + random_unit(seed, lobe as u32, 19) * 0.30 + if layer <= 2 { 0.10 } else { 0.0 };
        let center = Vec3::new(
            phase.cos() * (0.18 + layer_spread * random_unit(seed, lobe as u32, 29)),
            layer_height + (random_unit(seed, lobe as u32, 41) - 0.5) * 0.20,
            phase.sin() * (0.18 + layer_spread * random_unit(seed, lobe as u32, 53) * 0.96),
        );
        let lobe_color = cloud_lobe_color(seed, lobe as u32, layer, false);
        let lobe_start = positions.len();
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            Vec3::new(
                radius * (1.34 + layer as f32 * 0.025),
                radius * (0.58 + layer as f32 * 0.030),
                radius * (0.92 + layer as f32 * 0.045),
            ),
            5,
            10,
            seed.wrapping_add(lobe as u32 * 101),
            0.18,
        );
        colors.extend(std::iter::repeat_n(
            lobe_color,
            positions.len() - lobe_start,
        ));

        for card in 0..CLOUD_WISP_CARDS_PER_LOBE {
            let lower_depth_card = card == CLOUD_WISP_CARDS_PER_LOBE - 1;
            let card_phase = phase
                + card as f32 / CLOUD_WISP_CARDS_PER_LOBE as f32 * 2.35
                + random_unit(seed, lobe as u32, 211 + card as u32) * 0.45;
            let outward = if lower_depth_card {
                Vec3::new(
                    card_phase.cos() * 0.86,
                    -0.20 - layer as f32 * 0.020,
                    card_phase.sin() * 1.18,
                )
            } else {
                Vec3::new(
                    card_phase.cos(),
                    0.12 + layer as f32 * 0.024,
                    card_phase.sin(),
                )
            }
            .normalize();
            let tangent = Vec3::new(-card_phase.sin(), 0.0, card_phase.cos()).normalize();
            let up = if lower_depth_card {
                (Vec3::Y * 0.55 - outward * 0.25).normalize()
            } else {
                (Vec3::Y * 0.78 + outward * 0.22).normalize()
            };
            let card_center = if lower_depth_card {
                center
                    + outward
                        * radius
                        * (0.84 + random_unit(seed, lobe as u32, 223 + card as u32) * 0.28)
                    - Vec3::Y
                        * radius
                        * (0.22 + random_unit(seed, lobe as u32, 239 + card as u32) * 0.14)
            } else {
                center
                    + outward
                        * radius
                        * (0.62 + random_unit(seed, lobe as u32, 223 + card as u32) * 0.26)
            };
            let half_width = radius
                * if lower_depth_card {
                    0.82 + random_unit(seed, lobe as u32, 229 + card as u32) * 0.22
                } else {
                    0.68 + random_unit(seed, lobe as u32, 229 + card as u32) * 0.25
                };
            let half_height = radius
                * if lower_depth_card {
                    0.32 + random_unit(seed, lobe as u32, 233 + card as u32) * 0.14
                } else {
                    0.24 + random_unit(seed, lobe as u32, 233 + card as u32) * 0.12
                };
            let card_color = cloud_lobe_color(seed, lobe as u32 + card as u32 * 17, layer, true);
            let card_start = positions.len();
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
            colors.extend(std::iter::repeat_n(
                card_color,
                positions.len() - card_start,
            ));
        }

        for ribbon in 0..CLOUD_FILAMENT_RIBBONS_PER_LOBE {
            append_cloud_filament_ribbon(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut colors,
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn cloud_lobe_color(seed: u32, lobe: u32, layer: usize, wisp: bool) -> [f32; 4] {
    let layer_t = layer as f32 / (CLOUD_VERTICAL_LAYERS - 1) as f32;
    let sun_wash = Vec3::new(1.0, 0.88, 0.66);
    let vapor = Vec3::new(0.78, 0.88, 1.0);
    let lavender_shadow = Vec3::new(0.58, 0.66, 0.82);
    let warm_variation = random_unit(seed, lobe, 401) * 0.18;
    let cool_variation = random_unit(seed, lobe, 409) * 0.14;
    let color = vapor
        .lerp(
            sun_wash,
            (0.24 + layer_t * 0.42 + warm_variation).clamp(0.0, 1.0),
        )
        .lerp(
            lavender_shadow,
            ((1.0 - layer_t) * 0.22 + cool_variation).clamp(0.0, 0.48),
        );
    let alpha = if wisp {
        0.34 + layer_t * 0.07
    } else {
        0.50 + layer_t * 0.10
    };

    [
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
        alpha,
    ]
}

#[allow(clippy::too_many_arguments)]
fn append_cloud_filament_ribbon(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
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
        let color = cloud_lobe_color(seed, lobe + segment as u32 * 13 + ribbon * 29, 2, true);
        colors.extend([color, color]);
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
        colors.push(colors[start as usize + index]);
    }
    for segment in 0..CLOUD_FILAMENT_RIBBON_SEGMENTS {
        let a = reverse_start + (segment * 2) as u32;
        let b = a + 1;
        let c = a + 2;
        let d = a + 3;
        indices.extend([b, c, a, d, c, b]);
    }
}
