use super::constants::{
    ISLAND_BODY_SEGMENTS, ISLAND_TERRAIN_EDGE_SKIRT_DEPTH_M, ISLAND_TERRAIN_RINGS,
};
use super::normals::smooth_normals_from_triangles;
use super::palette::{
    balance_terrain_material_weights, island_terrain_material_weights_for_shape, island_terrain_uv,
    island_terrain_vertex_color_for_shape,
};
use super::shape::island_silhouette_scale;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

pub(crate) fn island_terrain_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let vertex_count = 1 + (ISLAND_TERRAIN_RINGS + 1) * ISLAND_BODY_SEGMENTS;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut material_weights = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(
        ISLAND_BODY_SEGMENTS * 3 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS * 6,
    );

    let center_y = island.mesh_top_y_at(island.center);
    positions.push([island.center.x, center_y, island.center.z]);
    uvs.push(island_terrain_uv(
        island_index,
        island,
        island.center.x,
        island.center.z,
    ));
    colors.push(island_terrain_vertex_color_for_shape(
        island.shape_language(),
        island_index,
        0.0,
        0.0,
        center_y - island.mesh_top_y(),
    ));
    material_weights.push(island_terrain_material_weights_for_shape(
        island.shape_language(),
        island_index,
        0.0,
        0.0,
        center_y - island.mesh_top_y(),
    ));

    for ring in 1..=ISLAND_TERRAIN_RINGS {
        let radius = ring as f32 / ISLAND_TERRAIN_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge_scale = island_silhouette_scale(island, angle);
            let radius_scale = radius * (1.0 + radius.powf(1.35) * (edge_scale - 1.0));
            let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
            let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
            let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));

            positions.push([x, y, z]);
            uvs.push(island_terrain_uv(island_index, island, x, z));
            colors.push(island_terrain_vertex_color_for_shape(
                island.shape_language(),
                island_index,
                radius,
                angle,
                y - island.mesh_top_y(),
            ));
            material_weights.push(island_terrain_material_weights_for_shape(
                island.shape_language(),
                island_index,
                radius,
                angle,
                y - island.mesh_top_y(),
            ));
        }
    }

    for segment in 0..ISLAND_BODY_SEGMENTS {
        let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
        let edge_scale = island_silhouette_scale(island, angle);
        let x = island.center.x + angle.cos() * island.half_extents.x * edge_scale;
        let z = island.center.z + angle.sin() * island.half_extents.y * edge_scale;
        let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z))
            - ISLAND_TERRAIN_EDGE_SKIRT_DEPTH_M;

        positions.push([x, y, z]);
        uvs.push(island_terrain_uv(island_index, island, x, z));
        colors.push(island_terrain_vertex_color_for_shape(
            island.shape_language(),
            island_index,
            1.0,
            angle,
            y - island.mesh_top_y(),
        ));
        material_weights.push(island_terrain_material_weights_for_shape(
            island.shape_language(),
            island_index,
            1.0,
            angle,
            y - island.mesh_top_y(),
        ));
    }
    balance_terrain_material_weights(&mut material_weights);

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (1 + (ring - 1) * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for segment in 0..ISLAND_BODY_SEGMENTS {
        indices.extend([0, ring_index(1, segment + 1), ring_index(1, segment)]);
    }

    for ring in 1..ISLAND_TERRAIN_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
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

    let skirt_index = |segment: usize| -> u32 {
        (1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for segment in 0..ISLAND_BODY_SEGMENTS {
        let outer_current = ring_index(ISLAND_TERRAIN_RINGS, segment);
        let outer_next = ring_index(ISLAND_TERRAIN_RINGS, segment + 1);
        let skirt_current = skirt_index(segment);
        let skirt_next = skirt_index(segment + 1);

        indices.extend([
            outer_current,
            outer_next,
            skirt_current,
            outer_next,
            skirt_next,
            skirt_current,
        ]);
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, material_weights)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
