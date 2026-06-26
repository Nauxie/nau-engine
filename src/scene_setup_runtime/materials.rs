use bevy::prelude::*;

use crate::generated_content::{
    IslandDetailMaterials, TERRAIN_BIOME_PALETTE_COUNT, biome_detail_materials,
    cloud_surface_material, cloud_veil_material, emissive_material, glider_airflow_material,
    terrain_surface_material, textured_material, updraft_column_material, updraft_ribbon_material,
    water_surface_material,
};

pub(super) struct SceneMaterials {
    pub(super) suit: Handle<StandardMaterial>,
    pub(super) skin: Handle<StandardMaterial>,
    pub(super) accent: Handle<StandardMaterial>,
    pub(super) glider: Handle<StandardMaterial>,
    pub(super) glider_airflow: Handle<StandardMaterial>,
    pub(super) island_grass: Handle<StandardMaterial>,
    pub(super) island_meadow: Handle<StandardMaterial>,
    pub(super) island_clay: Handle<StandardMaterial>,
    pub(super) island_alpine: Handle<StandardMaterial>,
    pub(super) island_highland: Handle<StandardMaterial>,
    pub(super) target_grass: Handle<StandardMaterial>,
    pub(super) island_rock: Handle<StandardMaterial>,
    pub(super) island_under: Handle<StandardMaterial>,
    pub(super) target_marker: Handle<StandardMaterial>,
    pub(super) biome_detail_sets: Vec<IslandDetailMaterials>,
    pub(super) flower: Handle<StandardMaterial>,
    pub(super) water: Handle<StandardMaterial>,
    pub(super) ground: Handle<StandardMaterial>,
    pub(super) pillar: Handle<StandardMaterial>,
    pub(super) cloud: Handle<StandardMaterial>,
    pub(super) cloud_veil: Handle<StandardMaterial>,
    pub(super) updraft_column: Handle<StandardMaterial>,
    pub(super) updraft_ribbon: Handle<StandardMaterial>,
    pub(super) updraft_marker: Handle<StandardMaterial>,
    pub(super) power_up: Handle<StandardMaterial>,
    pub(super) terrain_texture_detail_bands: usize,
}

pub(super) fn prepare_scene_materials(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> SceneMaterials {
    let suit = textured_material(
        images,
        materials,
        [38, 48, 62, 255],
        [24, 30, 42, 255],
        [78, 90, 104, 255],
        3,
        0.82,
        0.32,
    );
    let skin = textured_material(
        images,
        materials,
        [206, 145, 100, 255],
        [172, 106, 72, 255],
        [232, 176, 130, 255],
        5,
        0.64,
        0.24,
    );
    let accent = emissive_material(
        images,
        materials,
        [238, 156, 36, 255],
        [174, 92, 22, 255],
        [255, 220, 94, 255],
        7,
        LinearRgba::rgb(3.8, 1.7, 0.35),
    );
    let glider = textured_material(
        images,
        materials,
        [166, 88, 44, 255],
        [98, 48, 30, 255],
        [222, 156, 72, 255],
        11,
        0.86,
        0.28,
    );
    let glider_airflow = glider_airflow_material(materials);
    let (island_grass, island_grass_texture_detail_bands) = terrain_surface_material(
        images,
        materials,
        [54, 128, 70, 255],
        [28, 92, 48, 255],
        [128, 174, 78, 255],
        17,
        0.94,
        0.2,
    );
    let (island_meadow, island_meadow_texture_detail_bands) = terrain_surface_material(
        images,
        materials,
        [96, 138, 70, 255],
        [56, 104, 54, 255],
        [166, 172, 90, 255],
        19,
        0.92,
        0.21,
    );
    let (island_clay, island_clay_texture_detail_bands) = terrain_surface_material(
        images,
        materials,
        [126, 104, 76, 255],
        [80, 70, 60, 255],
        [162, 138, 96, 255],
        23,
        0.98,
        0.18,
    );
    let (island_alpine, island_alpine_texture_detail_bands) = terrain_surface_material(
        images,
        materials,
        [52, 110, 118, 255],
        [30, 80, 94, 255],
        [142, 176, 164, 255],
        29,
        0.9,
        0.22,
    );
    let (island_highland, island_highland_texture_detail_bands) = terrain_surface_material(
        images,
        materials,
        [132, 132, 92, 255],
        [86, 96, 70, 255],
        [178, 166, 112, 255],
        31,
        0.94,
        0.2,
    );
    let (target_grass, target_grass_texture_detail_bands) = terrain_surface_material(
        images,
        materials,
        [70, 150, 94, 255],
        [34, 100, 62, 255],
        [156, 198, 112, 255],
        37,
        0.9,
        0.24,
    );
    let island_rock = textured_material(
        images,
        materials,
        [92, 86, 80, 255],
        [48, 48, 48, 255],
        [140, 128, 112, 255],
        41,
        0.98,
        0.16,
    );
    let island_under = textured_material(
        images,
        materials,
        [54, 50, 44, 255],
        [26, 24, 22, 255],
        [88, 78, 64, 255],
        43,
        1.0,
        0.12,
    );
    let target_marker = emissive_material(
        images,
        materials,
        [242, 190, 48, 255],
        [170, 112, 24, 255],
        [255, 235, 120, 255],
        47,
        LinearRgba::rgb(4.8, 3.2, 0.7),
    );
    let biome_detail_sets = (0..TERRAIN_BIOME_PALETTE_COUNT)
        .map(|index| biome_detail_materials(images, materials, index))
        .collect::<Vec<_>>();
    let flower = emissive_material(
        images,
        materials,
        [210, 50, 96, 255],
        [124, 28, 80, 255],
        [255, 126, 162, 255],
        61,
        LinearRgba::rgb(1.2, 0.25, 0.45),
    );
    let water = water_surface_material(images, materials);
    let ground = textured_material(
        images,
        materials,
        [42, 94, 52, 255],
        [24, 60, 40, 255],
        [92, 130, 68, 255],
        71,
        0.96,
        0.18,
    );
    let pillar = textured_material(
        images,
        materials,
        [106, 94, 74, 255],
        [66, 58, 52, 255],
        [152, 134, 100, 255],
        73,
        0.98,
        0.16,
    );
    let cloud = cloud_surface_material(materials);
    let cloud_veil = cloud_veil_material(materials);
    let updraft_column = updraft_column_material(materials);
    let updraft_ribbon = updraft_ribbon_material(materials);
    let updraft_marker = emissive_material(
        images,
        materials,
        [62, 198, 244, 210],
        [20, 118, 184, 210],
        [178, 246, 255, 240],
        83,
        LinearRgba::rgb(0.5, 3.2, 5.8),
    );
    let power_up = emissive_material(
        images,
        materials,
        [255, 210, 70, 230],
        [210, 82, 34, 220],
        [255, 246, 150, 255],
        89,
        LinearRgba::rgb(5.6, 2.4, 0.5),
    );
    let terrain_texture_detail_bands = [
        island_grass_texture_detail_bands,
        island_meadow_texture_detail_bands,
        island_clay_texture_detail_bands,
        island_alpine_texture_detail_bands,
        island_highland_texture_detail_bands,
        target_grass_texture_detail_bands,
    ]
    .into_iter()
    .min()
    .unwrap_or(0);

    SceneMaterials {
        suit,
        skin,
        accent,
        glider,
        glider_airflow,
        island_grass,
        island_meadow,
        island_clay,
        island_alpine,
        island_highland,
        target_grass,
        island_rock,
        island_under,
        target_marker,
        biome_detail_sets,
        flower,
        water,
        ground,
        pillar,
        cloud,
        cloud_veil,
        updraft_column,
        updraft_ribbon,
        updraft_marker,
        power_up,
        terrain_texture_detail_bands,
    }
}
