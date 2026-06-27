use super::details::queue_sky_island_details;
use super::types::{
    IslandVisualCatalog, IslandVisualEntry, IslandVisualKey, IslandVisualLayer,
    IslandVisualMeshRecipe,
};
use crate::camera_runtime::CameraObstacle;
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::generated_content::{
    ISLAND_BODY_SEGMENTS, IslandDetailMaterials, island_body_mesh_diagnostics,
    island_impostor_mesh_diagnostics, island_terrain_mesh_diagnostics,
    island_visual_surface_position,
};
use crate::world_collision_runtime::{
    WorldCollisionProxy, WorldCollisionProxyKind, terrain_rim_collision_proxies,
};
use bevy::prelude::*;
use nau_engine::camera::CameraObstruction;
use nau_engine::world::{SkyIsland, is_recovery_branch_island};

#[allow(clippy::too_many_arguments)]
pub(super) fn queue_island_visual(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    name: &'static str,
) {
    queue_island_visual_with_motion(
        entries,
        visual_index,
        island,
        layer,
        Some(mesh),
        None,
        Some(material),
        transform,
        obstacle,
        None,
        None,
        name,
    );
}

#[allow(clippy::too_many_arguments)]
fn queue_generated_island_visual(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh_recipe: IslandVisualMeshRecipe,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    name: &'static str,
) {
    queue_island_visual_with_motion(
        entries,
        visual_index,
        island,
        layer,
        None,
        Some(mesh_recipe),
        Some(material),
        transform,
        obstacle,
        None,
        None,
        name,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn queue_collidable_island_visual(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    collision: WorldCollisionProxy,
    name: &'static str,
) {
    queue_island_visual_with_motion(
        entries,
        visual_index,
        island,
        layer,
        Some(mesh),
        None,
        Some(material),
        transform,
        obstacle,
        Some(collision),
        None,
        name,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn queue_wind_island_visual(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    wind_motion: crate::environment_visuals::WindVisualMotion,
    name: &'static str,
) {
    queue_island_visual_with_motion(
        entries,
        visual_index,
        island,
        layer,
        Some(mesh),
        None,
        Some(material),
        transform,
        obstacle,
        None,
        Some(wind_motion),
        name,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn queue_collidable_wind_island_visual(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    collision: WorldCollisionProxy,
    wind_motion: crate::environment_visuals::WindVisualMotion,
    name: &'static str,
) {
    queue_island_visual_with_motion(
        entries,
        visual_index,
        island,
        layer,
        Some(mesh),
        None,
        Some(material),
        transform,
        obstacle,
        Some(collision),
        Some(wind_motion),
        name,
    );
}

#[allow(clippy::too_many_arguments)]
fn queue_island_visual_with_motion(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Option<Handle<Mesh>>,
    mesh_recipe: Option<IslandVisualMeshRecipe>,
    material: Option<Handle<StandardMaterial>>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    collision: Option<WorldCollisionProxy>,
    wind_motion: Option<crate::environment_visuals::WindVisualMotion>,
    name: &'static str,
) {
    let key = IslandVisualKey {
        island_name: island.name,
        layer,
        index: *visual_index,
    };
    *visual_index += 1;

    entries.push(IslandVisualEntry {
        key,
        island,
        layer,
        mesh,
        mesh_recipe,
        material,
        transform,
        obstacle,
        collision,
        wind_motion,
        name,
    });
}

fn queue_collision_only_island_proxy(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    collision: WorldCollisionProxy,
    name: &'static str,
) {
    queue_island_visual_with_motion(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Collision,
        None,
        None,
        None,
        Transform::from_translation(collision.center),
        None,
        Some(collision),
        None,
        name,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_sky_island(
    catalog: &mut IslandVisualCatalog,
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    top_material: Handle<StandardMaterial>,
    rock_material: Handle<StandardMaterial>,
    under_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    branch_marker_material: Handle<StandardMaterial>,
    detail_materials: IslandDetailMaterials,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let top_y = island.mesh_top_y();
    let mut visual_index = 0;
    let entries = &mut catalog.entries;

    content_diagnostics.record_island_terrain_archetype(island.terrain_archetype);

    let impostor_diagnostics = island_impostor_mesh_diagnostics(island_index, island);
    content_diagnostics.record_island_impostor(
        impostor_diagnostics.vertex_count,
        impostor_diagnostics.color_bands,
    );
    queue_generated_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Impostor,
        IslandVisualMeshRecipe::Impostor {
            island_index,
            island,
        },
        top_material.clone(),
        Transform::default(),
        None,
        "island distant impostor",
    );

    let terrain_diagnostics = island_terrain_mesh_diagnostics(island_index, island);
    content_diagnostics.record_island_terrain_surface(
        terrain_diagnostics.vertex_count,
        terrain_diagnostics.color_bands,
        terrain_diagnostics.material_weight_bands,
        terrain_diagnostics.material_channels,
        terrain_diagnostics.material_regions,
        terrain_diagnostics.relief_range_m,
    );
    queue_generated_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        IslandVisualMeshRecipe::Terrain {
            island_index,
            island,
        },
        top_material,
        Transform::default(),
        None,
        "island terrain surface",
    );

    let rock_body_center = Vec3::new(
        island.center.x,
        top_y - island.thickness * 0.54,
        island.center.z,
    );
    let rock_body_half_extents = Vec3::new(
        island.half_extents.x * 0.78,
        island.thickness * 0.5,
        island.half_extents.y * 0.78,
    );
    let body_diagnostics = island_body_mesh_diagnostics(island_index, island);
    content_diagnostics.record_island_cliff_detail(body_diagnostics.cliff_color_bands);
    queue_generated_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        IslandVisualMeshRecipe::Cliff {
            island_index,
            island,
        },
        rock_material,
        Transform::default(),
        Some(CameraObstacle(CameraObstruction::new(
            rock_body_center,
            rock_body_half_extents,
        ))),
        "island procedural cliff body",
    );

    content_diagnostics.record_island_cliff_detail(body_diagnostics.underside_color_bands);
    queue_generated_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        IslandVisualMeshRecipe::Underside {
            island_index,
            island,
        },
        under_material.clone(),
        Transform::default(),
        None,
        "island tapered underside",
    );
    content_diagnostics
        .record_procedural_island_body(ISLAND_BODY_SEGMENTS, body_diagnostics.total_vertex_count());
    for collision in terrain_rim_collision_proxies(island) {
        queue_collision_only_island_proxy(
            entries,
            &mut visual_index,
            island,
            collision,
            "island terrain rim collision",
        );
    }

    let ridge_width = island.half_extents.x * 0.32;
    let ridge_surface = island_visual_surface_position(island, Vec2::new(0.28, -0.24));
    let ridge_center = ridge_surface + Vec3::Y * 0.375;
    let ridge_half_extents = Vec3::new(ridge_width * 0.5, 0.375, island.half_extents.y * 0.09);
    queue_collidable_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(Cuboid::new(ridge_width, 0.75, island.half_extents.y * 0.18)),
        under_material,
        Transform::from_translation(ridge_center),
        Some(CameraObstacle(CameraObstruction::new(
            ridge_center,
            ridge_half_extents,
        ))),
        WorldCollisionProxy::new(
            ridge_center,
            ridge_half_extents,
            WorldCollisionProxyKind::Landmark,
        ),
        "island ridge",
    );

    if island.is_target {
        let marker_center = Vec3::new(
            island.center.x,
            island.mesh_top_y_at(island.center) + 1.8,
            island.center.z,
        );
        queue_collidable_island_visual(
            entries,
            &mut visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(Cuboid::new(2.2, 6.0, 2.2)),
            marker_material,
            Transform::from_translation(marker_center),
            Some(CameraObstacle(CameraObstruction::new(
                marker_center,
                Vec3::new(1.1, 3.0, 1.1),
            ))),
            WorldCollisionProxy::new(
                marker_center,
                Vec3::new(1.1, 3.0, 1.1),
                WorldCollisionProxyKind::Landmark,
            ),
            "landing target marker",
        );
    }
    if is_recovery_branch_island(island.name) {
        queue_recovery_branch_marker(
            entries,
            &mut visual_index,
            meshes,
            branch_marker_material,
            island,
        );
    }

    queue_sky_island_details(
        entries,
        &mut visual_index,
        content_diagnostics,
        meshes,
        detail_materials,
        flower_material,
        water_material,
        island_index,
        island,
    );
}

fn queue_recovery_branch_marker(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    meshes: &mut Assets<Mesh>,
    marker_material: Handle<StandardMaterial>,
    island: SkyIsland,
) {
    let mast_height = 5.6;
    let mast_surface = island_visual_surface_position(island, Vec2::new(-0.08, 0.08));
    let mast_center = mast_surface + Vec3::Y * (mast_height * 0.5);
    queue_collidable_island_visual(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Beacon,
        meshes.add(Cylinder::new(0.42, mast_height)),
        marker_material.clone(),
        Transform::from_translation(mast_center),
        None,
        WorldCollisionProxy::new(
            mast_center,
            Vec3::new(0.42, mast_height * 0.5, 0.42),
            WorldCollisionProxyKind::Landmark,
        ),
        "recovery branch mast",
    );

    let ring_size = 7.2;
    for (offset, scale) in [
        (
            Vec3::new(0.0, 0.09, ring_size * 0.5),
            Vec3::new(ring_size, 0.12, 0.34),
        ),
        (
            Vec3::new(0.0, 0.09, -ring_size * 0.5),
            Vec3::new(ring_size, 0.12, 0.34),
        ),
        (
            Vec3::new(ring_size * 0.5, 0.09, 0.0),
            Vec3::new(0.34, 0.12, ring_size),
        ),
        (
            Vec3::new(-ring_size * 0.5, 0.09, 0.0),
            Vec3::new(0.34, 0.12, ring_size),
        ),
    ] {
        let surface_y = island.mesh_top_y_at(island.center + Vec3::new(offset.x, 0.0, offset.z));
        let ring_center = Vec3::new(
            island.center.x + offset.x,
            surface_y + offset.y,
            island.center.z + offset.z,
        );
        let ring_half_extents = scale * 0.5;
        queue_collidable_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(Cuboid::new(scale.x, scale.y, scale.z)),
            marker_material.clone(),
            Transform::from_translation(ring_center),
            Some(CameraObstacle(CameraObstruction::new(
                ring_center,
                ring_half_extents,
            ))),
            WorldCollisionProxy::new(
                ring_center,
                ring_half_extents,
                WorldCollisionProxyKind::Landmark,
            ),
            "recovery branch ring",
        );
    }
}
