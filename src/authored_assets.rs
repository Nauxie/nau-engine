use crate::Player;
use crate::generated_content::island_visual_surface_position;
use bevy::animation::graph::AnimationNodeIndex;
use bevy::asset::LoadState;
use bevy::gltf::{Gltf, GltfAssetLabel};
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use nau_engine::asset_pipeline::{
    DEFAULT_VISUAL_ASSET_LOAD_POLICY, VISUAL_ASSET_SPECS, VisualAssetAnimationState,
    VisualAssetKind, VisualAssetLoadAdmission, VisualAssetLoadState, VisualAssetPipelineMetrics,
    VisualAssetPreloadState, VisualAssetSceneState, VisualAssetSpec,
    visual_asset_load_admission_plan, visual_asset_pipeline_metrics_with_preload_states,
};
use nau_engine::movement::{FlightController, FlightMode, Velocity};
use nau_engine::world::{SkyIsland, SkyRoute};
#[cfg(test)]
use std::collections::HashMap;
use std::{collections::HashSet, path::Path, time::Duration};

pub(crate) const AUTHORED_WORLD_FIXTURE_KINDS: &[VisualAssetKind] = &[
    VisualAssetKind::IslandTerrain,
    VisualAssetKind::IslandFoliage,
    VisualAssetKind::IslandRock,
    VisualAssetKind::IslandWater,
    VisualAssetKind::RouteMarker,
    VisualAssetKind::WeatherLayer,
    VisualAssetKind::DistantImpostor,
];

#[derive(Resource, Debug)]
pub(crate) struct VisualAssetRegistry {
    pub(crate) slots: Vec<VisualAssetSlot>,
}

impl VisualAssetRegistry {
    pub(crate) fn scene_handle(&self, kind: VisualAssetKind) -> Option<Handle<Scene>> {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == kind)
            .and_then(|slot| slot.scene_handle.clone())
    }

    pub(crate) fn mark_scene_spawned(&mut self, kind: VisualAssetKind, entity: Entity) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.scene_entity = Some(entity);
        }
    }

    pub(crate) fn mark_scene_ready(&mut self, kind: VisualAssetKind) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.scene_ready = true;
        }
    }

    pub(crate) fn scene_ready(&self, kind: VisualAssetKind) -> bool {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == kind)
            .is_some_and(|slot| slot.scene_ready)
    }

    pub(crate) fn mark_animation_player_linked(
        &mut self,
        kind: VisualAssetKind,
        entity: Entity,
        ready_clip_count: usize,
    ) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.animation_player_entity = Some(entity);
            slot.ready_animation_clip_count =
                ready_clip_count.min(slot.spec.animation_clip_names.len());
        }
    }

    pub(crate) fn mark_animation_graph_ready(
        &mut self,
        kind: VisualAssetKind,
        entity: Entity,
        ready_clip_count: usize,
    ) {
        self.mark_animation_player_linked(kind, entity, ready_clip_count);
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.animation_graph_ready = true;
        }
    }

    pub(crate) fn pending_animation_links(&self) -> Vec<PendingAuthoredAnimationLink> {
        self.slots
            .iter()
            .filter(|slot| {
                slot.scene_ready
                    && !slot.animation_graph_ready
                    && !slot.spec.animation_clip_names.is_empty()
            })
            .filter_map(|slot| {
                Some(PendingAuthoredAnimationLink {
                    kind: slot.spec.kind,
                    spec: slot.spec,
                    scene_entity: slot.scene_entity?,
                    gltf_handle: slot.gltf_handle.clone()?,
                })
            })
            .collect()
    }

    pub(crate) fn scene_state_for(&self, spec: &VisualAssetSpec) -> VisualAssetSceneState {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == spec.kind)
            .map_or(VisualAssetSceneState::NotSpawned, |slot| {
                if slot.scene_ready {
                    VisualAssetSceneState::Ready
                } else if slot.scene_entity.is_some() {
                    VisualAssetSceneState::Spawned
                } else {
                    VisualAssetSceneState::NotSpawned
                }
            })
    }

    pub(crate) fn animation_state_for(&self, spec: &VisualAssetSpec) -> VisualAssetAnimationState {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == spec.kind)
            .map_or(VisualAssetAnimationState::default(), |slot| {
                VisualAssetAnimationState {
                    ready_clip_count: slot.ready_animation_clip_count,
                    animation_player_linked: slot.animation_player_entity.is_some(),
                    animation_graph_ready: slot.animation_graph_ready,
                }
            })
    }
}

#[derive(Debug)]
pub(crate) struct VisualAssetSlot {
    pub(crate) spec: VisualAssetSpec,
    pub(crate) load_admission: VisualAssetLoadAdmission,
    pub(crate) gltf_handle: Option<Handle<Gltf>>,
    pub(crate) scene_handle: Option<Handle<Scene>>,
    pub(crate) scene_entity: Option<Entity>,
    pub(crate) scene_ready: bool,
    pub(crate) animation_player_entity: Option<Entity>,
    pub(crate) ready_animation_clip_count: usize,
    pub(crate) animation_graph_ready: bool,
}

#[derive(Clone)]
pub(crate) struct PendingAuthoredAnimationLink {
    pub(crate) kind: VisualAssetKind,
    pub(crate) spec: VisualAssetSpec,
    pub(crate) scene_entity: Entity,
    pub(crate) gltf_handle: Handle<Gltf>,
}

#[derive(Debug)]
pub(crate) struct NamedAnimationClipResolution {
    pub(crate) clips: Vec<Handle<AnimationClip>>,
    pub(crate) expected_clip_count: usize,
    pub(crate) missing_clip_names: Vec<&'static str>,
}

impl NamedAnimationClipResolution {
    pub(crate) fn ready_clip_count(&self) -> usize {
        self.clips.len()
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.ready_clip_count() == self.expected_clip_count && self.missing_clip_names.is_empty()
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct VisualAssetDiagnostics {
    pub(crate) metrics: VisualAssetPipelineMetrics,
    pub(crate) visible_world_fixture_count: usize,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredVisualScene {
    pub(crate) kind: VisualAssetKind,
    pub(crate) role: AuthoredVisualSceneRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuthoredVisualSceneRole {
    PlayerRuntime,
    GliderRuntime,
    WorldFixture,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct VisibleAuthoredWorldFixture {
    pub(crate) kind: VisualAssetKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuthoredPlayerClip {
    Idle,
    Jog,
    Launch,
    Glide,
    AirBrake,
    Land,
}

impl AuthoredPlayerClip {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Jog => 1,
            Self::Launch => 2,
            Self::Glide => 3,
            Self::AirBrake => 4,
            Self::Land => 5,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredPlayerAnimation {
    pub(crate) nodes: [AnimationNodeIndex; 6],
    pub(crate) current: AuthoredPlayerClip,
}

impl AuthoredPlayerAnimation {
    pub(crate) fn new(nodes: [AnimationNodeIndex; 6], current: AuthoredPlayerClip) -> Self {
        Self { nodes, current }
    }

    pub(crate) fn node(self, clip: AuthoredPlayerClip) -> AnimationNodeIndex {
        self.nodes[clip.index()]
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct GeneratedPlayerPlaceholder;

pub(crate) fn prepare_visual_asset_registry(asset_server: &AssetServer) -> VisualAssetRegistry {
    let admissions = visual_asset_load_admission_plan(
        &VISUAL_ASSET_SPECS,
        |spec| visual_asset_path_exists(spec.gltf_scene_path),
        DEFAULT_VISUAL_ASSET_LOAD_POLICY,
    );
    let slots = VISUAL_ASSET_SPECS
        .iter()
        .copied()
        .zip(admissions)
        .map(|(spec, load_admission)| VisualAssetSlot {
            spec,
            load_admission,
            gltf_handle: load_admission
                .is_admitted()
                .then(|| asset_server.load(spec.gltf_scene_path)),
            scene_handle: load_admission.is_admitted().then(|| {
                asset_server.load(GltfAssetLabel::Scene(0).from_asset(spec.gltf_scene_path))
            }),
            scene_entity: None,
            scene_ready: false,
            animation_player_entity: None,
            ready_animation_clip_count: 0,
            animation_graph_ready: false,
        })
        .collect();

    VisualAssetRegistry { slots }
}

pub(crate) fn visual_asset_path_exists(asset_path: &str) -> bool {
    Path::new("assets").join(asset_path).is_file()
}

pub(crate) fn authored_world_fixture_scene_handles(
    registry: &VisualAssetRegistry,
) -> Vec<(VisualAssetKind, &'static str, Handle<Scene>)> {
    AUTHORED_WORLD_FIXTURE_KINDS
        .iter()
        .filter_map(|kind| {
            registry
                .slots
                .iter()
                .find(|slot| slot.spec.kind == *kind)
                .and_then(|slot| {
                    slot.scene_handle
                        .clone()
                        .map(|scene_handle| (*kind, slot.spec.label, scene_handle))
                })
        })
        .collect()
}

pub(crate) fn authored_world_fixture_transform(
    kind: VisualAssetKind,
    route: &SkyRoute,
) -> Transform {
    let Some((island, normalized_offset, surface_offset_y, scale, yaw_radians)) =
        authored_world_fixture_layout(kind, route.islands())
    else {
        return Transform::from_xyz(-140.0, -80.0, 140.0);
    };
    let surface = island_visual_surface_position(island, normalized_offset);

    Transform {
        translation: surface + Vec3::Y * surface_offset_y,
        rotation: Quat::from_rotation_y(yaw_radians),
        scale: Vec3::splat(scale),
    }
}

pub(crate) fn authored_world_fixture_layout(
    kind: VisualAssetKind,
    islands: &[SkyIsland],
) -> Option<(SkyIsland, Vec2, f32, f32, f32)> {
    let (island_index, normalized_offset, surface_offset_y, scale, yaw_radians) = match kind {
        VisualAssetKind::IslandTerrain => (0, Vec2::new(0.34, -0.34), 0.08, 0.82, 0.35),
        VisualAssetKind::IslandFoliage => (0, Vec2::new(-0.42, -0.2), 0.02, 2.2, -0.2),
        VisualAssetKind::IslandRock => (1, Vec2::new(0.3, -0.26), 0.08, 1.8, 0.75),
        VisualAssetKind::IslandWater => (5, Vec2::new(-0.18, 0.24), 0.04, 1.35, -0.45),
        VisualAssetKind::RouteMarker => (3, Vec2::new(-0.08, 0.2), 1.58, 1.4, 0.2),
        VisualAssetKind::WeatherLayer => (4, Vec2::new(0.18, -0.18), 8.2, 4.5, -0.75),
        VisualAssetKind::DistantImpostor => (6, Vec2::new(0.0, 0.0), 4.0, 4.3, 0.55),
        VisualAssetKind::PlayerCharacter | VisualAssetKind::Glider => return None,
    };
    let island = islands
        .get(island_index.min(islands.len().saturating_sub(1)))
        .copied()?;

    Some((
        island,
        normalized_offset,
        surface_offset_y,
        scale,
        yaw_radians,
    ))
}

pub(crate) fn mark_authored_scene_ready(
    scene_ready: On<SceneInstanceReady>,
    authored_scenes: Query<&AuthoredVisualScene>,
    mut registry: ResMut<VisualAssetRegistry>,
) {
    let Ok(scene) = authored_scenes.get(scene_ready.entity) else {
        return;
    };

    registry.mark_scene_ready(scene.kind);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn link_ready_authored_animations(
    children: Query<&Children>,
    animation_player_entities: Query<Entity, With<AnimationPlayer>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    gltfs: Res<Assets<Gltf>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut registry: ResMut<VisualAssetRegistry>,
    mut commands: Commands,
) {
    let pending_links = registry.pending_animation_links();
    for pending in pending_links {
        link_ready_authored_animation(
            pending,
            &children,
            &animation_player_entities,
            &mut animation_players,
            &gltfs,
            &mut animation_graphs,
            &mut registry,
            &mut commands,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn link_ready_authored_animation(
    pending: PendingAuthoredAnimationLink,
    children: &Query<&Children>,
    animation_player_entities: &Query<Entity, With<AnimationPlayer>>,
    animation_players: &mut Query<&mut AnimationPlayer>,
    gltfs: &Assets<Gltf>,
    animation_graphs: &mut Assets<AnimationGraph>,
    registry: &mut VisualAssetRegistry,
    commands: &mut Commands,
) {
    let Some(animation_player_entity) =
        find_descendant_animation_player(pending.scene_entity, children, animation_player_entities)
    else {
        return;
    };

    let Some(gltf) = gltfs.get(&pending.gltf_handle) else {
        return;
    };
    let clip_resolution = resolve_named_animation_clips(pending.spec.animation_clip_names, gltf);
    registry.mark_animation_player_linked(
        pending.kind,
        animation_player_entity,
        clip_resolution.ready_clip_count(),
    );

    if !clip_resolution.is_complete() {
        return;
    }

    let Ok(mut animation_player) = animation_players.get_mut(animation_player_entity) else {
        return;
    };
    let (animation_graph, animation_nodes) = AnimationGraph::from_clips(clip_resolution.clips);
    let graph_handle = animation_graphs.add(animation_graph);
    let player_animation = if pending.kind == VisualAssetKind::PlayerCharacter {
        let Ok(nodes) = <[AnimationNodeIndex; 6]>::try_from(animation_nodes.as_slice()) else {
            return;
        };
        Some(AuthoredPlayerAnimation::new(
            nodes,
            AuthoredPlayerClip::Idle,
        ))
    } else {
        None
    };
    let mut transitions = AnimationTransitions::default();
    if let Some(idle_node) = animation_nodes.first().copied() {
        transitions
            .play(&mut animation_player, idle_node, Duration::ZERO)
            .repeat();
    }

    commands
        .entity(animation_player_entity)
        .insert((AnimationGraphHandle(graph_handle), transitions));
    if let Some(player_animation) = player_animation {
        commands
            .entity(animation_player_entity)
            .insert(player_animation);
    }
    registry.mark_animation_graph_ready(
        pending.kind,
        animation_player_entity,
        animation_nodes.len(),
    );
}

pub(crate) fn find_descendant_animation_player(
    scene_entity: Entity,
    children: &Query<&Children>,
    animation_player_entities: &Query<Entity, With<AnimationPlayer>>,
) -> Option<Entity> {
    children
        .iter_descendants(scene_entity)
        .find(|entity| animation_player_entities.get(*entity).is_ok())
}

pub(crate) fn resolve_named_animation_clips(
    animation_clip_names: &'static [&'static str],
    gltf: &Gltf,
) -> NamedAnimationClipResolution {
    let mut clips = Vec::with_capacity(animation_clip_names.len());
    let mut missing_clip_names = Vec::new();
    for clip_name in animation_clip_names {
        if let Some(clip) = gltf.named_animations.get(*clip_name) {
            clips.push(clip.clone());
        } else {
            missing_clip_names.push(*clip_name);
        }
    }

    NamedAnimationClipResolution {
        clips,
        expected_clip_count: animation_clip_names.len(),
        missing_clip_names,
    }
}

#[cfg(test)]
pub(crate) fn resolve_named_animation_clip_handles(
    animation_clip_names: &'static [&'static str],
    named_animations: &HashMap<String, Handle<AnimationClip>>,
) -> NamedAnimationClipResolution {
    let mut clips = Vec::with_capacity(animation_clip_names.len());
    let mut missing_clip_names = Vec::new();
    for clip_name in animation_clip_names {
        if let Some(clip) = named_animations.get(*clip_name) {
            clips.push(clip.clone());
        } else {
            missing_clip_names.push(*clip_name);
        }
    }

    NamedAnimationClipResolution {
        clips,
        expected_clip_count: animation_clip_names.len(),
        missing_clip_names,
    }
}

pub(crate) fn authored_player_clip_for_state(
    mode: FlightMode,
    speed_mps: f32,
) -> AuthoredPlayerClip {
    match mode {
        FlightMode::Grounded if speed_mps > 0.8 => AuthoredPlayerClip::Jog,
        FlightMode::Grounded => AuthoredPlayerClip::Idle,
        FlightMode::Launching => AuthoredPlayerClip::Launch,
        FlightMode::Gliding => AuthoredPlayerClip::Glide,
        FlightMode::Airborne if speed_mps < 8.0 => AuthoredPlayerClip::Land,
        FlightMode::Airborne => AuthoredPlayerClip::AirBrake,
    }
}

pub(crate) fn update_authored_player_animation(
    player: Query<(&Velocity, &FlightController), With<Player>>,
    mut authored_players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &mut AuthoredPlayerAnimation,
    )>,
) {
    let Ok((velocity, controller)) = player.single() else {
        return;
    };
    let desired = authored_player_clip_for_state(controller.mode, velocity.0.length());

    for (mut animation_player, mut transitions, mut authored_animation) in &mut authored_players {
        let desired_node = authored_animation.node(desired);
        if authored_animation.current == desired
            && animation_player.is_playing_animation(desired_node)
        {
            continue;
        }

        let transition_duration = if authored_animation.current == desired {
            Duration::ZERO
        } else {
            Duration::from_millis(140)
        };
        transitions
            .play(&mut animation_player, desired_node, transition_duration)
            .repeat();
        authored_animation.current = desired;
    }
}

pub(crate) fn update_visual_asset_diagnostics(
    asset_server: Res<AssetServer>,
    registry: Res<VisualAssetRegistry>,
    visible_world_fixtures: Query<(&VisibleAuthoredWorldFixture, &Visibility)>,
    mut diagnostics: ResMut<VisualAssetDiagnostics>,
) {
    let mut visible_fixture_kinds = HashSet::new();
    for (fixture, visibility) in &visible_world_fixtures {
        if *visibility == Visibility::Hidden {
            continue;
        }
        visible_fixture_kinds.insert(fixture.kind);
    }

    diagnostics.metrics = visual_asset_pipeline_metrics_with_preload_states(
        &VISUAL_ASSET_SPECS,
        |spec| {
            registry
                .slots
                .iter()
                .find(|slot| slot.spec.gltf_scene_path == spec.gltf_scene_path)
                .map_or(VisualAssetLoadState::Missing, |slot| {
                    visual_asset_load_state(&asset_server, slot)
                })
        },
        |spec, _| {
            registry
                .slots
                .iter()
                .find(|slot| slot.spec.gltf_scene_path == spec.gltf_scene_path)
                .map_or(VisualAssetPreloadState::default(), |slot| {
                    visual_asset_preload_state(&asset_server, slot)
                })
        },
        |spec| registry.scene_state_for(spec),
        |spec| registry.animation_state_for(spec),
    );
    diagnostics.visible_world_fixture_count = visible_fixture_kinds.len();
}

pub(crate) fn visual_asset_load_state(
    asset_server: &AssetServer,
    slot: &VisualAssetSlot,
) -> VisualAssetLoadState {
    let Some(scene_handle) = &slot.scene_handle else {
        return slot.load_admission.load_state();
    };

    match asset_server.load_state(scene_handle) {
        LoadState::NotLoaded => VisualAssetLoadState::Queued,
        LoadState::Loading => VisualAssetLoadState::Loading,
        LoadState::Loaded => VisualAssetLoadState::Loaded,
        LoadState::Failed(_) => VisualAssetLoadState::Failed,
    }
}

pub(crate) fn visual_asset_preload_state(
    asset_server: &AssetServer,
    slot: &VisualAssetSlot,
) -> VisualAssetPreloadState {
    let Some(scene_handle) = &slot.scene_handle else {
        return VisualAssetPreloadState::default();
    };

    VisualAssetPreloadState::from_dependencies_loaded(
        asset_server.is_loaded_with_dependencies(scene_handle),
    )
}
