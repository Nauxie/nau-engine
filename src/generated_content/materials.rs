use bevy::image::ImageFilterMode;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::surface_material::{SurfaceExtension, SurfaceMaterial};

use super::island_meshes::{IslandDetailMaterials, biome_detail_materials};
use super::surface_textures::{TerrainSurfaceTextureSet, procedural_water_normal_texture};
use super::textures::{
    TERRAIN_TEXTURE_SIZE, procedural_depth_map, procedural_material_map, procedural_occlusion_map,
    procedural_surface_texture,
};

#[derive(Clone)]
pub(crate) struct WaterSurfaceMaterials {
    pub(crate) body: Handle<SurfaceMaterial>,
    pub(crate) foam: Handle<SurfaceMaterial>,
    pub(crate) mist: Handle<StandardMaterial>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn textured_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    perceptual_roughness: f32,
    reflectance: f32,
) -> Handle<StandardMaterial> {
    let material_seed = seed.wrapping_add(1_337);
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        metallic_roughness_texture: Some(
            images.add(procedural_material_map(material_seed, perceptual_roughness)),
        ),
        occlusion_texture: Some(
            images.add(procedural_occlusion_map(material_seed.wrapping_add(23))),
        ),
        depth_map: Some(images.add(procedural_depth_map(
            material_seed.wrapping_add(47),
            ImageFilterMode::Nearest,
        ))),
        parallax_depth_scale: 0.012,
        max_parallax_layer_count: 8.0,
        perceptual_roughness,
        reflectance,
        ..default()
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn terrain_surface_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<SurfaceMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    perceptual_roughness: f32,
    reflectance: f32,
) -> (Handle<SurfaceMaterial>, usize) {
    let texture_primary = terrain_texture_tint(primary);
    let texture_secondary = terrain_texture_tint(secondary);
    let texture_accent = terrain_texture_tint(accent);
    let surface_set = TerrainSurfaceTextureSet::generate(
        texture_primary,
        texture_secondary,
        texture_accent,
        seed,
        TERRAIN_TEXTURE_SIZE,
    );
    let detail_bands = surface_set.detail_bands;
    let base_color_texture = images.add(surface_set.albedo);
    let normal_texture = images.add(surface_set.normal);
    let orm_texture = images.add(surface_set.orm);

    (
        materials.add(SurfaceMaterial {
            base: StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(base_color_texture),
                metallic_roughness_texture: Some(orm_texture.clone()),
                occlusion_texture: Some(orm_texture),
                normal_map_texture: Some(normal_texture),
                perceptual_roughness: perceptual_roughness.clamp(0.72, 1.0),
                reflectance,
                ..default()
            },
            extension: SurfaceExtension::terrain(
                linear_tint(primary),
                linear_tint(secondary),
                linear_tint(accent),
                seed as f32 * 0.0137,
            ),
        }),
        detail_bands,
    )
}

pub(crate) fn emissive_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    emissive: LinearRgba,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        emissive,
        emissive_exposure_weight: 0.15,
        perceptual_roughness: 0.7,
        reflectance: 0.38,
        ..default()
    })
}

pub(crate) fn water_surface_materials(
    images: &mut Assets<Image>,
    materials: &mut Assets<SurfaceMaterial>,
    mist: Handle<StandardMaterial>,
) -> WaterSurfaceMaterials {
    let detail_normal = images.add(procedural_water_normal_texture(79, 256));
    let body = materials.add(SurfaceMaterial {
        base: StandardMaterial {
            base_color: Color::srgba(0.09, 0.31, 0.40, 0.88),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: true,
            perceptual_roughness: 0.18,
            reflectance: 0.35,
            diffuse_transmission: 0.10,
            ior: 1.33,
            ..default()
        },
        extension: SurfaceExtension::water(detail_normal.clone(), 0.79),
    });
    let foam = materials.add(SurfaceMaterial {
        base: StandardMaterial {
            base_color: Color::srgba(0.90, 0.98, 1.0, 0.72),
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            double_sided: true,
            unlit: true,
            perceptual_roughness: 0.58,
            reflectance: 0.46,
            ..default()
        },
        extension: SurfaceExtension::foam(detail_normal, 1.17),
    });

    WaterSurfaceMaterials { body, foam, mist }
}

fn linear_tint(color: [u8; 4]) -> Vec4 {
    LinearRgba::from(Color::srgba_u8(color[0], color[1], color[2], color[3])).to_vec4()
}

fn terrain_texture_tint(color: [u8; 4]) -> [u8; 4] {
    let luma =
        (u16::from(color[0]) * 77 + u16::from(color[1]) * 150 + u16::from(color[2]) * 29) / 256;
    let neutral = 166 + luma * 23 / 100;
    [
        ((neutral * 64 + u16::from(color[0]) * 36) / 100).min(255) as u8,
        ((neutral * 64 + u16::from(color[1]) * 36) / 100).min(255) as u8,
        ((neutral * 64 + u16::from(color[2]) * 36) / 100).min(255) as u8,
        color[3],
    ]
}

pub(crate) fn glider_airflow_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 0.96, 0.22),
        emissive: LinearRgba::rgb(0.46, 0.50, 0.42),
        emissive_exposure_weight: 0.05,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.72,
        reflectance: 0.1,
        ..default()
    })
}

pub(crate) fn cloud_surface_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.96, 0.90, 0.76, 0.48),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        reflectance: 0.18,
        diffuse_transmission: 0.28,
        ..default()
    })
}

pub(crate) fn cloud_veil_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.72, 0.82, 0.96, 0.30),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        reflectance: 0.10,
        diffuse_transmission: 0.42,
        ..default()
    })
}

pub(crate) fn updraft_column_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.38, 0.95, 0.70, 0.010),
        emissive: LinearRgba::rgb(0.010, 0.040, 0.026),
        emissive_exposure_weight: 0.12,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.32,
        reflectance: 0.2,
        ..default()
    })
}

pub(crate) fn updraft_ribbon_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.58, 1.0, 0.72, 0.34),
        emissive: LinearRgba::rgb(0.18, 1.45, 0.82),
        emissive_exposure_weight: 0.2,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.4,
        reflectance: 0.18,
        ..default()
    })
}

pub(crate) fn ground_cover_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        metallic_roughness_texture: Some(
            images.add(procedural_material_map(seed.wrapping_add(1_300), 0.94)),
        ),
        alpha_mode: AlphaMode::Opaque,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 0.94,
        reflectance: 0.2,
        ..default()
    })
}

pub(crate) fn allocate_authored_island_detail_materials(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    shared_palette_materials: &[IslandDetailMaterials],
) -> Vec<IslandDetailMaterials> {
    assert!(
        !shared_palette_materials.is_empty(),
        "runtime island materials require at least one shared palette"
    );

    let profiles = nau_engine::world::island_art_directions();
    assert!(
        shared_palette_materials.len() <= profiles.len(),
        "shared runtime palettes cannot exceed the authored island count"
    );

    let mut family_materials = HashMap::new();
    for (island_index, profile) in profiles.iter().enumerate() {
        family_materials
            .entry(profile.palette_family)
            .or_insert_with(|| {
                shared_palette_materials
                    .get(island_index)
                    .cloned()
                    .unwrap_or_else(|| biome_detail_materials(images, materials, island_index))
            });
    }

    profiles
        .iter()
        .map(|profile| {
            family_materials
                .get(&profile.palette_family)
                .expect("every authored palette family should have runtime materials")
                .clone()
        })
        .collect()
}
