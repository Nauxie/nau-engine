mod constants;
mod hud;
mod materials;
mod player;
mod world;

use bevy::pbr::ScatteringMedium;
use bevy::prelude::*;

use crate::authored_assets::{VisualAssetRegistry, prepare_visual_asset_registry};
use crate::camera_runtime::spawn_follow_camera;
use crate::scene_setup_runtime::hud::spawn_debug_readout;
use crate::scene_setup_runtime::materials::prepare_scene_materials;
use crate::scene_setup_runtime::player::spawn_player_runtime;
use crate::scene_setup_runtime::world::spawn_world_runtime;
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::world::SkyRoute;

pub(crate) use constants::{INITIAL_SKY_CLEAR_COLOR, PLAYER_START, WORLD_RADIUS};

pub(crate) fn setup(
    mut commands: Commands,
    route: Res<SkyRoute>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    asset_server: Res<AssetServer>,
) {
    let mut visual_asset_registry = prepare_visual_asset_registry(&asset_server);
    let player_scene_handle = visual_asset_registry.scene_handle(VisualAssetKind::PlayerCharacter);
    let glider_scene_handle = visual_asset_registry.scene_handle(VisualAssetKind::Glider);
    let scene_materials = prepare_scene_materials(&mut images, &mut materials);
    let authored_world_fixture_scene_entities = spawn_world_runtime(
        &mut commands,
        &route,
        &mut meshes,
        &scene_materials,
        &visual_asset_registry,
    );
    let player_scene_entities = spawn_player_runtime(
        &mut commands,
        &mut meshes,
        &scene_materials,
        player_scene_handle,
        glider_scene_handle,
    );

    mark_spawned_scenes(
        &mut visual_asset_registry,
        player_scene_entities.player_scene_entity,
        player_scene_entities.glider_scene_entity,
        authored_world_fixture_scene_entities,
    );
    commands.insert_resource(visual_asset_registry);

    spawn_follow_camera(
        &mut commands,
        &mut scattering_mediums,
        PLAYER_START,
        WORLD_RADIUS,
        INITIAL_SKY_CLEAR_COLOR,
    );

    spawn_debug_readout(&mut commands);
}

fn mark_spawned_scenes(
    visual_asset_registry: &mut VisualAssetRegistry,
    player_scene_entity: Option<Entity>,
    glider_scene_entity: Option<Entity>,
    authored_world_fixture_scene_entities: Vec<(VisualAssetKind, Entity)>,
) {
    if let Some(entity) = player_scene_entity {
        visual_asset_registry.mark_scene_spawned(VisualAssetKind::PlayerCharacter, entity);
    }
    if let Some(entity) = glider_scene_entity {
        visual_asset_registry.mark_scene_spawned(VisualAssetKind::Glider, entity);
    }
    for (kind, entity) in authored_world_fixture_scene_entities {
        visual_asset_registry.mark_scene_spawned(kind, entity);
    }
}
