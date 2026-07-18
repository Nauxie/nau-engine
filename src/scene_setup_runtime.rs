mod constants;
mod hud;
mod materials;
mod player;
mod world;

use bevy::pbr::ScatteringMedium;
use bevy::prelude::*;

use crate::authored_assets::{VisualAssetRegistry, prepare_visual_asset_registry};
use crate::camera_runtime::{spawn_follow_camera, spawn_follow_camera_with_settings};
use crate::eval_runtime::{EvalRun, ISLAND_HERO_GALLERY, RunMode};
use crate::game_ui_runtime::{GameUiState, spawn_game_ui};
use crate::island_visuals::{IslandStreamState, IslandVisualCatalog};
use crate::play_profile_runtime::PlayProfileRun;
use crate::scene_setup_runtime::hud::spawn_debug_readout;
use crate::scene_setup_runtime::materials::prepare_scene_materials;
use crate::scene_setup_runtime::player::spawn_player_runtime;
use crate::scene_setup_runtime::world::spawn_world_runtime;
use crate::surface_material::SurfaceMaterial;
use crate::{Player, PlayerDisplacementDiagnostics};
use nau_engine::animation::{AnimationState, PlayerPoseIntent};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::camera::FollowCamera;
use nau_engine::eval::{
    GREAT_SKY_PLATEAU_VISTAS, ISLAND_SURFACE_REVIEW, PLATEAU_ARRIVAL_CAMERA,
    TERRAIN_BODY_COLLISION_CONTACT, TERRAIN_EDGE_WALKOFF, TERRAIN_RIM_COLLISION_CONTACT,
    UNDERBRIDGE_UNDER_ROUTE,
};
use nau_engine::movement::{FlightController, FlightMode, Velocity};
use nau_engine::world::{
    IslandReviewView, SkyRoute, WorldCollisionProxyKind, route_obstruction_spires,
    terrain_collision_contact_probe_position,
};

pub(crate) use constants::{INITIAL_SKY_CLEAR_COLOR, PLAYER_START, WORLD_RADIUS};

const PLATEAU_CAMERA_START_BACKOFF_M: f32 = 7.0;

#[allow(clippy::too_many_arguments)]
pub(crate) fn setup(
    mut commands: Commands,
    route: Res<SkyRoute>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut surface_materials: ResMut<Assets<SurfaceMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    asset_server: Res<AssetServer>,
    eval_run: Option<Res<EvalRun>>,
    play_profile: Option<Res<PlayProfileRun>>,
    run_mode: Res<RunMode>,
    game_ui: Res<GameUiState>,
) {
    let mut visual_asset_registry = prepare_visual_asset_registry(&asset_server);
    let player_scene_handle = visual_asset_registry.scene_handle(VisualAssetKind::PlayerCharacter);
    let glider_scene_handle = visual_asset_registry.scene_handle(VisualAssetKind::Glider);
    let scene_materials =
        prepare_scene_materials(&mut images, &mut materials, &mut surface_materials);
    let screenshot_eval = eval_run
        .as_deref()
        .is_some_and(|run| run.screenshot_path.is_some());
    let player_start =
        initial_player_position(eval_run.as_deref(), play_profile.as_deref(), &route);
    let authored_world_fixture_scene_entities = spawn_world_runtime(
        &mut commands,
        &route,
        &mut meshes,
        &scene_materials,
        &visual_asset_registry,
        player_start,
    );
    let player_scene_entities = spawn_player_runtime(
        &mut commands,
        &mut meshes,
        &scene_materials,
        player_scene_handle,
        glider_scene_handle,
        player_start,
    );

    mark_spawned_scenes(
        &mut visual_asset_registry,
        player_scene_entities.player_scene_entity,
        player_scene_entities.glider_scene_entity,
        authored_world_fixture_scene_entities,
    );
    commands.insert_resource(visual_asset_registry);

    if eval_run.as_deref().is_some_and(|run| {
        matches!(
            run.scenario.name,
            GREAT_SKY_PLATEAU_VISTAS | ISLAND_SURFACE_REVIEW | ISLAND_HERO_GALLERY
        )
    }) {
        spawn_follow_camera_with_settings(
            &mut commands,
            &mut scattering_mediums,
            player_start,
            plateau_vista_follow_camera(),
            WORLD_RADIUS,
            INITIAL_SKY_CLEAR_COLOR,
        );
    } else {
        spawn_follow_camera(
            &mut commands,
            &mut scattering_mediums,
            player_start,
            WORLD_RADIUS,
            INITIAL_SKY_CLEAR_COLOR,
        );
    }

    if run_mode.debug_readout_enabled() && !screenshot_eval {
        spawn_debug_readout(&mut commands);
    }
    spawn_game_ui(&mut commands, &game_ui);
}

pub(crate) fn apply_authored_island_material_parity(
    route: Res<SkyRoute>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut catalog: ResMut<IslandVisualCatalog>,
    stream_state: Res<IslandStreamState>,
    mut spawned_materials: Query<(Option<&Name>, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let (spawned_updates, obstruction_stones) =
        catalog.apply_authored_detail_materials(&route, &mut images, &mut materials, &stream_state);

    for (entity, replacement) in spawned_updates {
        if let Ok((_, mut material)) = spawned_materials.get_mut(entity) {
            material.0 = replacement;
        }
    }

    for (name, mut material) in &mut spawned_materials {
        let Some(island_name) = name
            .map(Name::as_str)
            .and_then(|name| name.strip_suffix(" obstruction spire"))
        else {
            continue;
        };
        if let Some((_, replacement)) = obstruction_stones
            .iter()
            .find(|(candidate, _)| *candidate == island_name)
        {
            material.0 = replacement.clone();
        }
    }
}

pub(crate) fn fix_island_hero_gallery_player(
    run: Res<EvalRun>,
    mut displacement: ResMut<PlayerDisplacementDiagnostics>,
    mut player: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut FlightController,
            &mut AnimationState,
        ),
        With<Player>,
    >,
) {
    if run.scenario.name != ISLAND_HERO_GALLERY {
        return;
    }
    let Some(pose) = run.island_review_pose() else {
        return;
    };
    let Ok((mut transform, mut velocity, mut controller, mut animation)) = player.single_mut()
    else {
        return;
    };
    let (mode, pose_intent) = match pose.view {
        IslandReviewView::Near => (FlightMode::Grounded, PlayerPoseIntent::GroundedIdle),
        IslandReviewView::Mid | IslandReviewView::Traversal => {
            (FlightMode::Gliding, PlayerPoseIntent::Gliding)
        }
    };

    transform.translation = pose.player_position;
    transform.rotation = Transform::from_translation(pose.player_position)
        .looking_at(pose.camera_target, Vec3::Y)
        .rotation;
    velocity.0 = Vec3::ZERO;
    *controller = FlightController { mode, ..default() };
    *animation = AnimationState {
        pose_intent,
        ..default()
    };
    *displacement = PlayerDisplacementDiagnostics::default();
}

fn initial_player_position(
    eval_run: Option<&EvalRun>,
    play_profile: Option<&PlayProfileRun>,
    route: &SkyRoute,
) -> Vec3 {
    if let Some(pose) = eval_run.and_then(EvalRun::island_review_pose) {
        return pose.player_position;
    }
    if eval_run.is_some_and(|run| run.scenario.name == TERRAIN_RIM_COLLISION_CONTACT) {
        return terrain_collision_eval_start_position(
            route,
            WorldCollisionProxyKind::TerrainRim,
            Vec2::new(1.0, 0.75),
        );
    }
    if eval_run.is_some_and(|run| run.scenario.name == TERRAIN_BODY_COLLISION_CONTACT) {
        return terrain_collision_eval_start_position(
            route,
            WorldCollisionProxyKind::TerrainBody,
            Vec2::X,
        );
    }
    if eval_run.is_some_and(|run| run.scenario.name == TERRAIN_EDGE_WALKOFF) {
        return terrain_edge_walkoff_start_position(route);
    }
    if eval_run.is_some_and(|run| run.scenario.name == UNDERBRIDGE_UNDER_ROUTE) {
        return underbridge_under_route_start_position(route);
    }
    if eval_run.is_some_and(|run| run.scenario.name == PLATEAU_ARRIVAL_CAMERA) {
        return plateau_arrival_camera_start_position(route);
    }
    if eval_run.is_some_and(|run| run.scenario.name == ISLAND_SURFACE_REVIEW) {
        return route.playtest_reset_position();
    }
    if eval_run.is_some_and(|run| run.scenario.name == GREAT_SKY_PLATEAU_VISTAS) {
        return plateau_vista_start_position(route);
    }
    if eval_run.is_none()
        && let Some(position) =
            play_profile.and_then(|profile| profile.scripted_start_position(route))
    {
        return position;
    }

    PLAYER_START
}

fn terrain_collision_eval_start_position(
    route: &SkyRoute,
    kind: WorldCollisionProxyKind,
    preferred_outward: Vec2,
) -> Vec3 {
    route
        .island_named("launch mesa")
        .and_then(|island| {
            terrain_collision_contact_probe_position(island, kind, preferred_outward)
        })
        .unwrap_or(PLAYER_START)
}

fn terrain_edge_walkoff_start_position(route: &SkyRoute) -> Vec3 {
    let Some(island) = route.island_named("launch mesa") else {
        return PLAYER_START;
    };
    let contour = island.footprint_contour_point(0.0, false);
    let outward = (contour - Vec2::new(island.center.x, island.center.z)).normalize_or_zero();
    let horizontal = contour - outward * 8.0;
    let mut position = Vec3::new(horizontal.x, 0.0, horizontal.y);
    position.y = island.terrain_surface_y_at(position);
    position
}

fn underbridge_under_route_start_position(route: &SkyRoute) -> Vec3 {
    route
        .under_island_route_segments()
        .into_iter()
        .find(|segment| segment.island_name == "underbridge cay")
        .map(|segment| segment.exit + Vec3::NEG_Z * 8.0)
        .unwrap_or(PLAYER_START)
}

fn plateau_arrival_camera_start_position(route: &SkyRoute) -> Vec3 {
    let mut position = route_obstruction_spires(route)
        .into_iter()
        .find(|spire| spire.island_name == "great sky plateau")
        .map(|spire| spire.base_position + Vec3::NEG_Z * PLATEAU_CAMERA_START_BACKOFF_M)
        .unwrap_or_else(|| route.playtest_reset_position());
    position.y = route.ground_at(position).floor_y;
    position
}

fn plateau_vista_start_position(route: &SkyRoute) -> Vec3 {
    let Some(plateau) = route.island_named("great sky plateau") else {
        return route.playtest_reset_position();
    };
    let mut position = plateau
        .plateau_region_position(nau_engine::world::IslandPlateauRegion::BrokenEdge)
        .unwrap_or(plateau.center);
    position.y = route.ground_at(position + Vec3::Y * 1_000.0).floor_y;
    position
}

fn plateau_vista_follow_camera() -> FollowCamera {
    FollowCamera {
        distance: 24.0,
        height: 11.0,
        look_height: 2.4,
        look_ahead: 70.0,
        position_smoothing: 16.0,
        rotation_smoothing: 18.0,
        direction_smoothing: 1.0,
        min_height: 2.0,
    }
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
