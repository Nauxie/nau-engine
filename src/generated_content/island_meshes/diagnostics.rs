use super::constants::{
    ISLAND_BODY_SEGMENTS, ISLAND_CLIFF_RINGS, ISLAND_IMPOSTOR_SEGMENTS, ISLAND_TERRAIN_RINGS,
    ISLAND_UNDERSIDE_RINGS,
};
use super::palette::{
    island_rock_vertex_color, island_terrain_material_weights, island_terrain_vertex_color,
    terrain_material_region_id,
};
use super::shape::island_playable_silhouette_scale;
use bevy::prelude::*;
use nau_engine::world::SkyIsland;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandTerrainMeshDiagnostics {
    pub(crate) vertex_count: usize,
    pub(crate) color_bands: usize,
    pub(crate) material_weight_bands: usize,
    pub(crate) material_channels: usize,
    pub(crate) material_regions: usize,
    pub(crate) relief_range_m: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandImpostorMeshDiagnostics {
    pub(crate) vertex_count: usize,
    pub(crate) color_bands: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandBodyMeshDiagnostics {
    pub(crate) cliff_color_bands: usize,
    pub(crate) underside_color_bands: usize,
    pub(crate) cliff_vertex_count: usize,
    pub(crate) underside_vertex_count: usize,
}

impl IslandBodyMeshDiagnostics {
    pub(crate) fn total_vertex_count(self) -> usize {
        self.cliff_vertex_count + self.underside_vertex_count
    }
}

pub(crate) fn island_terrain_mesh_diagnostics(
    island_index: usize,
    island: SkyIsland,
) -> IslandTerrainMeshDiagnostics {
    let vertex_count = 1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS;
    let mut color_bands = HashSet::new();
    let mut material_weight_bands = HashSet::new();
    let mut material_regions = HashSet::new();
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut base_material = false;
    let mut lush_material = false;
    let mut exposed_material = false;

    let mut record_vertex = |radius: f32, angle: f32, x: f32, z: f32| {
        let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));
        min_y = min_y.min(y);
        max_y = max_y.max(y);

        let height_delta = y - island.mesh_top_y();
        let color = island_terrain_vertex_color(island_index, radius, angle, height_delta);
        color_bands.insert(quantized_color_band(color));

        let weight = island_terrain_material_weights(island_index, radius, angle, height_delta);
        material_weight_bands.insert([
            (weight[0].clamp(0.0, 1.0) * 15.0).round() as u8,
            (weight[1].clamp(0.0, 1.0) * 15.0).round() as u8,
        ]);
        material_regions.insert(terrain_material_region_id(weight));
        base_material |= weight[0] < 0.18 && weight[1] < 0.18;
        lush_material |= weight[0] > 0.18;
        exposed_material |= weight[1] > 0.18;
    };

    record_vertex(0.0, 0.0, island.center.x, island.center.z);
    for ring in 1..=ISLAND_TERRAIN_RINGS {
        let radius = ring as f32 / ISLAND_TERRAIN_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge_scale = island_playable_silhouette_scale(island, angle);
            let radius_scale = radius * (1.0 + radius.powf(1.35) * (edge_scale - 1.0));
            let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
            let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
            record_vertex(radius, angle, x, z);
        }
    }

    IslandTerrainMeshDiagnostics {
        vertex_count,
        color_bands: color_bands.len(),
        material_weight_bands: material_weight_bands.len(),
        material_channels: usize::from(base_material)
            + usize::from(lush_material)
            + usize::from(exposed_material),
        material_regions: material_regions.len(),
        relief_range_m: if min_y.is_finite() && max_y.is_finite() {
            max_y - min_y
        } else {
            0.0
        },
    }
}

pub(crate) fn island_impostor_mesh_diagnostics(
    island_index: usize,
    island: SkyIsland,
) -> IslandImpostorMeshDiagnostics {
    let vertex_count = 2 + ISLAND_IMPOSTOR_SEGMENTS * 3;
    let mut color_bands = HashSet::new();
    color_bands.insert(quantized_color_band(island_terrain_vertex_color(
        island_index,
        0.0,
        0.0,
        0.0,
    )));

    let phase = island_index as f32 * 0.71;
    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
        let edge_variation =
            1.0 + 0.09 * (angle * 3.0 + phase).sin() + 0.045 * (angle * 7.0 - phase).cos();
        let radius_x = island.half_extents.x * 0.9 * edge_variation;
        let radius_z = island.half_extents.y * 0.9 * edge_variation;
        let x = island.center.x + angle.cos() * radius_x;
        let z = island.center.z + angle.sin() * radius_z;
        let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) - 0.18;
        color_bands.insert(quantized_color_band(island_terrain_vertex_color(
            island_index,
            0.9,
            angle,
            y - island.mesh_top_y(),
        )));
    }

    for (t, underside) in [(0.34_f32, false), (0.78_f32, true)] {
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
            let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
            color_bands.insert(quantized_color_band(island_rock_vertex_color(
                island_index,
                angle,
                t,
                underside,
            )));
        }
    }

    color_bands.insert(quantized_color_band(island_rock_vertex_color(
        island_index,
        0.0,
        1.0,
        true,
    )));

    IslandImpostorMeshDiagnostics {
        vertex_count,
        color_bands: color_bands.len(),
    }
}

pub(crate) fn island_body_mesh_diagnostics(
    island_index: usize,
    _island: SkyIsland,
) -> IslandBodyMeshDiagnostics {
    let mut cliff_color_bands = HashSet::new();
    for ring in 0..=ISLAND_CLIFF_RINGS {
        let t = ring as f32 / ISLAND_CLIFF_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            cliff_color_bands.insert(quantized_color_band(island_rock_vertex_color(
                island_index,
                angle,
                t,
                false,
            )));
        }
    }

    let mut underside_color_bands = HashSet::new();
    for ring in 0..=ISLAND_UNDERSIDE_RINGS {
        let t = ring as f32 / ISLAND_UNDERSIDE_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            underside_color_bands.insert(quantized_color_band(island_rock_vertex_color(
                island_index,
                angle,
                t,
                true,
            )));
        }
    }
    underside_color_bands.insert(quantized_color_band(island_rock_vertex_color(
        island_index,
        0.0,
        1.0,
        true,
    )));

    IslandBodyMeshDiagnostics {
        cliff_color_bands: cliff_color_bands.len(),
        underside_color_bands: underside_color_bands.len(),
        cliff_vertex_count: (ISLAND_CLIFF_RINGS + 1) * ISLAND_BODY_SEGMENTS,
        underside_vertex_count: (ISLAND_UNDERSIDE_RINGS + 1) * ISLAND_BODY_SEGMENTS + 1,
    }
}

fn quantized_color_band(color: [f32; 4]) -> [u8; 3] {
    [
        (color[0].clamp(0.0, 1.0) * 31.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 31.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 31.0).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::super::body::{island_cliff_mesh, island_impostor_mesh, island_underside_mesh};
    use super::super::metrics::{
        mesh_terrain_material_channel_count, mesh_terrain_material_region_count,
        mesh_terrain_material_weight_band_count, mesh_vertex_color_band_count, mesh_y_range,
    };
    use super::super::terrain::island_terrain_mesh;
    use super::*;

    #[test]
    fn lightweight_diagnostics_match_generated_mesh_metrics() {
        let route = nau_engine::world::SkyRoute::default();
        let island_index = 3;
        let island = route.islands()[island_index];

        let terrain = island_terrain_mesh(island_index, island);
        let terrain_diagnostics = island_terrain_mesh_diagnostics(island_index, island);
        assert_eq!(terrain_diagnostics.vertex_count, terrain.count_vertices());
        assert_eq!(
            terrain_diagnostics.color_bands,
            mesh_vertex_color_band_count(&terrain)
        );
        assert_eq!(
            terrain_diagnostics.material_weight_bands,
            mesh_terrain_material_weight_band_count(&terrain)
        );
        assert_eq!(
            terrain_diagnostics.material_channels,
            mesh_terrain_material_channel_count(&terrain)
        );
        assert_eq!(
            terrain_diagnostics.material_regions,
            mesh_terrain_material_region_count(&terrain)
        );
        assert!((terrain_diagnostics.relief_range_m - mesh_y_range(&terrain)).abs() < 0.001);

        let impostor = island_impostor_mesh(island_index, island);
        let impostor_diagnostics = island_impostor_mesh_diagnostics(island_index, island);
        assert_eq!(impostor_diagnostics.vertex_count, impostor.count_vertices());
        assert_eq!(
            impostor_diagnostics.color_bands,
            mesh_vertex_color_band_count(&impostor)
        );

        let cliff = island_cliff_mesh(island_index, island);
        let underside = island_underside_mesh(island_index, island);
        let body_diagnostics = island_body_mesh_diagnostics(island_index, island);
        assert_eq!(
            body_diagnostics.cliff_color_bands,
            mesh_vertex_color_band_count(&cliff)
        );
        assert_eq!(
            body_diagnostics.underside_color_bands,
            mesh_vertex_color_band_count(&underside)
        );
        assert_eq!(
            body_diagnostics.total_vertex_count(),
            cliff.count_vertices() + underside.count_vertices()
        );
    }
}
