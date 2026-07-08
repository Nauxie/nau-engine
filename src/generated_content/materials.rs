use bevy::image::ImageFilterMode;
use bevy::prelude::*;

use super::textures::{
    TERRAIN_TEXTURE_SIZE, procedural_depth_map, procedural_depth_map_with_size,
    procedural_material_map, procedural_material_map_with_size, procedural_occlusion_map,
    procedural_occlusion_map_with_size, procedural_srgb_texture, procedural_surface_texture,
    procedural_terrain_surface_texture_data, texture_detail_band_count,
};

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
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    perceptual_roughness: f32,
    reflectance: f32,
) -> (Handle<StandardMaterial>, usize) {
    let material_seed = seed.wrapping_add(1_337);
    let surface_data = procedural_terrain_surface_texture_data(
        primary,
        secondary,
        accent,
        seed,
        TERRAIN_TEXTURE_SIZE,
    );
    let detail_bands = texture_detail_band_count(&surface_data);
    let base_color_texture = procedural_srgb_texture(
        surface_data,
        TERRAIN_TEXTURE_SIZE,
        ImageFilterMode::Linear,
        16,
    );

    (
        materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(images.add(base_color_texture)),
            metallic_roughness_texture: Some(images.add(procedural_material_map_with_size(
                material_seed,
                perceptual_roughness,
                TERRAIN_TEXTURE_SIZE,
            ))),
            occlusion_texture: Some(images.add(procedural_occlusion_map_with_size(
                material_seed.wrapping_add(23),
                TERRAIN_TEXTURE_SIZE,
            ))),
            depth_map: Some(images.add(procedural_depth_map_with_size(
                material_seed.wrapping_add(47),
                ImageFilterMode::Linear,
                TERRAIN_TEXTURE_SIZE,
            ))),
            parallax_depth_scale: 0.018,
            max_parallax_layer_count: 12.0,
            perceptual_roughness,
            reflectance,
            ..default()
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

pub(crate) fn water_surface_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.22, 0.58, 0.86, 0.76),
        base_color_texture: Some(images.add(procedural_surface_texture(
            [54, 154, 210, 210],
            [22, 92, 156, 210],
            [160, 220, 244, 210],
            79,
        ))),
        metallic_roughness_texture: Some(images.add(procedural_material_map(1_079, 0.22))),
        depth_map: Some(images.add(procedural_depth_map(1_113, ImageFilterMode::Linear))),
        parallax_depth_scale: 0.018,
        max_parallax_layer_count: 10.0,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        perceptual_roughness: 0.18,
        reflectance: 0.82,
        clearcoat: 0.85,
        clearcoat_perceptual_roughness: 0.06,
        diffuse_transmission: 0.18,
        specular_transmission: 0.08,
        thickness: 0.08,
        ior: 1.33,
        ..default()
    })
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
