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
use nau_engine::movement::Velocity;

use super::types::{PendingAuthoredAnimationLink, VisualAssetRegistry};

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
            Self::Dive => 4,
            Self::AirBrake => 5,
            Self::Land => 6,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredPlayerAnimation {
    pub(crate) nodes: [AnimationNodeIndex; 7],
    pub(crate) current: AuthoredPlayerClip,
}

impl AuthoredPlayerAnimation {
    pub(crate) fn new(nodes: [AnimationNodeIndex; 7], current: AuthoredPlayerClip) -> Self {
        Self { nodes, current }
    }

    pub(crate) fn node(self, clip: AuthoredPlayerClip) -> AnimationNodeIndex {
        self.nodes[clip.index()]
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredPlayerPoseNode {
    pub(crate) part: CharacterPart,
}

impl AuthoredPlayerPoseNode {
    pub(crate) fn new(part: CharacterPart) -> Self {
        Self { part }
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
        let Ok(nodes) = <[AnimationNodeIndex; 7]>::try_from(animation_nodes.as_slice()) else {
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

pub(crate) fn authored_player_clip_for_pose_intent(
    intent: PlayerPoseIntent,
    speed_mps: f32,
) -> AuthoredPlayerClip {
    match intent {
        PlayerPoseIntent::GroundedIdle => AuthoredPlayerClip::Idle,
        PlayerPoseIntent::GroundedStride => AuthoredPlayerClip::Jog,
        PlayerPoseIntent::Launching => AuthoredPlayerClip::Launch,
        PlayerPoseIntent::Gliding => AuthoredPlayerClip::Glide,
        PlayerPoseIntent::Diving => AuthoredPlayerClip::Dive,
        PlayerPoseIntent::AirBrake => AuthoredPlayerClip::AirBrake,
        PlayerPoseIntent::LandingAnticipation => AuthoredPlayerClip::Land,
        PlayerPoseIntent::LandingRecovery => AuthoredPlayerClip::Land,
        PlayerPoseIntent::Falling if speed_mps < 8.0 => AuthoredPlayerClip::Land,
        PlayerPoseIntent::Falling => AuthoredPlayerClip::AirBrake,
    }
}

pub(crate) fn update_authored_player_animation(
    player: Query<(&Velocity, &AnimationState), With<Player>>,
    mut authored_players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &mut AuthoredPlayerAnimation,
    )>,
) {
    let Ok((velocity, animation)) = player.single() else {
        return;
    };
    let desired = authored_player_clip_for_pose_intent(animation.pose_intent, velocity.0.length());

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
}
