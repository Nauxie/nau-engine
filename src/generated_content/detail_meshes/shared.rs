use super::super::random_unit;
use bevy::prelude::*;

#[cfg(test)]
pub(crate) const DETAIL_CARD_VERTICES: usize = 8;

#[allow(clippy::too_many_arguments)]
pub(super) fn append_double_sided_detail_card(
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
pub(super) fn append_ellipsoid_lobe(
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
