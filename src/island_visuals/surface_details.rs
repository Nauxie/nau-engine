use super::queue::{queue_collidable_island_visual, queue_island_visual, queue_wind_island_visual};
use super::types::{IslandStreamState, IslandVisualCatalog, IslandVisualEntry, IslandVisualLayer};
use crate::camera_runtime::CameraObstacle;
use crate::content_diagnostics::{GeneratedLandmarkKind, IslandContentDiagnostics};
use crate::environment_visuals::wind_visual_motion;
use crate::generated_content::{
    FloraMaterialRole, IslandDetailMaterials, TERRAIN_BIOME_PALETTE_COUNT, WaterDetailMaterialRole,
    allocate_authored_island_detail_materials, island_flora_visual_specs,
    island_hero_landmark_spec, island_rock_formation_specs, island_ruin_complex_specs,
    island_water_detail_specs, island_water_visual_specs,
};
use crate::world_collision_runtime::{WorldCollisionProxy, WorldCollisionProxyKind};
use bevy::prelude::*;
use nau_engine::camera::CameraObstruction;
use nau_engine::world::{SkyIsland, SkyRoute};

type SpawnedMaterialUpdates = Vec<(Entity, Handle<StandardMaterial>)>;
type IslandStoneMaterials = Vec<(&'static str, Handle<StandardMaterial>)>;

#[allow(clippy::too_many_arguments)]
pub(super) fn queue_island_surface_details(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    detail_materials: IslandDetailMaterials,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    if let Some(hero) = island_hero_landmark_spec(island_index, island) {
        let mesh = hero.build_mesh();
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::SurfaceFeature,
            mesh.count_vertices(),
        );
        queue_static_surface_feature(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Vista,
            meshes.add(mesh),
            detail_materials.stone.clone(),
            hero.translation,
            hero.rotation_y,
            Some(hero.camera_half_extents),
            hero.collision_half_extents,
            hero.label,
        );
    }

    for (feature_index, feature) in island_flora_visual_specs(island_index, island)
        .into_iter()
        .enumerate()
    {
        let material = match feature.material {
            FloraMaterialRole::Foliage => detail_materials.foliage.clone(),
            FloraMaterialRole::GroundCover => detail_materials.ground_cover.clone(),
            FloraMaterialRole::Flower => flower_material.clone(),
        };
        let mesh = feature.build_mesh();
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::SurfaceFeature,
            mesh.count_vertices(),
        );
        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(mesh),
            material,
            Transform {
                translation: feature.translation,
                rotation: Quat::from_rotation_y(feature.rotation_y),
                ..default()
            },
            None,
            wind_visual_motion(
                island_index,
                feature.wind_phase + feature_index as f32 * 0.31,
                0.08 * feature.wind_motion_scale,
                0.035 * feature.wind_motion_scale,
                1.2 * feature.wind_motion_scale,
            ),
            feature.kind.visual_name(),
        );
    }

    for feature in island_ruin_complex_specs(island_index, island) {
        let mesh = feature.build_mesh();
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::SurfaceFeature,
            mesh.count_vertices(),
        );
        let layer = hero_surface_layer(island, feature.camera_half_extents);
        queue_static_surface_feature(
            entries,
            visual_index,
            island,
            layer,
            meshes.add(mesh),
            detail_materials.stone.clone(),
            feature.translation,
            feature.rotation_y,
            feature.camera_half_extents,
            feature.collision_half_extents,
            feature.kind.visual_name(),
        );
    }

    for feature in island_rock_formation_specs(island_index, island) {
        let mesh = feature.build_mesh();
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::SurfaceFeature,
            mesh.count_vertices(),
        );
        let layer = hero_surface_layer(island, feature.camera_half_extents);
        queue_static_surface_feature(
            entries,
            visual_index,
            island,
            layer,
            meshes.add(mesh),
            detail_materials.stone.clone(),
            feature.translation,
            feature.rotation_y,
            feature.camera_half_extents,
            feature.collision_half_extents,
            feature.kind.visual_name(),
        );
    }

    let water_visuals = island_water_visual_specs(island_index, island);
    let water_layer = if island.is_great_plateau_anchor() {
        IslandVisualLayer::Vista
    } else {
        IslandVisualLayer::Detail
    };
    for (feature_index, feature) in island_water_detail_specs(island_index, island, &water_visuals)
        .into_iter()
        .enumerate()
    {
        let material = match feature.material {
            WaterDetailMaterialRole::Water => water_material.clone(),
            WaterDetailMaterialRole::Stone => detail_materials.stone.clone(),
            WaterDetailMaterialRole::Foliage => detail_materials.foliage.clone(),
            WaterDetailMaterialRole::Flower => flower_material.clone(),
        };
        let mesh = feature.build_mesh();
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::SurfaceFeature,
            mesh.count_vertices(),
        );
        if feature.wind_motion_scale > 0.0 && feature.collision_half_extents.is_none() {
            queue_wind_island_visual(
                entries,
                visual_index,
                island,
                water_layer,
                meshes.add(mesh),
                material,
                Transform {
                    translation: feature.translation,
                    rotation: Quat::from_rotation_y(feature.rotation_y),
                    ..default()
                },
                feature.camera_half_extents.map(|half_extents| {
                    CameraObstacle(CameraObstruction::soft_local_prop(
                        feature.translation,
                        half_extents,
                    ))
                }),
                wind_visual_motion(
                    island_index,
                    feature.wind_phase + feature_index as f32 * 0.23,
                    0.035 * feature.wind_motion_scale,
                    0.018 * feature.wind_motion_scale,
                    1.1 * feature.wind_motion_scale,
                ),
                feature.kind.visual_name(),
            );
        } else {
            queue_static_surface_feature(
                entries,
                visual_index,
                island,
                water_layer,
                meshes.add(mesh),
                material,
                feature.translation,
                feature.rotation_y,
                feature.camera_half_extents,
                feature.collision_half_extents,
                feature.kind.visual_name(),
            );
        }
    }
}

fn hero_surface_layer(island: SkyIsland, camera_half_extents: Option<Vec3>) -> IslandVisualLayer {
    if island.is_great_plateau_anchor()
        || camera_half_extents.is_some_and(|half_extents| half_extents.max_element() >= 5.0)
    {
        IslandVisualLayer::Vista
    } else {
        IslandVisualLayer::Detail
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_static_surface_feature(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    translation: Vec3,
    rotation_y: f32,
    camera_half_extents: Option<Vec3>,
    collision_half_extents: Option<Vec3>,
    name: &'static str,
) {
    let transform = Transform {
        translation,
        rotation: Quat::from_rotation_y(rotation_y),
        ..default()
    };
    let camera_obstacle = camera_half_extents.map(|half_extents| {
        let half_extents = rotated_aabb_half_extents(half_extents, rotation_y);
        CameraObstacle(CameraObstruction::soft_local_prop(
            translation + Vec3::Y * half_extents.y,
            half_extents,
        ))
    });
    if let Some(half_extents) = collision_half_extents {
        let half_extents = rotated_aabb_half_extents(half_extents, rotation_y);
        let collision_center = translation + Vec3::Y * half_extents.y;
        queue_collidable_island_visual(
            entries,
            visual_index,
            island,
            layer,
            mesh,
            material,
            transform,
            camera_obstacle,
            WorldCollisionProxy::new(
                collision_center,
                half_extents,
                WorldCollisionProxyKind::Landmark,
            ),
            name,
        );
    } else {
        queue_island_visual(
            entries,
            visual_index,
            island,
            layer,
            mesh,
            material,
            transform,
            camera_obstacle,
            name,
        );
    }
}

fn rotated_aabb_half_extents(half_extents: Vec3, rotation_y: f32) -> Vec3 {
    let cosine = rotation_y.cos().abs();
    let sine = rotation_y.sin().abs();
    Vec3::new(
        cosine * half_extents.x + sine * half_extents.z,
        half_extents.y,
        sine * half_extents.x + cosine * half_extents.z,
    )
}

impl IslandVisualCatalog {
    pub(crate) fn apply_authored_detail_materials(
        &mut self,
        route: &SkyRoute,
        images: &mut Assets<Image>,
        materials: &mut Assets<StandardMaterial>,
        stream_state: &IslandStreamState,
    ) -> (SpawnedMaterialUpdates, IslandStoneMaterials) {
        let shared_materials = route
            .islands()
            .iter()
            .take(TERRAIN_BIOME_PALETTE_COUNT)
            .map(|island| {
                detail_materials_for_island(&self.entries, island.name).unwrap_or_else(|| {
                    panic!("{} is missing queued runtime detail materials", island.name)
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            shared_materials.len(),
            TERRAIN_BIOME_PALETTE_COUNT,
            "runtime detail palettes must cover the shared material set"
        );

        let authored_materials = allocate_authored_island_detail_materials(
            images,
            materials,
            &shared_materials,
            route.islands().len(),
        );
        let mut spawned_updates = Vec::new();

        for (island_index, island) in route.islands().iter().enumerate() {
            let source = &shared_materials[island_index % shared_materials.len()];
            let target = &authored_materials[island_index];
            for entry in self
                .entries
                .iter_mut()
                .filter(|entry| entry.key.island_name == island.name)
            {
                let Some(current) = entry.material.as_ref() else {
                    continue;
                };
                let Some(replacement) = replacement_detail_material(current, source, target) else {
                    continue;
                };
                if &replacement == current {
                    continue;
                }

                entry.material = Some(replacement.clone());
                if stream_state.visual_resident.contains(&entry.key)
                    && let Some(entity) = stream_state.spawned.get(&entry.key)
                {
                    spawned_updates.push((*entity, replacement));
                }
            }
        }

        let obstruction_stones = route
            .islands()
            .iter()
            .enumerate()
            .map(|(island_index, island)| {
                (island.name, authored_materials[island_index].stone.clone())
            })
            .collect();
        (spawned_updates, obstruction_stones)
    }

    #[cfg(test)]
    pub(crate) fn entry_material_handle(
        &self,
        island_name: &'static str,
        entry_name: &'static str,
    ) -> Option<Handle<StandardMaterial>> {
        self.entries
            .iter()
            .find(|entry| {
                entry.key.island_name == island_name
                    && entry.name == entry_name
                    && entry.has_visible_mesh()
            })
            .and_then(|entry| entry.material.clone())
    }

    #[cfg(test)]
    pub(crate) fn spawned_entry_material_handle(
        world: &World,
        island_name: &'static str,
        entry_name: &'static str,
    ) -> Option<Handle<StandardMaterial>> {
        let catalog = world.get_resource::<Self>()?;
        let entry = catalog.entries.iter().find(|entry| {
            entry.key.island_name == island_name
                && entry.name == entry_name
                && entry.has_visible_mesh()
        })?;
        let stream_state = world.get_resource::<IslandStreamState>()?;
        let entity = *stream_state.spawned.get(&entry.key)?;
        world
            .get::<MeshMaterial3d<StandardMaterial>>(entity)
            .map(|material| material.0.clone())
    }
}

fn detail_materials_for_island(
    entries: &[IslandVisualEntry],
    island_name: &'static str,
) -> Option<IslandDetailMaterials> {
    let material_for = |predicate: &dyn Fn(&IslandVisualEntry) -> bool| {
        entries
            .iter()
            .find(|entry| entry.key.island_name == island_name && predicate(entry))
            .and_then(|entry| entry.material.clone())
    };

    Some(IslandDetailMaterials {
        trunk: material_for(&|entry| entry.name.ends_with(" trunk"))?,
        foliage: material_for(&|entry| entry.name.ends_with(" canopy"))?,
        ground_cover: material_for(&|entry| entry.name == "island ground cover")?,
        stone: material_for(&|entry| entry.name == "island stone scatter")?,
    })
}

fn replacement_detail_material(
    current: &Handle<StandardMaterial>,
    source: &IslandDetailMaterials,
    target: &IslandDetailMaterials,
) -> Option<Handle<StandardMaterial>> {
    if current == &source.trunk {
        Some(target.trunk.clone())
    } else if current == &source.foliage {
        Some(target.foliage.clone())
    } else if current == &source.ground_cover {
        Some(target.ground_cover.clone())
    } else if current == &source.stone {
        Some(target.stone.clone())
    } else {
        None
    }
}
