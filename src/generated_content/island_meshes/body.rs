use super::constants::{
    ISLAND_BODY_SEGMENTS, ISLAND_CLIFF_RINGS, ISLAND_CLIFF_STRATA_BANDS, ISLAND_IMPOSTOR_SEGMENTS,
    ISLAND_UNDERSIDE_RINGS,
};
use super::normals::{smooth_normals_from_triangles, smooth_normals_from_triangles_oriented};
use super::palette::{island_rock_vertex_color, island_terrain_vertex_color_for_shape};
use super::shape::{island_polar_position, island_silhouette_scale};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

pub(crate) fn island_cliff_surface_position(
    island_index: usize,
    island: SkyIsland,
    angle: f32,
    t: f32,
) -> [f32; 3] {
    let phase = island_index as f32 * 0.73;
    if t <= f32::EPSILON {
        let radius_scale = island_silhouette_scale(island, angle);
        let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
        let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
        return [x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z];
    }

    let shelf_variation = 1.0
        + t * 0.035 * (angle * 5.0 + phase + t * 1.7).sin()
        + t * 0.025 * (angle * 13.0 - phase * 0.3 + t * 2.1).cos();
    let ledge_phase = (t * ISLAND_CLIFF_STRATA_BANDS as f32 + phase * 0.11).fract();
    let ledge_shelf = (1.0 - (ledge_phase - 0.5).abs() * 2.0).max(0.0).powf(2.2);
    let radius_scale = island_silhouette_scale(island, angle)
        * (1.0 - t.powf(1.18) * 0.34)
        * shelf_variation
        * (1.0 + ledge_shelf * 0.028);
    let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
    let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
    let vertical_fracture = t
        * ((angle * 8.0 + phase).sin() * (0.45 + t) + (angle * 17.0 - phase).cos() * 0.22).abs()
        * island.thickness
        * 0.045;
    let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z))
        - 0.06
        - island.thickness * (t * 0.78)
        - ledge_shelf * island.thickness * 0.018
        - vertical_fracture;

    [x, y, z]
}

pub(crate) fn island_cliff_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let mut positions = Vec::with_capacity((ISLAND_CLIFF_RINGS + 1) * ISLAND_BODY_SEGMENTS);
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut colors = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(ISLAND_CLIFF_RINGS * ISLAND_BODY_SEGMENTS * 6);

    for ring in 0..=ISLAND_CLIFF_RINGS {
        let t = ring as f32 / ISLAND_CLIFF_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            positions.push(island_cliff_surface_position(
                island_index,
                island,
                angle,
                t,
            ));
            uvs.push([segment as f32 / ISLAND_BODY_SEGMENTS as f32 * 4.0, t]);
            colors.push(island_rock_vertex_color(island_index, angle, t, false));
        }
    }

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (ring * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for ring in 0..ISLAND_CLIFF_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let upper_current = ring_index(ring, segment);
            let upper_next = ring_index(ring, segment + 1);
            let lower_current = ring_index(ring + 1, segment);
            let lower_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                upper_current,
                upper_next,
                lower_current,
                upper_next,
                lower_next,
                lower_current,
            ]);
        }
    }

    let normals = smooth_normals_from_triangles_oriented(&positions, &indices, Vec3::Z, false);

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

pub(crate) fn island_underside_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let ring_vertex_count = (ISLAND_UNDERSIDE_RINGS + 1) * ISLAND_BODY_SEGMENTS;
    let bottom_index = ring_vertex_count as u32;
    let mut positions = Vec::with_capacity(ring_vertex_count + 1);
    let mut uvs = Vec::with_capacity(ring_vertex_count + 1);
    let mut colors = Vec::with_capacity(ring_vertex_count + 1);
    let mut indices = Vec::with_capacity(
        ISLAND_UNDERSIDE_RINGS * ISLAND_BODY_SEGMENTS * 6 + ISLAND_BODY_SEGMENTS * 3,
    );
    let phase = island_index as f32 * 0.73;
    let top_y = island.mesh_top_y();

    for ring in 0..=ISLAND_UNDERSIDE_RINGS {
        let t = ring as f32 / ISLAND_UNDERSIDE_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            if ring == 0 {
                positions.push(island_cliff_surface_position(
                    island_index,
                    island,
                    angle,
                    1.0,
                ));
                uvs.push([0.5 + angle.cos() * 0.34, 0.5 + angle.sin() * 0.34]);
                colors.push(island_rock_vertex_color(island_index, angle, t, true));
                continue;
            }

            let twist = 0.045 * (angle * 6.0 + phase + t * 2.4).sin();
            let radius_scale = island_silhouette_scale(island, angle)
                * (0.66 * (1.0 - t).powf(1.35) + 0.18 * t)
                * (1.0 + twist);
            let y = top_y
                - island.thickness * (0.82 + t * 0.58)
                - island.thickness * 0.06 * (angle * 5.0 - phase).sin().abs();

            positions.push(island_polar_position(island, angle, radius_scale, y));
            uvs.push([
                0.5 + angle.cos() * (0.34 - t * 0.19),
                0.5 + angle.sin() * (0.34 - t * 0.19),
            ]);
            colors.push(island_rock_vertex_color(island_index, angle, t, true));
        }
    }

    positions.push([
        island.center.x,
        top_y - island.thickness * 1.58,
        island.center.z,
    ]);
    uvs.push([0.5, 0.5]);
    colors.push(island_rock_vertex_color(island_index, 0.0, 1.0, true));

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (ring * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for ring in 0..ISLAND_UNDERSIDE_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let upper_current = ring_index(ring, segment);
            let upper_next = ring_index(ring, segment + 1);
            let lower_current = ring_index(ring + 1, segment);
            let lower_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                upper_current,
                upper_next,
                lower_current,
                upper_next,
                lower_next,
                lower_current,
            ]);
        }
    }
    for segment in 0..ISLAND_BODY_SEGMENTS {
        indices.extend([
            ring_index(ISLAND_UNDERSIDE_RINGS, segment),
            ring_index(ISLAND_UNDERSIDE_RINGS, segment + 1),
            bottom_index,
        ]);
    }

    let normals = smooth_normals_from_triangles_oriented(&positions, &indices, Vec3::NEG_Y, false);

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

pub(crate) fn island_impostor_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let top_center_y = island.mesh_top_y() - 0.16;
    let shoulder_center_y = top_center_y - island.thickness * 0.30;
    let lower_center_y = top_center_y - island.thickness * 0.62;
    let bottom_y = top_center_y - island.thickness * 0.92;
    let phase = island_index as f32 * 0.71;
    let top_ring_start = 1;
    let shoulder_ring_start = top_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let lower_ring_start = shoulder_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let bottom_index = lower_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let mut positions = Vec::with_capacity(bottom_index + 1);
    let mut uvs = Vec::with_capacity(bottom_index + 1);
    let mut colors = Vec::with_capacity(bottom_index + 1);
    let mut indices = Vec::with_capacity(ISLAND_IMPOSTOR_SEGMENTS * 18);

    positions.push([island.center.x, top_center_y, island.center.z]);
    uvs.push([0.5, 0.5]);
    colors.push(island_terrain_vertex_color_for_shape(
        island.shape_language(),
        island_index,
        0.0,
        0.0,
        0.0,
    ));

    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
        let contour_scale = island_silhouette_scale(island, angle);
        let edge_variation = 0.96 + 0.035 * (angle * 7.0 - phase).cos();
        let radius_x = island.half_extents.x * 0.9 * contour_scale * edge_variation;
        let radius_z = island.half_extents.y * 0.9 * contour_scale * edge_variation;
        let x = island.center.x + angle.cos() * radius_x;
        let z = island.center.z + angle.sin() * radius_z;
        let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) - 0.18;

        positions.push([x, y, z]);
        uvs.push([0.5 + angle.cos() * 0.45, 0.5 + angle.sin() * 0.45]);
        colors.push(island_terrain_vertex_color_for_shape(
            island.shape_language(),
            island_index,
            0.9,
            angle,
            y - island.mesh_top_y(),
        ));
    }

    for (ring, (center_y, radius_scale, t, underside)) in [
        (shoulder_center_y, 0.72, 0.34, false),
        (lower_center_y, 0.48, 0.78, true),
    ]
    .into_iter()
    .enumerate()
    {
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
            let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
            let contour_scale = island_silhouette_scale(island, angle);
            let edge_variation =
                1.0 + 0.08 * (angle * 4.0 + phase).sin() - 0.035 * (angle * 8.0).cos();
            let radius_x = island.half_extents.x * radius_scale * contour_scale * edge_variation;
            let radius_z = island.half_extents.y * radius_scale * contour_scale * edge_variation;
            let x = island.center.x + angle.cos() * radius_x;
            let z = island.center.z + angle.sin() * radius_z;
            let y = center_y - island.thickness * 0.05 * (angle * 5.0 + phase).sin().abs();

            positions.push([x, y, z]);
            uvs.push([
                0.5 + angle.cos() * (0.35 - ring as f32 * 0.11),
                0.78 + angle.sin() * 0.11 + ring as f32 * 0.14,
            ]);
            colors.push(island_rock_vertex_color(island_index, angle, t, underside));
        }
    }

    positions.push([island.center.x, bottom_y, island.center.z]);
    uvs.push([0.5, 1.0]);
    colors.push(island_rock_vertex_color(island_index, 0.0, 1.0, true));

    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        let next = (segment + 1) % ISLAND_IMPOSTOR_SEGMENTS;
        let top_current = (top_ring_start + segment) as u32;
        let top_next = (top_ring_start + next) as u32;
        let shoulder_current = (shoulder_ring_start + segment) as u32;
        let shoulder_next = (shoulder_ring_start + next) as u32;
        let lower_current = (lower_ring_start + segment) as u32;
        let lower_next = (lower_ring_start + next) as u32;
        let bottom = bottom_index as u32;

        indices.extend([0, top_next, top_current]);
        indices.extend([top_current, top_next, shoulder_current]);
        indices.extend([top_next, shoulder_next, shoulder_current]);
        indices.extend([shoulder_current, shoulder_next, lower_current]);
        indices.extend([shoulder_next, lower_next, lower_current]);
        indices.extend([lower_current, lower_next, bottom]);
    }

    let normals = smooth_normals_from_triangles(&positions, &indices);

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
