use bevy::prelude::*;

pub(crate) fn smooth_normals_from_triangles(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Vec<[f32; 3]> {
    smooth_normals_from_triangles_oriented(positions, indices, Vec3::Y, true)
}

pub(crate) fn smooth_normals_from_triangles_oriented(
    positions: &[[f32; 3]],
    indices: &[u32],
    fallback: Vec3,
    force_positive_y: bool,
) -> Vec<[f32; 3]> {
    let mut normals = vec![Vec3::ZERO; positions.len()];
    let fallback = fallback.normalize_or_zero();
    let fallback = if fallback.length_squared() <= f32::EPSILON {
        Vec3::Y
    } else {
        fallback
    };

    for triangle in indices.chunks_exact(3) {
        let a_index = triangle[0] as usize;
        let b_index = triangle[1] as usize;
        let c_index = triangle[2] as usize;
        let a = Vec3::from_array(positions[a_index]);
        let b = Vec3::from_array(positions[b_index]);
        let c = Vec3::from_array(positions[c_index]);
        let mut face_normal = (b - a).cross(c - a).normalize_or_zero();

        if force_positive_y && face_normal.y < 0.0 {
            face_normal = -face_normal;
        }
        if face_normal.length_squared() <= f32::EPSILON {
            face_normal = fallback;
        }

        normals[a_index] += face_normal;
        normals[b_index] += face_normal;
        normals[c_index] += face_normal;
    }

    normals
        .into_iter()
        .map(|normal| {
            if normal.length_squared() <= f32::EPSILON {
                fallback.to_array()
            } else {
                normal.normalize().to_array()
            }
        })
        .collect()
}
