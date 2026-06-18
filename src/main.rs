use bevy::prelude::*;
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, Side, advance_phase,
    part_pose, pose_blend,
};
use nau_engine::camera::{FollowCamera, step_camera};
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightState, FlightTuning, Velocity,
    face_horizontal_velocity,
};

const PLAYER_START: Vec3 = Vec3::new(0.0, 1.2, 0.0);
const WORLD_RADIUS: f32 = 220.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.9)))
        .insert_resource(FlightTuning::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The NAU Engine Flight Sandbox".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, fly_player)
        .add_systems(
            Update,
            (animate_character, follow_camera, update_debug_readout).after(fly_player),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DebugReadout;

type CameraFollowFilter = (With<Camera3d>, Without<Player>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let suit_material = materials.add(Color::srgb(0.18, 0.23, 0.3));
    let skin_material = materials.add(Color::srgb(0.82, 0.58, 0.42));
    let accent_material = materials.add(Color::srgb(0.96, 0.64, 0.16));
    let glider_material = materials.add(Color::srgb(0.78, 0.42, 0.18));
    let torso_mesh = meshes.add(Capsule3d::new(0.4, 1.0));
    let head_mesh = meshes.add(Sphere::new(0.3));
    let arm_mesh = meshes.add(Cuboid::new(0.2, 0.82, 0.2));
    let leg_mesh = meshes.add(Cuboid::new(0.24, 0.9, 0.24));
    let wing_mesh = meshes.add(Cuboid::new(2.15, 0.05, 0.75));

    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.55, 0.0)),
    ));

    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(WORLD_RADIUS * 2.0, WORLD_RADIUS * 2.0),
            ),
        ),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.44, 0.25))),
        Transform::default(),
    ));

    for (index, x) in (-5..=5).enumerate() {
        let height = 5.0 + (index as f32 % 4.0) * 4.0;
        let z = if index % 2 == 0 { -28.0 } else { 34.0 };

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(5.0, height, 5.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.42, 0.38, 0.31))),
            Transform::from_xyz(x as f32 * 20.0, height * 0.5, z),
        ));
    }

    commands
        .spawn((
            Transform::from_translation(PLAYER_START),
            Player,
            Velocity::default(),
            FlightController::default(),
            AnimationState::default(),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(torso_mesh.clone()),
                MeshMaterial3d(suit_material.clone()),
                Transform::from_xyz(0.0, 0.95, 0.0),
                Visibility::Inherited,
                CharacterPart::new(
                    CharacterPartRole::Torso,
                    Vec3::new(0.0, 0.95, 0.0),
                    Quat::IDENTITY,
                ),
            ));

            parent.spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(skin_material),
                Transform::from_xyz(0.0, 1.78, 0.0),
                Visibility::Inherited,
                CharacterPart::new(
                    CharacterPartRole::Head,
                    Vec3::new(0.0, 1.78, 0.0),
                    Quat::IDENTITY,
                ),
            ));

            for side in [Side::Left, Side::Right] {
                let sign = side.sign();
                let arm_translation = Vec3::new(sign * 0.58, 1.05, 0.0);
                let arm_rotation = Quat::from_rotation_z(sign * 0.18);
                let leg_translation = Vec3::new(sign * 0.22, 0.28, 0.0);
                let leg_rotation = Quat::from_rotation_z(sign * 0.08);

                parent.spawn((
                    Mesh3d(arm_mesh.clone()),
                    MeshMaterial3d(suit_material.clone()),
                    Transform {
                        translation: arm_translation,
                        rotation: arm_rotation,
                        ..default()
                    },
                    Visibility::Inherited,
                    CharacterPart::new(CharacterPartRole::Arm(side), arm_translation, arm_rotation),
                ));

                parent.spawn((
                    Mesh3d(leg_mesh.clone()),
                    MeshMaterial3d(suit_material.clone()),
                    Transform {
                        translation: leg_translation,
                        rotation: leg_rotation,
                        ..default()
                    },
                    Visibility::Inherited,
                    CharacterPart::new(CharacterPartRole::Leg(side), leg_translation, leg_rotation),
                ));

                let wing_translation = Vec3::new(sign * 1.02, 1.45, -0.46);
                let wing_rotation =
                    Quat::from_rotation_z(sign * 0.16) * Quat::from_rotation_x(-0.08);

                parent.spawn((
                    Mesh3d(wing_mesh.clone()),
                    MeshMaterial3d(glider_material.clone()),
                    Transform {
                        translation: wing_translation,
                        rotation: wing_rotation,
                        ..default()
                    },
                    Visibility::Hidden,
                    CharacterPart::new(
                        CharacterPartRole::Wing(side),
                        wing_translation,
                        wing_rotation,
                    ),
                ));
            }

            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.18, 0.18, 0.38))),
                MeshMaterial3d(accent_material),
                Transform::from_xyz(0.0, 1.15, -0.28),
            ));
        });

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7.0, -13.0).looking_at(PLAYER_START + Vec3::Y, Vec3::Y),
        FollowCamera::default(),
    ));

    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(14.0),
            ..default()
        },
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        DebugReadout,
    ));
}

fn fly_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    tuning: Res<FlightTuning>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut FlightController), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut controller)) = player.single_mut() else {
        return;
    };

    let next = nau_engine::movement::step_flight(
        FlightState::new(transform.translation, velocity.0, *controller),
        FlightInput {
            forward: keyboard.pressed(KeyCode::KeyW),
            backward: keyboard.pressed(KeyCode::KeyS),
            left: keyboard.pressed(KeyCode::KeyA),
            right: keyboard.pressed(KeyCode::KeyD),
            glide: keyboard.pressed(KeyCode::Space),
            dive: keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
            launch: keyboard.just_pressed(KeyCode::KeyE),
        },
        Facing::new(*transform.forward(), *transform.right()),
        &tuning,
        time.delta_secs(),
    );

    transform.translation = next.position;
    velocity.0 = next.velocity;
    *controller = next.controller;
    transform.rotation = face_horizontal_velocity(
        transform.rotation,
        velocity.0,
        tuning.turn_rate,
        time.delta_secs(),
    );
}

fn animate_character(
    time: Res<Time>,
    mut player: Query<(&Velocity, &FlightController, &mut AnimationState), With<Player>>,
    mut parts: Query<(&CharacterPart, &mut Transform, &mut Visibility)>,
) {
    let Ok((velocity, controller, mut animation)) = player.single_mut() else {
        return;
    };

    animation.phase = advance_phase(animation.phase, velocity.0.length(), time.delta_secs());
    let blend = pose_blend(time.delta_secs());

    for (part, mut transform, mut visibility) in &mut parts {
        let pose = part_pose(part, controller.mode, velocity.0, animation.phase);
        transform.translation = transform.translation.lerp(pose.translation, blend);
        transform.rotation = transform.rotation.slerp(pose.rotation, blend);

        *visibility = match pose.visibility {
            PartVisibility::Inherited => Visibility::Inherited,
            PartVisibility::Hidden => Visibility::Hidden,
            PartVisibility::Visible => Visibility::Visible,
        };
    }
}

fn follow_camera(
    time: Res<Time>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    mut camera: Query<(&mut Transform, &FollowCamera), CameraFollowFilter>,
) {
    let Ok((player_transform, velocity)) = player.single() else {
        return;
    };
    let Ok((mut camera_transform, follow)) = camera.single_mut() else {
        return;
    };

    let frame = step_camera(
        camera_transform.translation,
        camera_transform.rotation,
        player_transform.translation,
        *player_transform.forward(),
        velocity.0,
        follow,
        time.delta_secs(),
    );

    camera_transform.translation = frame.position;
    camera_transform.rotation = frame.rotation;
}

fn update_debug_readout(
    player: Query<(&Transform, &Velocity, &FlightController), With<Player>>,
    mut readout: Query<&mut Text, With<DebugReadout>>,
) {
    let Ok((transform, velocity, controller)) = player.single() else {
        return;
    };
    let Ok(mut text) = readout.single_mut() else {
        return;
    };

    **text = format!(
        "mode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\nlaunch cooldown {:>4.1}s\nlaunch ready {}\nWASD steer  Space glider  E launch  Shift dive",
        controller.mode.label(),
        velocity.0.length(),
        transform.translation.y,
        velocity.0.x,
        velocity.0.y,
        velocity.0.z,
        controller.launch_cooldown_remaining,
        if controller.launch_available {
            "yes"
        } else {
            "no"
        }
    );
}
