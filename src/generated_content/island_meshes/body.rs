use super::constants::{
    ISLAND_BODY_SEGMENTS, ISLAND_CLIFF_RINGS, ISLAND_CLIFF_STRATA_BANDS,
    ISLAND_IMPOSTOR_CLIFF_RINGS, ISLAND_IMPOSTOR_SEGMENTS, ISLAND_IMPOSTOR_TERRAIN_RINGS,
    ISLAND_IMPOSTOR_UNDERSIDE_RINGS, ISLAND_UNDERSIDE_RINGS,
};
use super::normals::{
    smooth_normals_from_triangles, smooth_normals_from_triangles_oriented,
    world_aligned_tangents_from_positions_and_normals,
};
use super::palette::{
    balance_terrain_material_weights, island_rock_vertex_color, island_terrain_material_weights,
    island_terrain_uv, island_terrain_vertex_color,
};
use super::shape::{island_polar_position, island_silhouette_scale};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
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
    let top_y = island.mesh_top_y();

    for ring in 0..=ISLAND_UNDERSIDE_RINGS {
        let t = ring as f32 / ISLAND_UNDERSIDE_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            positions.push(island_underside_surface_position(
                island_index,
                island,
                angle,
                t,
            ));
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

fn island_underside_surface_position(
    island_index: usize,
    island: SkyIsland,
    angle: f32,
    t: f32,
) -> [f32; 3] {
    if t <= f32::EPSILON {
        return island_cliff_surface_position(island_index, island, angle, 1.0);
    }

    let phase = island_index as f32 * 0.73;
    let twist = 0.045 * (angle * 6.0 + phase + t * 2.4).sin();
    let radius_scale = island_silhouette_scale(island, angle)
        * (0.66 * (1.0 - t).powf(1.35) + 0.18 * t)
        * (1.0 + twist);
    let y = island.mesh_top_y()
        - island.thickness * (0.82 + t * 0.58)
        - island.thickness * 0.06 * (angle * 5.0 - phase).sin().abs();

    island_polar_position(island, angle, radius_scale, y)
}

pub(crate) fn island_impostor_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let terrain = island_impostor_terrain_mesh(island_index, island);
    let cliff = island_impostor_cliff_mesh(island_index, island);
    let underside = island_impostor_underside_mesh(island_index, island);
    let vertex_count =
        terrain.count_vertices() + cliff.count_vertices() + underside.count_vertices();
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut material_weights = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(
        mesh_index_count(&terrain) + mesh_index_count(&cliff) + mesh_index_count(&underside),
    );

    append_mesh(
        &terrain,
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut colors,
        &mut material_weights,
        &mut indices,
    );
    append_mesh(
        &cliff,
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut colors,
        &mut material_weights,
        &mut indices,
    );
    append_mesh(
        &underside,
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut colors,
        &mut material_weights,
        &mut indices,
    );

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, material_weights)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn island_impostor_terrain_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let vertex_count = 1 + ISLAND_IMPOSTOR_TERRAIN_RINGS * ISLAND_IMPOSTOR_SEGMENTS;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut material_weights = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(
        ISLAND_IMPOSTOR_SEGMENTS * 3
            + (ISLAND_IMPOSTOR_TERRAIN_RINGS - 1) * ISLAND_IMPOSTOR_SEGMENTS * 6,
    );

    let center_y = island.mesh_top_y_at(island.center);
    positions.push([island.center.x, center_y, island.center.z]);
    uvs.push(island_terrain_uv(
        island_index,
        island,
        island.center.x,
        island.center.z,
    ));
    colors.push(island_terrain_vertex_color(
        island_index,
        0.0,
        0.0,
        center_y - island.mesh_top_y(),
    ));
    material_weights.push(island_terrain_material_weights(
        island_index,
        0.0,
        0.0,
        center_y - island.mesh_top_y(),
    ));

    for ring in 1..=ISLAND_IMPOSTOR_TERRAIN_RINGS {
        let radius = ring as f32 / ISLAND_IMPOSTOR_TERRAIN_RINGS as f32;
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
            let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge_scale = island_silhouette_scale(island, angle);
            let radius_scale = radius * (1.0 + radius.powf(1.35) * (edge_scale - 1.0));
            let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
            let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
            let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));
            let height_delta = y - island.mesh_top_y();

            positions.push([x, y, z]);
            uvs.push(island_terrain_uv(island_index, island, x, z));
            colors.push(island_terrain_vertex_color(
                island_index,
                radius,
                angle,
                height_delta,
            ));
            material_weights.push(island_terrain_material_weights(
                island_index,
                radius,
                angle,
                height_delta,
            ));
        }
    }
    balance_terrain_material_weights(&mut material_weights);

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (1 + (ring - 1) * ISLAND_IMPOSTOR_SEGMENTS + segment % ISLAND_IMPOSTOR_SEGMENTS) as u32
    };

    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        indices.extend([0, ring_index(1, segment + 1), ring_index(1, segment)]);
    }
    for ring in 1..ISLAND_IMPOSTOR_TERRAIN_RINGS {
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
            let inner_current = ring_index(ring, segment);
            let inner_next = ring_index(ring, segment + 1);
            let outer_current = ring_index(ring + 1, segment);
            let outer_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                inner_current,
                inner_next,
                outer_current,
                inner_next,
                outer_next,
                outer_current,
            ]);
        }
    }

    let normals = smooth_normals_from_triangles(&positions, &indices);
    let tangents = world_aligned_tangents_from_positions_and_normals(&positions, &normals);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, tangents)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, material_weights)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn island_impostor_cliff_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let mut positions =
        Vec::with_capacity((ISLAND_IMPOSTOR_CLIFF_RINGS + 1) * ISLAND_IMPOSTOR_SEGMENTS);
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut colors = Vec::with_capacity(positions.capacity());
    let mut indices =
        Vec::with_capacity(ISLAND_IMPOSTOR_CLIFF_RINGS * ISLAND_IMPOSTOR_SEGMENTS * 6);

    for ring in 0..=ISLAND_IMPOSTOR_CLIFF_RINGS {
        let t = ring as f32 / ISLAND_IMPOSTOR_CLIFF_RINGS as f32;
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
            let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
            positions.push(island_cliff_surface_position(
                island_index,
                island,
                angle,
                t,
            ));
            uvs.push([segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * 4.0, t]);
            colors.push(island_rock_vertex_color(island_index, angle, t, false));
        }
    }

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (ring * ISLAND_IMPOSTOR_SEGMENTS + segment % ISLAND_IMPOSTOR_SEGMENTS) as u32
    };

    for ring in 0..ISLAND_IMPOSTOR_CLIFF_RINGS {
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
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

pub(crate) fn island_impostor_underside_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let ring_vertex_count = (ISLAND_IMPOSTOR_UNDERSIDE_RINGS + 1) * ISLAND_IMPOSTOR_SEGMENTS;
    let bottom_index = ring_vertex_count as u32;
    let mut positions = Vec::with_capacity(ring_vertex_count + 1);
    let mut uvs = Vec::with_capacity(ring_vertex_count + 1);
    let mut colors = Vec::with_capacity(ring_vertex_count + 1);
    let mut indices = Vec::with_capacity(
        ISLAND_IMPOSTOR_UNDERSIDE_RINGS * ISLAND_IMPOSTOR_SEGMENTS * 6
            + ISLAND_IMPOSTOR_SEGMENTS * 3,
    );
    let top_y = island.mesh_top_y();

    for ring in 0..=ISLAND_IMPOSTOR_UNDERSIDE_RINGS {
        let t = ring as f32 / ISLAND_IMPOSTOR_UNDERSIDE_RINGS as f32;
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
            let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
            positions.push(island_underside_surface_position(
                island_index,
                island,
                angle,
                t,
            ));
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
        (ring * ISLAND_IMPOSTOR_SEGMENTS + segment % ISLAND_IMPOSTOR_SEGMENTS) as u32
    };

    for ring in 0..ISLAND_IMPOSTOR_UNDERSIDE_RINGS {
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
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
    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        indices.extend([
            ring_index(ISLAND_IMPOSTOR_UNDERSIDE_RINGS, segment),
            ring_index(ISLAND_IMPOSTOR_UNDERSIDE_RINGS, segment + 1),
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

fn append_mesh(
    mesh: &Mesh,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    material_weights: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let vertex_base = positions.len() as u32;
    let Some(VertexAttributeValues::Float32x3(mesh_positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };
    let vertex_count = mesh_positions.len();

    positions.extend(mesh_positions.iter().copied());
    if let Some(VertexAttributeValues::Float32x3(mesh_normals)) =
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        normals.extend(mesh_normals.iter().copied());
    } else {
        normals.extend(std::iter::repeat_n([0.0, 1.0, 0.0], vertex_count));
    }
    if let Some(VertexAttributeValues::Float32x2(mesh_uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        uvs.extend(mesh_uvs.iter().copied());
    } else {
        uvs.extend(std::iter::repeat_n([0.0, 0.0], vertex_count));
    }
    if let Some(VertexAttributeValues::Float32x4(mesh_colors)) =
        mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    {
        colors.extend(mesh_colors.iter().copied());
    } else {
        colors.extend(std::iter::repeat_n([1.0, 1.0, 1.0, 1.0], vertex_count));
    }
    if let Some(VertexAttributeValues::Float32x2(mesh_weights)) =
        mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    {
        material_weights.extend(mesh_weights.iter().copied());
    } else {
        material_weights.extend(std::iter::repeat_n([0.0, 0.0], vertex_count));
    }

    match mesh.indices() {
        Some(Indices::U16(mesh_indices)) => {
            indices.extend(
                mesh_indices
                    .iter()
                    .map(|index| vertex_base + u32::from(*index)),
            );
        }
        Some(Indices::U32(mesh_indices)) => {
            indices.extend(mesh_indices.iter().map(|index| vertex_base + *index));
        }
        None => {
            indices.extend((0..vertex_count as u32).map(|index| vertex_base + index));
        }
    }
}

fn mesh_index_count(mesh: &Mesh) -> usize {
    match mesh.indices() {
        Some(Indices::U16(indices)) => indices.len(),
        Some(Indices::U32(indices)) => indices.len(),
        None => mesh.count_vertices(),
    }
}
