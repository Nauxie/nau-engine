use bevy::animation::graph::AnimationNodeIndex;
use bevy::gltf::Gltf;
use bevy::prelude::*;
#[cfg(test)]
use std::collections::HashMap;
use std::time::Duration;

use super::types::{AuthoredVisualScene, AuthoredVisualSceneRole};
use crate::{Player, eval_runtime::EvalRun};
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PlayerPoseIntent, ScarfSegment, Side,
};
use nau_engine::asset_pipeline::VisualAssetKind;
#[cfg(test)]
use nau_engine::movement::FlightMode;
use nau_engine::movement::{FlightInput, Velocity};

use super::types::{PendingAuthoredAnimationLink, VisualAssetRegistry};

const AUTHORED_PLAYER_CLIP_COUNT: usize = 11;

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
    Walk,
    Run,
    Launch,
    Fall,
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
            Self::Walk => 1,
            Self::Run => 2,
            Self::Launch => 3,
            Self::Fall => 4,
            Self::Glide => 5,
            Self::BankLeft => 6,
            Self::BankRight => 7,
            Self::Dive => 8,
            Self::AirBrake => 9,
            Self::Land => 10,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Walk => "walk",
            Self::Run => "run",
            Self::Launch => "launch",
            Self::Fall => "fall",
            Self::Glide => "glide",
            Self::BankLeft => "bank_left",
            Self::BankRight => "bank_right",
            Self::Dive => "dive",
            Self::AirBrake => "air_brake",
            Self::Land => "land",
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub(crate) struct AuthoredAnimationDiagnostics {
    pub(crate) player_count: usize,
    pub(crate) current_clip: Option<AuthoredPlayerClip>,
    pub(crate) desired_clip: Option<AuthoredPlayerClip>,
    pub(crate) transition_from_clip: Option<AuthoredPlayerClip>,
    pub(crate) transition_to_clip: Option<AuthoredPlayerClip>,
    pub(crate) transition_active: bool,
    pub(crate) transition_elapsed_ms: u64,
    pub(crate) transition_duration_ms: u64,
    pub(crate) transition_progress: f32,
    pub(crate) transition_class_label: &'static str,
}

impl Default for AuthoredAnimationDiagnostics {
    fn default() -> Self {
        Self {
            player_count: 0,
            current_clip: None,
            desired_clip: None,
            transition_from_clip: None,
            transition_to_clip: None,
            transition_active: false,
            transition_elapsed_ms: 0,
            transition_duration_ms: 0,
            transition_progress: 0.0,
            transition_class_label: "none",
        }
    }
}

impl AuthoredAnimationDiagnostics {
    pub(crate) fn current_label(self) -> &'static str {
        self.current_clip.map_or("none", AuthoredPlayerClip::label)
    }

    pub(crate) fn desired_label(self) -> &'static str {
        self.desired_clip.map_or("none", AuthoredPlayerClip::label)
    }

    pub(crate) fn transition_from_label(self) -> &'static str {
        self.transition_from_clip
            .map_or("none", AuthoredPlayerClip::label)
    }

    pub(crate) fn transition_to_label(self) -> &'static str {
        self.transition_to_clip
            .map_or("none", AuthoredPlayerClip::label)
    }

    fn observe_player_transition(&mut self, animation: AuthoredPlayerAnimation) {
        self.current_clip = Some(animation.current);
        if !animation.transition_active() {
            return;
        }

        self.transition_active = true;
        self.transition_from_clip = animation.transition_from;
        self.transition_to_clip = animation.transition_to;
        self.transition_elapsed_ms = self
            .transition_elapsed_ms
            .max((animation.transition_elapsed_secs * 1000.0).round() as u64);
        self.transition_duration_ms = self
            .transition_duration_ms
            .max((animation.transition_duration_secs * 1000.0).round() as u64);
        self.transition_progress = self
            .transition_progress
            .max(animation.transition_progress());
        self.transition_class_label = animation.transition_class_label;
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredPlayerAnimation {
    pub(crate) nodes: [AnimationNodeIndex; AUTHORED_PLAYER_CLIP_COUNT],
    pub(crate) current: AuthoredPlayerClip,
    transition_from: Option<AuthoredPlayerClip>,
    transition_to: Option<AuthoredPlayerClip>,
    transition_elapsed_secs: f32,
    transition_duration_secs: f32,
    transition_class_label: &'static str,
}

impl AuthoredPlayerAnimation {
    pub(crate) fn new(
        nodes: [AnimationNodeIndex; AUTHORED_PLAYER_CLIP_COUNT],
        current: AuthoredPlayerClip,
    ) -> Self {
        Self {
            nodes,
            current,
            transition_from: None,
            transition_to: None,
            transition_elapsed_secs: 0.0,
            transition_duration_secs: 0.0,
            transition_class_label: "none",
        }
    }

    pub(crate) fn node(self, clip: AuthoredPlayerClip) -> AnimationNodeIndex {
        self.nodes[clip.index()]
    }

    fn advance_transition(&mut self, dt: f32) {
        if self.transition_duration_secs <= f32::EPSILON {
            self.clear_transition();
            return;
        }

        self.transition_elapsed_secs =
            (self.transition_elapsed_secs + dt.max(0.0)).min(self.transition_duration_secs);
        if !self.transition_active() {
            self.clear_transition();
        }
    }

    fn start_transition(
        &mut self,
        from: AuthoredPlayerClip,
        to: AuthoredPlayerClip,
        profile: AuthoredTransitionProfile,
    ) {
        self.current = to;
        if profile.duration.is_zero() {
            self.clear_transition();
            return;
        }

        self.transition_from = Some(from);
        self.transition_to = Some(to);
        self.transition_elapsed_secs = 0.0;
        self.transition_duration_secs = profile.duration.as_secs_f32();
        self.transition_class_label = profile.class_label;
    }

    fn transition_active(self) -> bool {
        self.transition_duration_secs > f32::EPSILON
            && self.transition_elapsed_secs < self.transition_duration_secs
    }

    fn transition_progress(self) -> f32 {
        if self.transition_duration_secs <= f32::EPSILON {
            1.0
        } else {
            (self.transition_elapsed_secs / self.transition_duration_secs).clamp(0.0, 1.0)
        }
    }

    fn clear_transition(&mut self) {
        self.transition_from = None;
        self.transition_to = None;
        self.transition_elapsed_secs = 0.0;
        self.transition_duration_secs = 0.0;
        self.transition_class_label = "none";
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AuthoredTransitionProfile {
    duration: Duration,
    class_label: &'static str,
}

impl AuthoredTransitionProfile {
    fn new(duration_ms: u64, class_label: &'static str) -> Self {
        Self {
            duration: Duration::from_millis(duration_ms),
            class_label,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredPlayerPoseNode {
    pub(crate) part: CharacterPart,
    pub(crate) smoothed_translation: Vec3,
    pub(crate) smoothed_rotation: Quat,
    pub(crate) smoothing_initialized: bool,
    pub(crate) rest_transform_initialized: bool,
    pub(crate) last_smoothed_time_secs: Option<f32>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuthoredPlayerAttachmentMarker {
    Neck,
    Shoulder(Side),
    Elbow(Side),
    Wrist(Side),
    Hip(Side),
    Knee(Side),
    Ankle(Side),
}

impl AuthoredPlayerPoseNode {
    pub(crate) fn new(part: CharacterPart) -> Self {
        Self {
            part,
            smoothed_translation: part.base_translation,
            smoothed_rotation: part.base_rotation,
            smoothing_initialized: false,
            rest_transform_initialized: false,
            last_smoothed_time_secs: None,
        }
    }

    pub(crate) fn capture_rest_transform(&mut self, transform: &Transform) {
        if self.rest_transform_initialized {
            return;
        }
        self.part.base_translation = transform.translation;
        self.part.base_rotation = transform.rotation;
        self.smoothed_translation = transform.translation;
        self.smoothed_rotation = transform.rotation;
        self.rest_transform_initialized = true;
    }
}

pub(crate) fn authored_player_pose_node_for_name(name: &str) -> Option<AuthoredPlayerPoseNode> {
    let part = match name {
        "Nau Hips" => CharacterPart::new(
            CharacterPartRole::Hips,
            Vec3::new(0.0, 0.82, 0.0),
            Quat::IDENTITY,
        ),
        "Nau Torso" => CharacterPart::new(
            CharacterPartRole::Torso,
            Vec3::new(0.0, 0.36, 0.0),
            Quat::IDENTITY,
        ),
        "Nau Head" => CharacterPart::new(
            CharacterPartRole::Head,
            Vec3::new(0.0, 0.72, -0.02),
            Quat::IDENTITY,
        ),
        "Nau Left Arm" => CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::new(-0.52, 0.54, -0.02),
            Quat::IDENTITY,
        ),
        "Nau Left Forearm" => CharacterPart::new(
            CharacterPartRole::Forearm(Side::Left),
            Vec3::new(0.0, -0.44, 0.018),
            Quat::IDENTITY,
        ),
        "Nau Left Hand" => CharacterPart::new(
            CharacterPartRole::Hand(Side::Left),
            Vec3::new(0.0, -0.49, -0.005),
            Quat::IDENTITY,
        ),
        "Nau Right Arm" => CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::new(0.52, 0.54, -0.02),
            Quat::IDENTITY,
        ),
        "Nau Right Forearm" => CharacterPart::new(
            CharacterPartRole::Forearm(Side::Right),
            Vec3::new(0.0, -0.44, 0.018),
            Quat::IDENTITY,
        ),
        "Nau Right Hand" => CharacterPart::new(
            CharacterPartRole::Hand(Side::Right),
            Vec3::new(0.0, -0.49, -0.005),
            Quat::IDENTITY,
        ),
        "Nau Left Leg" => CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::new(-0.20, -0.15, 0.02),
            Quat::IDENTITY,
        ),
        "Nau Left Lower Leg" => CharacterPart::new(
            CharacterPartRole::LowerLeg(Side::Left),
            Vec3::new(0.0, -0.34, 0.01),
            Quat::IDENTITY,
        ),
        "Nau Left Boot" => CharacterPart::new(
            CharacterPartRole::Foot(Side::Left),
            Vec3::new(0.0, -0.32, -0.012),
            Quat::IDENTITY,
        ),
        "Nau Right Leg" => CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::new(0.20, -0.15, 0.02),
            Quat::IDENTITY,
        ),
        "Nau Right Lower Leg" => CharacterPart::new(
            CharacterPartRole::LowerLeg(Side::Right),
            Vec3::new(0.0, -0.34, 0.01),
            Quat::IDENTITY,
        ),
        "Nau Right Boot" => CharacterPart::new(
            CharacterPartRole::Foot(Side::Right),
            Vec3::new(0.0, -0.32, -0.012),
            Quat::IDENTITY,
        ),
        "Nau Back Scarf Anchor Accent" => CharacterPart::new(
            CharacterPartRole::Scarf(ScarfSegment::Anchor),
            Vec3::new(0.0, 0.42, 0.25),
            Quat::IDENTITY,
        ),
        "Nau Wind Scarf Accent" => CharacterPart::new(
            CharacterPartRole::Scarf(ScarfSegment::Trail),
            Vec3::new(0.20, 0.32, 0.36),
            Quat::IDENTITY,
        ),
        _ => return None,
    };

    Some(AuthoredPlayerPoseNode::new(part))
}

fn authored_player_attachment_marker_for_name(
    name: &str,
) -> Option<AuthoredPlayerAttachmentMarker> {
    match name {
        "Nau Neck Socket" => Some(AuthoredPlayerAttachmentMarker::Neck),
        "Nau Left Shoulder Socket" => Some(AuthoredPlayerAttachmentMarker::Shoulder(Side::Left)),
        "Nau Right Shoulder Socket" => Some(AuthoredPlayerAttachmentMarker::Shoulder(Side::Right)),
        "Nau Left Elbow Socket" => Some(AuthoredPlayerAttachmentMarker::Elbow(Side::Left)),
        "Nau Right Elbow Socket" => Some(AuthoredPlayerAttachmentMarker::Elbow(Side::Right)),
        "Nau Left Wrist Socket" => Some(AuthoredPlayerAttachmentMarker::Wrist(Side::Left)),
        "Nau Right Wrist Socket" => Some(AuthoredPlayerAttachmentMarker::Wrist(Side::Right)),
        "Nau Left Hip Socket" => Some(AuthoredPlayerAttachmentMarker::Hip(Side::Left)),
        "Nau Right Hip Socket" => Some(AuthoredPlayerAttachmentMarker::Hip(Side::Right)),
        "Nau Left Knee Socket" => Some(AuthoredPlayerAttachmentMarker::Knee(Side::Left)),
        "Nau Right Knee Socket" => Some(AuthoredPlayerAttachmentMarker::Knee(Side::Right)),
        "Nau Left Ankle Socket" => Some(AuthoredPlayerAttachmentMarker::Ankle(Side::Left)),
        "Nau Right Ankle Socket" => Some(AuthoredPlayerAttachmentMarker::Ankle(Side::Right)),
        _ => None,
    }
}

pub(crate) fn tag_authored_player_pose_nodes(
    mut commands: Commands,
    children: Query<&Children>,
    authored_scenes: Query<(Entity, &AuthoredVisualScene)>,
    names: Query<&Name>,
    pose_nodes: Query<(), With<AuthoredPlayerPoseNode>>,
    attachment_markers: Query<(), With<AuthoredPlayerAttachmentMarker>>,
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
            if attachment_markers.get(descendant).is_ok() {
                continue;
            }
            if let Some(marker) = authored_player_attachment_marker_for_name(name.as_str()) {
                commands.entity(descendant).insert(marker);
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
        FlightMode::Grounded if speed_mps > 6.0 => AuthoredPlayerClip::Run,
        FlightMode::Grounded if speed_mps > 1.0 => AuthoredPlayerClip::Walk,
        FlightMode::Grounded => AuthoredPlayerClip::Idle,
        FlightMode::Launching => AuthoredPlayerClip::Launch,
        FlightMode::Gliding => AuthoredPlayerClip::Glide,
        FlightMode::Airborne => AuthoredPlayerClip::Fall,
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
    _speed_mps: f32,
    input: FlightInput,
) -> AuthoredPlayerClip {
    match intent {
        PlayerPoseIntent::GroundedIdle => AuthoredPlayerClip::Idle,
        PlayerPoseIntent::GroundedWalk => AuthoredPlayerClip::Walk,
        PlayerPoseIntent::GroundedStride | PlayerPoseIntent::GroundedRun => AuthoredPlayerClip::Run,
        PlayerPoseIntent::Launching => AuthoredPlayerClip::Launch,
        PlayerPoseIntent::Gliding => AuthoredPlayerClip::Glide,
        PlayerPoseIntent::AirTurn => authored_player_air_turn_clip(input),
        PlayerPoseIntent::Diving => AuthoredPlayerClip::Dive,
        PlayerPoseIntent::AirBrake => AuthoredPlayerClip::AirBrake,
        PlayerPoseIntent::LandingAnticipation => AuthoredPlayerClip::Land,
        PlayerPoseIntent::LandingRecovery => AuthoredPlayerClip::Land,
        PlayerPoseIntent::Falling => AuthoredPlayerClip::Fall,
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
    time: Res<Time>,
    eval: Option<Res<EvalRun>>,
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
    let dt = eval.map_or_else(|| time.delta_secs(), |run| run.scenario.fixed_dt);

    for (mut animation_player, mut transitions, mut authored_animation) in &mut authored_players {
        next_diagnostics.player_count += 1;
        authored_animation.advance_transition(dt);
        let desired_node = authored_animation.node(desired);
        if authored_animation.current == desired
            && animation_player.is_playing_animation(desired_node)
        {
            next_diagnostics.observe_player_transition(*authored_animation);
            continue;
        }

        let from_clip = authored_animation.current;
        let transition_profile = authored_player_transition_profile(from_clip, desired);
        transitions
            .play(
                &mut animation_player,
                desired_node,
                transition_profile.duration,
            )
            .repeat();
        authored_animation.start_transition(from_clip, desired, transition_profile);
        next_diagnostics.observe_player_transition(*authored_animation);
    }
    *diagnostics = next_diagnostics;
}

#[cfg(test)]
fn authored_player_transition_duration(
    current: AuthoredPlayerClip,
    desired: AuthoredPlayerClip,
) -> Duration {
    authored_player_transition_profile(current, desired).duration
}

fn authored_player_transition_profile(
    current: AuthoredPlayerClip,
    desired: AuthoredPlayerClip,
) -> AuthoredTransitionProfile {
    if current == desired {
        return AuthoredTransitionProfile::new(0, "settled");
    }

    if desired == AuthoredPlayerClip::Launch {
        return AuthoredTransitionProfile::new(40, "urgent_pose");
    }

    if desired == AuthoredPlayerClip::Land {
        return if traversal_clip(current) {
            AuthoredTransitionProfile::new(150, "landing_blend")
        } else {
            AuthoredTransitionProfile::new(120, "landing_settle")
        };
    }

    if current == AuthoredPlayerClip::Launch && desired == AuthoredPlayerClip::Fall {
        return AuthoredTransitionProfile::new(100, "launch_settle");
    }

    if desired == AuthoredPlayerClip::Dive {
        return AuthoredTransitionProfile::new(90, "urgent_air_pose");
    }

    if bank_clip(current) && bank_clip(desired) {
        return AuthoredTransitionProfile::new(70, "bank_reversal");
    }

    if bank_clip(desired) {
        return AuthoredTransitionProfile::new(80, "air_turn_snap");
    }

    if traversal_clip(current) && traversal_clip(desired) {
        return AuthoredTransitionProfile::new(190, "traversal_blend");
    }

    AuthoredTransitionProfile::new(140, "standard")
}

fn bank_clip(clip: AuthoredPlayerClip) -> bool {
    matches!(
        clip,
        AuthoredPlayerClip::BankLeft | AuthoredPlayerClip::BankRight
    )
}

fn traversal_clip(clip: AuthoredPlayerClip) -> bool {
    matches!(
        clip,
        AuthoredPlayerClip::Fall
            | AuthoredPlayerClip::Glide
            | AuthoredPlayerClip::BankLeft
            | AuthoredPlayerClip::BankRight
            | AuthoredPlayerClip::Dive
            | AuthoredPlayerClip::AirBrake
    )
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
        let scarf =
            authored_player_pose_node_for_name("Nau Wind Scarf Accent").expect("scarf node");

        assert_eq!(torso.part.role, CharacterPartRole::Torso);
        assert_eq!(torso.part.base_translation, Vec3::new(0.0, 0.36, 0.0));
        assert_eq!(left_arm.part.role, CharacterPartRole::Arm(Side::Left));
        assert_eq!(
            left_arm.part.base_translation,
            Vec3::new(-0.52, 0.54, -0.02)
        );
        assert_eq!(right_leg.part.role, CharacterPartRole::Leg(Side::Right));
        assert_eq!(
            scarf.part.role,
            CharacterPartRole::Scarf(ScarfSegment::Trail)
        );
        assert!(authored_player_pose_node_for_name("Nau Belt Buckle Plate").is_none());
    }

    #[test]
    fn authored_player_attachment_marker_names_map_core_joints() {
        assert_eq!(
            authored_player_attachment_marker_for_name("Nau Left Shoulder Socket"),
            Some(AuthoredPlayerAttachmentMarker::Shoulder(Side::Left))
        );
        assert_eq!(
            authored_player_attachment_marker_for_name("Nau Right Hip Socket"),
            Some(AuthoredPlayerAttachmentMarker::Hip(Side::Right))
        );
        assert_eq!(
            authored_player_attachment_marker_for_name("Nau Neck Socket"),
            Some(AuthoredPlayerAttachmentMarker::Neck)
        );
        assert_eq!(
            authored_player_attachment_marker_for_name("Nau Left Arm"),
            None
        );
    }

    #[test]
    fn authored_player_clip_labels_match_debug_contract() {
        assert_eq!(AuthoredPlayerClip::Idle.label(), "idle");
        assert_eq!(AuthoredPlayerClip::Walk.index(), 1);
        assert_eq!(AuthoredPlayerClip::Walk.label(), "walk");
        assert_eq!(AuthoredPlayerClip::Run.index(), 2);
        assert_eq!(AuthoredPlayerClip::Run.label(), "run");
        assert_eq!(AuthoredPlayerClip::Fall.index(), 4);
        assert_eq!(AuthoredPlayerClip::Fall.label(), "fall");
        assert_eq!(AuthoredPlayerClip::BankLeft.label(), "bank_left");
        assert_eq!(AuthoredPlayerClip::BankRight.label(), "bank_right");
        assert_eq!(AuthoredPlayerClip::AirBrake.label(), "air_brake");

        let diagnostics = AuthoredAnimationDiagnostics {
            player_count: 1,
            current_clip: Some(AuthoredPlayerClip::BankLeft),
            desired_clip: Some(AuthoredPlayerClip::BankRight),
            transition_from_clip: Some(AuthoredPlayerClip::Glide),
            transition_to_clip: Some(AuthoredPlayerClip::BankRight),
            transition_duration_ms: 70,
            transition_class_label: "bank_reversal",
            ..default()
        };

        assert_eq!(diagnostics.current_label(), "bank_left");
        assert_eq!(diagnostics.desired_label(), "bank_right");
        assert_eq!(diagnostics.transition_from_label(), "glide");
        assert_eq!(diagnostics.transition_to_label(), "bank_right");
    }

    #[test]
    fn authored_player_pose_intent_maps_falling_to_fall_clip() {
        assert_eq!(
            authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedWalk, 2.0),
            AuthoredPlayerClip::Walk
        );
        assert_eq!(
            authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedStride, 4.0),
            AuthoredPlayerClip::Run
        );
        assert_eq!(
            authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedRun, 6.0),
            AuthoredPlayerClip::Run
        );
        assert_eq!(
            authored_player_clip_for_pose_intent(PlayerPoseIntent::Falling, 2.0),
            AuthoredPlayerClip::Fall
        );
        assert_eq!(
            authored_player_clip_for_pose_intent(PlayerPoseIntent::Falling, 14.0),
            AuthoredPlayerClip::Fall
        );
        assert_eq!(
            authored_player_clip_for_pose_intent(PlayerPoseIntent::AirBrake, 14.0),
            AuthoredPlayerClip::AirBrake
        );
        assert_eq!(
            authored_player_clip_for_pose_intent(PlayerPoseIntent::LandingRecovery, 2.0),
            AuthoredPlayerClip::Land
        );
    }

    #[test]
    fn authored_player_pose_intent_maps_air_turn_input_to_directional_bank_clip() {
        assert_eq!(
            authored_player_clip_for_pose_intent_with_input(
                PlayerPoseIntent::AirTurn,
                18.0,
                FlightInput {
                    left: true,
                    ..default()
                },
            ),
            AuthoredPlayerClip::BankLeft
        );
        assert_eq!(
            authored_player_clip_for_pose_intent_with_input(
                PlayerPoseIntent::AirTurn,
                18.0,
                FlightInput {
                    right: true,
                    ..default()
                },
            ),
            AuthoredPlayerClip::BankRight
        );
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
            Duration::from_millis(90)
        );
        assert_eq!(
            authored_player_transition_duration(
                AuthoredPlayerClip::Glide,
                AuthoredPlayerClip::BankRight
            ),
            Duration::from_millis(80)
        );
    }

    #[test]
    fn authored_player_transition_profile_is_pair_aware() {
        assert_eq!(
            authored_player_transition_profile(AuthoredPlayerClip::Glide, AuthoredPlayerClip::Land),
            AuthoredTransitionProfile::new(150, "landing_blend")
        );
        assert_eq!(
            authored_player_transition_profile(AuthoredPlayerClip::Run, AuthoredPlayerClip::Land),
            AuthoredTransitionProfile::new(120, "landing_settle")
        );
        assert_eq!(
            authored_player_transition_profile(
                AuthoredPlayerClip::Launch,
                AuthoredPlayerClip::Fall
            ),
            AuthoredTransitionProfile::new(100, "launch_settle")
        );
        assert_eq!(
            authored_player_transition_profile(
                AuthoredPlayerClip::BankLeft,
                AuthoredPlayerClip::BankRight
            ),
            AuthoredTransitionProfile::new(70, "bank_reversal")
        );
        assert_eq!(
            authored_player_transition_profile(
                AuthoredPlayerClip::Glide,
                AuthoredPlayerClip::BankRight
            ),
            AuthoredTransitionProfile::new(80, "air_turn_snap")
        );
        assert_eq!(
            authored_player_transition_profile(AuthoredPlayerClip::Glide, AuthoredPlayerClip::Dive),
            AuthoredTransitionProfile::new(90, "urgent_air_pose")
        );
        assert_eq!(
            authored_player_transition_profile(AuthoredPlayerClip::Run, AuthoredPlayerClip::Walk),
            AuthoredTransitionProfile::new(140, "standard")
        );
    }
}
