use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::authored_assets::{
    AuthoredVisualScene, AuthoredVisualSceneRole, GeneratedPlayerPlaceholder, VisualAssetRegistry,
};
use crate::camera_runtime::CameraFollowFilter;
use crate::eval_runtime::{EvalMovementBasis, EvalRun};
use crate::power_up_runtime::{PowerUpCollectionState, collect_aerial_power_ups};
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, advance_phase, part_pose,
    pose_blend,
};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::environment::{LiftField, apply_lift_fields};
use nau_engine::eval::scripted_input;
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning, Velocity,
    body_forward, face_flight_direction, step_flight,
};
use nau_engine::world::{SkyRoute, TERRAIN_VISUAL_FOOTING_OFFSET_M};

const ATTACHED_PLAYER_VISUAL_OFFSET_Y: f32 = -TERRAIN_VISUAL_FOOTING_OFFSET_M;

pub(crate) fn authored_player_scene_transform() -> Transform {
    Transform::from_xyz(0.0, ATTACHED_PLAYER_VISUAL_OFFSET_Y, 0.0)
}

pub(crate) fn authored_glider_scene_transform() -> Transform {
    Transform::from_xyz(0.0, 1.35 + ATTACHED_PLAYER_VISUAL_OFFSET_Y, -0.45)
}

pub(crate) fn grounded_visual_foot_gap_m(
    player_y: f32,
    ground_floor_y: f32,
    mode: FlightMode,
) -> f32 {
    if mode != FlightMode::Grounded {
        return 0.0;
    }

    let visual_foot_y = player_y + authored_player_scene_transform().translation.y;
    let terrain_visual_y = ground_floor_y - TERRAIN_VISUAL_FOOTING_OFFSET_M;
    visual_foot_y - terrain_visual_y
}

#[derive(Component)]
pub(crate) struct Player;

#[derive(Resource, Clone, Debug, Default)]
pub(crate) struct RouteObjectiveTracker {
    pub(crate) target_island_name: Option<&'static str>,
    pub(crate) completed_count: usize,
    pub(crate) total_count: usize,
    pub(crate) current_label: &'static str,
    pub(crate) current_distance_m: f32,
    pub(crate) complete: bool,
}

#[derive(SystemParam)]
pub(crate) struct MovementWorld<'w, 's> {
    route: Res<'w, SkyRoute>,
    lift_fields: Query<'w, 's, &'static LiftField>,
    power_ups: ResMut<'w, PowerUpCollectionState>,
}

struct PlayerKinematics<'a> {
    transform: &'a mut Transform,
    velocity: &'a mut Velocity,
    controller: &'a mut FlightController,
}

struct PlayerStepContext<'a> {
    tuning: &'a FlightTuning,
    route: &'a SkyRoute,
    lift_fields: &'a [LiftField],
    power_ups: &'a mut PowerUpCollectionState,
}

pub(crate) type GeneratedPlayerPlaceholderFilter = (
    With<GeneratedPlayerPlaceholder>,
    Without<CharacterPart>,
    Without<AuthoredVisualScene>,
);
pub(crate) type GeneratedCharacterPartAnimationFilter =
    (Without<AuthoredVisualScene>, Without<Player>);

pub(crate) fn update_route_objectives(
    eval: Option<Res<EvalRun>>,
    route: Res<SkyRoute>,
    player: Query<(&Transform, &FlightController), With<Player>>,
    mut tracker: ResMut<RouteObjectiveTracker>,
) {
    let Ok((transform, controller)) = player.single() else {
        return;
    };
    let target_island_name = eval
        .as_deref()
        .and_then(|run| run.scenario.target_island_name);

    if tracker.target_island_name != target_island_name {
        *tracker = RouteObjectiveTracker {
            target_island_name,
            ..default()
        };
    }

    let objectives = route.route_objectives(target_island_name);
    tracker.total_count = objectives.len();
    tracker.completed_count = tracker.completed_count.min(objectives.len());

    while let Some(objective) = objectives.get(tracker.completed_count).copied() {
        if !objective.is_complete(&route, transform.translation, controller.mode) {
            break;
        }
        tracker.completed_count += 1;
    }

    if let Some(objective) = objectives.get(tracker.completed_count).copied() {
        tracker.current_label = objective.label;
        tracker.current_distance_m = objective.horizontal_distance(transform.translation);
        tracker.complete = false;
    } else {
        tracker.current_label = "complete";
        tracker.current_distance_m = 0.0;
        tracker.complete = !objectives.is_empty();
    }
}

pub(crate) fn fly_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    tuning: Res<FlightTuning>,
    mut world: MovementWorld,
    camera: Query<&Transform, CameraFollowFilter>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut FlightController), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut controller)) = player.single_mut() else {
        return;
    };
    let facing = movement_facing(camera.single().ok(), &transform);
    let mut kinematics = PlayerKinematics {
        transform: &mut transform,
        velocity: &mut velocity,
        controller: &mut controller,
    };
    let dt = time.delta_secs();
    let lift_fields = world.lift_fields.iter().copied().collect::<Vec<_>>();
    world.power_ups.begin_frame(dt);
    let mut context = PlayerStepContext {
        tuning: &tuning,
        route: &world.route,
        lift_fields: &lift_fields,
        power_ups: &mut world.power_ups,
    };

    step_player(
        dt,
        keyboard_flight_input(&keyboard),
        facing,
        &mut context,
        &mut kinematics,
    );
}

pub(crate) fn keyboard_flight_input(keyboard: &ButtonInput<KeyCode>) -> FlightInput {
    FlightInput {
        forward: keyboard.pressed(KeyCode::KeyW),
        backward: keyboard.pressed(KeyCode::KeyS),
        left: keyboard.pressed(KeyCode::KeyA),
        right: keyboard.pressed(KeyCode::KeyD),
        glide: keyboard.pressed(KeyCode::Space),
        dive: keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
        launch: keyboard.just_pressed(KeyCode::KeyE),
    }
}

pub(crate) fn eval_fly_player(
    run: Res<EvalRun>,
    tuning: Res<FlightTuning>,
    mut world: MovementWorld,
    camera: Query<&Transform, CameraFollowFilter>,
    mut movement_basis: ResMut<EvalMovementBasis>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut FlightController), With<Player>>,
) {
    if run.finalized {
        return;
    }

    let Ok((mut transform, mut velocity, mut controller)) = player.single_mut() else {
        return;
    };
    let facing = movement_facing(camera.single().ok(), &transform);
    *movement_basis = EvalMovementBasis {
        frame: run.frame,
        facing: Some(facing),
    };
    let mut kinematics = PlayerKinematics {
        transform: &mut transform,
        velocity: &mut velocity,
        controller: &mut controller,
    };
    let dt = run.scenario.fixed_dt;
    let lift_fields = world.lift_fields.iter().copied().collect::<Vec<_>>();
    world.power_ups.begin_frame(dt);
    let mut context = PlayerStepContext {
        tuning: &tuning,
        route: &world.route,
        lift_fields: &lift_fields,
        power_ups: &mut world.power_ups,
    };

    step_player(
        dt,
        scripted_input(run.scenario, run.frame),
        facing,
        &mut context,
        &mut kinematics,
    );
}

fn step_player(
    dt: f32,
    input: FlightInput,
    facing: Facing,
    context: &mut PlayerStepContext,
    player: &mut PlayerKinematics,
) {
    let mut tuning = *context.tuning;
    let was_grounded = context.route.is_grounded_at(player.transform.translation);
    tuning.floor_y = context
        .route
        .ground_at(player.transform.translation)
        .floor_y;
    let next = step_flight(
        FlightState::new(
            player.transform.translation,
            player.velocity.0,
            *player.controller,
        ),
        input,
        facing,
        &tuning,
        dt,
    );
    let mut next = next;
    let lift = apply_lift_fields(
        next.position,
        next.velocity,
        context.lift_fields.iter().copied(),
        dt,
        next.controller.mode != FlightMode::Grounded,
    );
    next.velocity = lift.velocity;
    collect_aerial_power_ups(&mut next, context.power_ups);
    let next = context
        .route
        .resolve_ground_contact_after_step(next, was_grounded);

    player.transform.translation = next.position;
    player.velocity.0 = next.velocity;
    *player.controller = next.controller;
    player.transform.rotation = face_flight_direction(
        player.transform.rotation,
        player.velocity.0,
        input,
        facing,
        *player.controller,
        &tuning,
        dt,
    );
}

pub(crate) fn movement_facing(camera: Option<&Transform>, player_transform: &Transform) -> Facing {
    camera.map_or_else(
        || Facing::new(*player_transform.forward(), *player_transform.right()),
        |camera_transform| Facing::new(*camera_transform.forward(), *camera_transform.right()),
    )
}

pub(crate) fn animate_character(
    time: Res<Time>,
    eval: Option<Res<EvalRun>>,
    visual_assets: Res<VisualAssetRegistry>,
    mut player: Query<
        (
            &Transform,
            &Velocity,
            &FlightController,
            &mut AnimationState,
        ),
        With<Player>,
    >,
    mut parts: Query<
        (&CharacterPart, &mut Transform, &mut Visibility),
        GeneratedCharacterPartAnimationFilter,
    >,
    mut authored_scenes: Query<(&AuthoredVisualScene, &mut Visibility), Without<CharacterPart>>,
    mut generated_placeholders: Query<&mut Visibility, GeneratedPlayerPlaceholderFilter>,
) {
    let Ok((transform, velocity, controller, mut animation)) = player.single_mut() else {
        return;
    };

    let dt = eval_dt(&time, eval.as_deref());
    animation.phase = advance_phase(animation.phase, velocity.0.length(), dt);
    let pose_velocity = character_pose_velocity(velocity.0, transform.rotation);
    let blend = pose_blend(dt);
    let authored_player_ready = visual_assets.scene_ready(VisualAssetKind::PlayerCharacter);
    let authored_glider_ready = visual_assets.scene_ready(VisualAssetKind::Glider);

    for (part, mut transform, mut visibility) in &mut parts {
        let pose = part_pose(part, controller.mode, pose_velocity, animation.phase);
        transform.translation = transform.translation.lerp(pose.translation, blend);
        transform.rotation = transform.rotation.slerp(pose.rotation, blend);

        let replaced_by_authored_scene = match part.role {
            CharacterPartRole::Wing(_) => authored_glider_ready,
            _ => authored_player_ready,
        };

        *visibility = if replaced_by_authored_scene {
            Visibility::Hidden
        } else {
            match pose.visibility {
                PartVisibility::Inherited => Visibility::Inherited,
                PartVisibility::Hidden => Visibility::Hidden,
                PartVisibility::Visible => Visibility::Visible,
            }
        };
    }

    for (scene, mut visibility) in &mut authored_scenes {
        let visible = match scene.role {
            AuthoredVisualSceneRole::PlayerRuntime => authored_player_ready,
            AuthoredVisualSceneRole::GliderRuntime => {
                authored_glider_ready && controller.mode == FlightMode::Gliding
            }
            AuthoredVisualSceneRole::WorldFixture => visual_assets.scene_ready(scene.kind),
        };
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in &mut generated_placeholders {
        *visibility = if authored_player_ready {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

fn character_pose_velocity(world_velocity: Vec3, player_rotation: Quat) -> Vec3 {
    let forward = body_forward(player_rotation);
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    Vec3::new(
        world_velocity.dot(right),
        world_velocity.y,
        -world_velocity.dot(forward),
    )
}

fn eval_dt(time: &Time, eval: Option<&EvalRun>) -> f32 {
    eval.map_or_else(|| time.delta_secs(), |run| run.scenario.fixed_dt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_pose_velocity_uses_body_local_lateral_axis() {
        let rotation = Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::X, Vec3::Y)
            .rotation;
        let pose_velocity = character_pose_velocity(Vec3::NEG_Z * 14.0, rotation);

        assert!(pose_velocity.x < -13.9);
        assert!(pose_velocity.z.abs() < 0.001);
    }

    #[test]
    fn attached_authored_visuals_share_terrain_footing_offset() {
        assert_eq!(
            authored_player_scene_transform().translation.y,
            -TERRAIN_VISUAL_FOOTING_OFFSET_M
        );
        assert_eq!(
            authored_glider_scene_transform().translation.y,
            1.35 - TERRAIN_VISUAL_FOOTING_OFFSET_M
        );
        assert_eq!(
            grounded_visual_foot_gap_m(28.0, 28.0, FlightMode::Grounded),
            0.0
        );
    }
}
