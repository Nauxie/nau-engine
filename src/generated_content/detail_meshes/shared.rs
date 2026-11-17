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
            // The UV seam has two vertices, but they must share geometry and
            // noise. Likewise every longitude meets at one coincident pole.
            let geometry_lon = if lat == 0 || lat == latitude_segments {
                0
            } else {
                lon % longitude_segments
            };
            let phi = geometry_lon as f32 / longitude_segments as f32 * std::f32::consts::TAU;
            let unit = if lat == 0 {
                Vec3::Y
            } else if lat == latitude_segments {
                Vec3::NEG_Y
            } else {
                Vec3::new(sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin())
            };
            let noise = (random_unit(seed, lat as u32 * 31 + geometry_lon as u32, 83) - 0.5)
                * noise_strength;
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
            // Latitude runs from the top down. Keep geometric face normals
            // pointing outward, matching the vertex normals and back-face culling.
            indices.extend([a, b, c, b, d, c]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ellipsoid_lobes_wind_outward_for_backface_culling() {
        for noise in [0.0, 0.12, 0.32] {
            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut indices = Vec::new();
            let center = Vec3::new(3.0, -2.0, 5.0);
            append_ellipsoid_lobe(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                center,
                Vec3::new(1.55, 1.1, 1.8),
                6,
                12,
                8_000,
                noise,
            );
            assert_eq!(positions.len(), 7 * 13);
            assert_eq!(indices.len(), 6 * 12 * 6);
            for lat in 0..=6 {
                let first = lat * 13;
                let last = first + 12;
                assert_eq!(positions[first], positions[last], "open seam at row {lat}");
                assert_eq!(normals[first], normals[last]);
                assert_eq!(uvs[first][0], 0.0);
                assert_eq!(uvs[last][0], 1.0);
            }
            for lon in 1..=12 {
                assert_eq!(positions[0], positions[lon], "split north pole");
                assert_eq!(
                    positions[6 * 13],
                    positions[6 * 13 + lon],
                    "split south pole"
                );
            }
            let mut facing_faces = 0;
            for [a, b, c] in indices.as_chunks::<3>().0 {
                let points = [*a, *b, *c].map(|i| Vec3::from_array(positions[i as usize]));
                let face = (points[1] - points[0]).cross(points[2] - points[0]);
                let normal = [*a, *b, *c]
                    .map(|i| Vec3::from_array(normals[i as usize]))
                    .into_iter()
                    .sum::<Vec3>();
                let alignment = face.dot(normal);
                // Pole vertices coincide for smooth lobes. Noisy poles can
                // form grazing triangles whose normal alignment is zero.
                if alignment.abs() <= 0.00001 {
                    continue;
                }
                assert!(alignment > 0.0, "inward lobe face at noise {noise}");
                let triangle_center = (points[0] + points[1] + points[2]) / 3.0;
                assert!(face.dot(triangle_center - center) > 0.0);
                facing_faces += 1;
            }
            assert!(facing_faces >= 120);
        }
    }
}
