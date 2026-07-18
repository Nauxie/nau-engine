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

/// Generates one tangent per position for terrain whose UV axes follow world X/Z.
/// This avoids UV-derivative failures where vertical skirt pairs share the same UV.
pub(crate) fn world_aligned_tangents_from_positions_and_normals(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> Vec<[f32; 4]> {
    const MIN_AXIS_LENGTH_SQUARED: f32 = 1.0e-8;

    debug_assert_eq!(positions.len(), normals.len());

    positions
        .iter()
        .enumerate()
        .map(|(vertex_index, _)| {
            let normal = normals
                .get(vertex_index)
                .copied()
                .map(Vec3::from_array)
                .filter(|normal| normal.is_finite())
                .unwrap_or(Vec3::Y)
                .normalize_or_zero();
            let normal = if normal.length_squared() <= MIN_AXIS_LENGTH_SQUARED {
                Vec3::Y
            } else {
                normal
            };
            let world_u = Vec3::X - normal * normal.x;
            let world_v = Vec3::Z - normal * normal.z;
            let mut tangent = normal.cross(Vec3::Z);

            if tangent.length_squared() <= MIN_AXIS_LENGTH_SQUARED {
                tangent = world_u;
            }
            if tangent.length_squared() <= MIN_AXIS_LENGTH_SQUARED {
                tangent = normal.cross(Vec3::Y);
            }

            tangent = tangent.normalize_or_zero();
            if world_u.length_squared() > MIN_AXIS_LENGTH_SQUARED && tangent.dot(world_u) < 0.0 {
                tangent = -tangent;
            }

            let handedness = if world_v.length_squared() > MIN_AXIS_LENGTH_SQUARED
                && normal.cross(tangent).dot(world_v) < 0.0
            {
                -1.0
            } else {
                1.0
            };

            [tangent.x, tangent.y, tangent.z, handedness]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::body::island_impostor_terrain_mesh;
    use super::super::constants::{
        ISLAND_BODY_SEGMENTS, ISLAND_IMPOSTOR_SEGMENTS, ISLAND_IMPOSTOR_TERRAIN_RINGS,
        ISLAND_TERRAIN_RINGS,
    };
    use super::super::terrain::island_terrain_mesh;
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn near_terrain_has_valid_world_aligned_tangents_including_skirts() {
        let route = nau_engine::world::SkyRoute::default();

        for island_index in [0, 3] {
            let mesh = island_terrain_mesh(island_index, route.islands()[island_index]);

            assert_eq!(
                mesh.count_vertices(),
                1 + (ISLAND_TERRAIN_RINGS + 1) * ISLAND_BODY_SEGMENTS
            );
            assert_mesh_tangent_contract(&mesh);
        }
    }

    #[test]
    fn impostor_terrain_has_valid_world_aligned_tangents() {
        let route = nau_engine::world::SkyRoute::default();

        for island_index in [0, 3] {
            let mesh = island_impostor_terrain_mesh(island_index, route.islands()[island_index]);

            assert_eq!(
                mesh.count_vertices(),
                1 + ISLAND_IMPOSTOR_TERRAIN_RINGS * ISLAND_IMPOSTOR_SEGMENTS
            );
            assert_mesh_tangent_contract(&mesh);
        }
    }

    #[test]
    fn world_aligned_tangents_handle_axis_aligned_skirt_normals() {
        let positions = [[0.0, 0.0, 0.0]; 4];
        let normals = [
            Vec3::X.to_array(),
            Vec3::NEG_X.to_array(),
            Vec3::Z.to_array(),
            Vec3::NEG_Z.to_array(),
        ];
        let tangents = world_aligned_tangents_from_positions_and_normals(&positions, &normals);

        assert_eq!(tangents.len(), positions.len());
        assert_tangent_basis(&normals, &tangents);
    }

    fn assert_mesh_tangent_contract(mesh: &Mesh) {
        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(positions)) => positions,
            _ => panic!("terrain mesh should expose Float32x3 positions"),
        };
        let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
            Some(VertexAttributeValues::Float32x3(normals)) => normals,
            _ => panic!("terrain mesh should expose Float32x3 normals"),
        };
        let tangents = match mesh.attribute(Mesh::ATTRIBUTE_TANGENT) {
            Some(VertexAttributeValues::Float32x4(tangents)) => tangents,
            _ => panic!("terrain mesh should expose Float32x4 tangents"),
        };

        assert_eq!(positions.len(), mesh.count_vertices());
        assert_eq!(normals.len(), positions.len());
        assert_eq!(tangents.len(), positions.len());
        assert_tangent_basis(normals, tangents);
    }

    fn assert_tangent_basis(normals: &[[f32; 3]], tangents: &[[f32; 4]]) {
        assert_eq!(normals.len(), tangents.len());

        for (vertex_index, (normal, tangent)) in normals.iter().zip(tangents).enumerate() {
            let normal = Vec3::from_array(*normal);
            let direction = Vec3::new(tangent[0], tangent[1], tangent[2]);

            assert!(
                tangent.iter().all(|component| component.is_finite()),
                "vertex {vertex_index} tangent should be finite"
            );
            assert!(
                (direction.length() - 1.0).abs() < 1.0e-4,
                "vertex {vertex_index} tangent should be unit-ish"
            );
            assert!(
                normal.dot(direction).abs() < 1.0e-4,
                "vertex {vertex_index} tangent should be orthogonal to its normal"
            );
            assert!(
                (tangent[3].abs() - 1.0).abs() <= f32::EPSILON,
                "vertex {vertex_index} tangent handedness should be signed unit"
            );
        }
    }
}
