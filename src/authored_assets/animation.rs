use bevy::animation::graph::AnimationNodeIndex;
use bevy::gltf::Gltf;
use bevy::prelude::*;
#[cfg(test)]
use std::collections::HashMap;
use std::time::Duration;

use super::types::{AuthoredVisualScene, AuthoredVisualSceneRole};
use crate::Player;
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PlayerPoseIntent, Side,
};
use nau_engine::asset_pipeline::VisualAssetKind;
#[cfg(test)]
use nau_engine::movement::FlightMode;
use nau_engine::movement::{FlightInput, Velocity};

use super::types::{PendingAuthoredAnimationLink, VisualAssetRegistry};

const AUTHORED_PLAYER_CLIP_COUNT: usize = 9;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuthoredPlayerClip {
    Idle,
    Jog,
    Launch,
    Glide,
    BankLeft,
    BankRight,
    Dive,
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
            Self::BankLeft => 4,
            Self::BankRight => 5,
            Self::Dive => 6,
            Self::AirBrake => 7,
            Self::Land => 8,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Jog => "jog",
            Self::Launch => "launch",
            Self::Glide => "glide",
            Self::BankLeft => "bank_left",
            Self::BankRight => "bank_right",
            Self::Dive => "dive",
            Self::AirBrake => "air_brake",
            Self::Land => "land",
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct AuthoredAnimationDiagnostics {
    pub(crate) player_count: usize,
    pub(crate) current_clip: Option<AuthoredPlayerClip>,
    pub(crate) desired_clip: Option<AuthoredPlayerClip>,
    pub(crate) transition_duration_ms: u64,
}

impl AuthoredAnimationDiagnostics {
    pub(crate) fn current_label(self) -> &'static str {
        self.current_clip.map_or("none", AuthoredPlayerClip::label)
    }

    pub(crate) fn desired_label(self) -> &'static str {
        self.desired_clip.map_or("none", AuthoredPlayerClip::label)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredPlayerAnimation {
    pub(crate) nodes: [AnimationNodeIndex; AUTHORED_PLAYER_CLIP_COUNT],
    pub(crate) current: AuthoredPlayerClip,
}

impl AuthoredPlayerAnimation {
    pub(crate) fn new(
        nodes: [AnimationNodeIndex; AUTHORED_PLAYER_CLIP_COUNT],
        current: AuthoredPlayerClip,
    ) -> Self {
        Self { nodes, current }
    }

    pub(crate) fn node(self, clip: AuthoredPlayerClip) -> AnimationNodeIndex {
        self.nodes[clip.index()]
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredPlayerPoseNode {
    pub(crate) part: CharacterPart,
    pub(crate) smoothed_translation: Vec3,
    pub(crate) smoothed_rotation: Quat,
    pub(crate) smoothing_initialized: bool,
    pub(crate) last_smoothed_time_secs: Option<f32>,
}

impl AuthoredPlayerPoseNode {
    pub(crate) fn new(part: CharacterPart) -> Self {
        Self {
            part,
            smoothed_translation: part.base_translation,
            smoothed_rotation: part.base_rotation,
            smoothing_initialized: false,
            last_smoothed_time_secs: None,
        }
    }
}

pub(crate) fn authored_player_pose_node_for_name(name: &str) -> Option<AuthoredPlayerPoseNode> {
    let part = match name {
        "Nau Torso" => CharacterPart::new(
            CharacterPartRole::Torso,
            Vec3::new(0.0, 1.08, 0.0),
            Quat::IDENTITY,
        ),
        "Nau Head" => CharacterPart::new(
            CharacterPartRole::Head,
            Vec3::new(0.0, 1.68, 0.0),
            Quat::IDENTITY,
        ),
        "Nau Left Arm" => CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::new(-0.48, 1.18, 0.01),
            Quat::IDENTITY,
        ),
        "Nau Right Arm" => CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::new(0.48, 1.18, 0.01),
            Quat::IDENTITY,
        ),
        "Nau Left Leg" => CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::new(-0.17, 0.30, 0.01),
            Quat::IDENTITY,
        ),
        "Nau Right Leg" => CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::new(0.17, 0.30, 0.01),
            Quat::IDENTITY,
        ),
        _ => return None,
    };

    Some(AuthoredPlayerPoseNode::new(part))
}

pub(crate) fn tag_authored_player_pose_nodes(
    mut commands: Commands,
    children: Query<&Children>,
    authored_scenes: Query<(Entity, &AuthoredVisualScene)>,
    names: Query<&Name>,
    pose_nodes: Query<(), With<AuthoredPlayerPoseNode>>,
) {
    for (scene_entity, scene) in &authored_scenes {
        if scene.role != AuthoredVisualSceneRole::PlayerRuntime {
            continue;
        }

        for descendant in children.iter_descendants(scene_entity) {
            if pose_nodes.get(descendant).is_ok() {
                continue;
            }
            let Ok(name) = names.get(descendant) else {
                continue;
            };
            if let Some(node) = authored_player_pose_node_for_name(name.as_str()) {
                commands.entity(descendant).insert(node);
            }
        }
    }
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
fn link_ready_authored_animation(
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
        let Ok(nodes) = <[AnimationNodeIndex; AUTHORED_PLAYER_CLIP_COUNT]>::try_from(
            animation_nodes.as_slice(),
        ) else {
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

fn find_descendant_animation_player(
    scene_entity: Entity,
    children: &Query<&Children>,
    animation_player_entities: &Query<Entity, With<AnimationPlayer>>,
) -> Option<Entity> {
    children
        .iter_descendants(scene_entity)
        .find(|entity| animation_player_entities.get(*entity).is_ok())
}

fn resolve_named_animation_clips(
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

#[cfg(test)]
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

#[cfg(test)]
pub(crate) fn authored_player_clip_for_pose_intent(
    intent: PlayerPoseIntent,
    speed_mps: f32,
) -> AuthoredPlayerClip {
    authored_player_clip_for_pose_intent_with_input(intent, speed_mps, FlightInput::default())
}

pub(crate) fn authored_player_clip_for_pose_intent_with_input(
    intent: PlayerPoseIntent,
    speed_mps: f32,
    input: FlightInput,
) -> AuthoredPlayerClip {
    match intent {
        PlayerPoseIntent::GroundedIdle => AuthoredPlayerClip::Idle,
        PlayerPoseIntent::GroundedStride
        | PlayerPoseIntent::GroundedWalk
        | PlayerPoseIntent::GroundedRun => AuthoredPlayerClip::Jog,
        PlayerPoseIntent::Launching => AuthoredPlayerClip::Launch,
        PlayerPoseIntent::Gliding => AuthoredPlayerClip::Glide,
        PlayerPoseIntent::AirTurn => authored_player_air_turn_clip(input),
        PlayerPoseIntent::Diving => AuthoredPlayerClip::Dive,
        PlayerPoseIntent::AirBrake => AuthoredPlayerClip::AirBrake,
        PlayerPoseIntent::LandingAnticipation => AuthoredPlayerClip::Land,
        PlayerPoseIntent::LandingRecovery => AuthoredPlayerClip::Land,
        PlayerPoseIntent::Falling if speed_mps < 8.0 => AuthoredPlayerClip::Land,
        PlayerPoseIntent::Falling => AuthoredPlayerClip::AirBrake,
    }
}

fn authored_player_air_turn_clip(input: FlightInput) -> AuthoredPlayerClip {
    let lateral_axis = input.planar_axis().x;
    if lateral_axis < 0.0 {
        AuthoredPlayerClip::BankLeft
    } else if lateral_axis > 0.0 {
        AuthoredPlayerClip::BankRight
    } else {
        AuthoredPlayerClip::Glide
    }
}

pub(crate) fn update_authored_player_animation(
    player: Query<(&Velocity, &AnimationState), With<Player>>,
    mut authored_players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &mut AuthoredPlayerAnimation,
    )>,
    mut diagnostics: ResMut<AuthoredAnimationDiagnostics>,
) {
    let Ok((velocity, animation)) = player.single() else {
        *diagnostics = AuthoredAnimationDiagnostics::default();
        return;
    };
    let desired = authored_player_clip_for_pose_intent_with_input(
        animation.pose_intent,
        velocity.0.length(),
        animation.last_input,
    );
    let mut next_diagnostics = AuthoredAnimationDiagnostics {
        desired_clip: Some(desired),
        ..default()
    };

    for (mut animation_player, mut transitions, mut authored_animation) in &mut authored_players {
        next_diagnostics.player_count += 1;
        let desired_node = authored_animation.node(desired);
        if authored_animation.current == desired
            && animation_player.is_playing_animation(desired_node)
        {
            next_diagnostics.current_clip = Some(authored_animation.current);
            continue;
        }

        let transition_duration =
            authored_player_transition_duration(authored_animation.current, desired);
        next_diagnostics.transition_duration_ms = next_diagnostics
            .transition_duration_ms
            .max(transition_duration.as_millis() as u64);
        transitions
            .play(&mut animation_player, desired_node, transition_duration)
            .repeat();
        authored_animation.current = desired;
        next_diagnostics.current_clip = Some(authored_animation.current);
    }
    *diagnostics = next_diagnostics;
}

fn authored_player_transition_duration(
    current: AuthoredPlayerClip,
    desired: AuthoredPlayerClip,
) -> Duration {
    if current == desired {
        Duration::ZERO
    } else {
        Duration::from_millis(140)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_player_pose_node_names_map_core_fixture_limbs() {
        let torso = authored_player_pose_node_for_name("Nau Torso").expect("torso node");
        let left_arm = authored_player_pose_node_for_name("Nau Left Arm").expect("left arm node");
        let right_leg =
            authored_player_pose_node_for_name("Nau Right Leg").expect("right leg node");

        assert_eq!(torso.part.role, CharacterPartRole::Torso);
        assert_eq!(torso.part.base_translation, Vec3::new(0.0, 1.08, 0.0));
        assert_eq!(left_arm.part.role, CharacterPartRole::Arm(Side::Left));
        assert_eq!(left_arm.part.base_translation, Vec3::new(-0.48, 1.18, 0.01));
        assert_eq!(right_leg.part.role, CharacterPartRole::Leg(Side::Right));
        assert!(authored_player_pose_node_for_name("Nau Belt Buckle Plate").is_none());
    }

    #[test]
    fn authored_player_clip_labels_match_debug_contract() {
        assert_eq!(AuthoredPlayerClip::Idle.label(), "idle");
        assert_eq!(AuthoredPlayerClip::BankLeft.label(), "bank_left");
        assert_eq!(AuthoredPlayerClip::BankRight.label(), "bank_right");
        assert_eq!(AuthoredPlayerClip::AirBrake.label(), "air_brake");

        let diagnostics = AuthoredAnimationDiagnostics {
            player_count: 1,
            current_clip: Some(AuthoredPlayerClip::BankLeft),
            desired_clip: Some(AuthoredPlayerClip::BankRight),
            transition_duration_ms: 140,
        };

        assert_eq!(diagnostics.current_label(), "bank_left");
        assert_eq!(diagnostics.desired_label(), "bank_right");
    }

    #[test]
    fn authored_player_transition_duration_is_zero_for_same_clip() {
        assert_eq!(
            authored_player_transition_duration(
                AuthoredPlayerClip::Glide,
                AuthoredPlayerClip::Glide
            ),
            Duration::ZERO
        );
        assert_eq!(
            authored_player_transition_duration(
                AuthoredPlayerClip::Glide,
                AuthoredPlayerClip::Dive
            ),
            Duration::from_millis(140)
        );
    }
}
