use bevy::prelude::*;

const PLAYER_START: Vec3 = Vec3::new(0.0, 24.0, 0.0);
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
        .add_systems(
            Update,
            (
                fly_player,
                animate_character.after(fly_player),
                follow_camera,
                update_debug_readout,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec3);

#[derive(Component)]
struct FlightController {
    mode: FlightMode,
    launch_cooldown_remaining: f32,
    launch_timer: f32,
}

impl Default for FlightController {
    fn default() -> Self {
        Self {
            mode: FlightMode::Airborne,
            launch_cooldown_remaining: 0.0,
            launch_timer: 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FlightMode {
    Grounded,
    Airborne,
    Gliding,
    Launching,
}

impl FlightMode {
    fn label(self) -> &'static str {
        match self {
            Self::Grounded => "grounded",
            Self::Airborne => "airborne",
            Self::Gliding => "gliding",
            Self::Launching => "launching",
        }
    }
}

#[derive(Component)]
struct FollowCamera {
    distance: f32,
    height: f32,
    smoothing: f32,
}

#[derive(Component)]
struct DebugReadout;

#[derive(Component)]
struct CharacterPart {
    role: CharacterPartRole,
    base_translation: Vec3,
    base_rotation: Quat,
}

impl CharacterPart {
    fn new(role: CharacterPartRole, base_translation: Vec3, base_rotation: Quat) -> Self {
        Self {
            role,
            base_translation,
            base_rotation,
        }
    }
}

#[derive(Clone, Copy)]
enum CharacterPartRole {
    Torso,
    Head,
    Arm(Side),
    Leg(Side),
    Wing(Side),
}

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

type CameraFollowFilter = (With<Camera3d>, Without<Player>);

#[derive(Resource)]
struct FlightTuning {
    forward_accel: f32,
    lateral_accel: f32,
    dive_accel: f32,
    gravity: f32,
    glide_gravity_scale: f32,
    glide_forward_accel: f32,
    glide_lift_from_speed: f32,
    glide_max_fall_speed: f32,
    launch_speed: f32,
    launch_forward_bonus: f32,
    launch_cooldown: f32,
    launch_duration: f32,
    drag: f32,
    max_speed: f32,
    turn_rate: f32,
    floor_y: f32,
}

impl Default for FlightTuning {
    fn default() -> Self {
        Self {
            forward_accel: 48.0,
            lateral_accel: 30.0,
            dive_accel: 32.0,
            gravity: 18.0,
            glide_gravity_scale: 0.22,
            glide_forward_accel: 18.0,
            glide_lift_from_speed: 0.08,
            glide_max_fall_speed: 7.5,
            launch_speed: 38.0,
            launch_forward_bonus: 12.0,
            launch_cooldown: 1.4,
            launch_duration: 0.35,
            drag: 0.82,
            max_speed: 58.0,
            turn_rate: 8.0,
            floor_y: 1.2,
        }
    }
}

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
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(torso_mesh.clone()),
                MeshMaterial3d(suit_material.clone()),
                Transform::from_xyz(0.0, 0.95, 0.0),
                Visibility::Visible,
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
                Visibility::Visible,
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
                    Visibility::Visible,
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
                    Visibility::Visible,
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
        Transform::from_xyz(0.0, 32.0, 18.0).looking_at(PLAYER_START, Vec3::Y),
        FollowCamera {
            distance: 12.0,
            height: 5.0,
            smoothing: 10.0,
        },
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

    let dt = time.delta_secs();
    controller.launch_cooldown_remaining = (controller.launch_cooldown_remaining - dt).max(0.0);
    controller.launch_timer = (controller.launch_timer - dt).max(0.0);

    let forward = transform.forward();
    let right = transform.right();
    let grounded = transform.translation.y <= tuning.floor_y + 0.05;
    let dive_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if keyboard.just_pressed(KeyCode::KeyE) && controller.launch_cooldown_remaining <= 0.0 {
        velocity.0.y = velocity.0.y.max(tuning.launch_speed);
        velocity.0 += *forward * tuning.launch_forward_bonus;
        controller.launch_cooldown_remaining = tuning.launch_cooldown;
        controller.launch_timer = tuning.launch_duration;
    }

    let is_gliding = keyboard.pressed(KeyCode::Space)
        && !grounded
        && !dive_pressed
        && controller.launch_timer <= 0.05;

    let mut acceleration = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        acceleration += *forward * tuning.forward_accel;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        acceleration -= *forward * tuning.forward_accel * 0.55;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        acceleration -= *right * tuning.lateral_accel;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        acceleration += *right * tuning.lateral_accel;
    }
    if dive_pressed {
        acceleration.y -= tuning.dive_accel;
    }

    if is_gliding {
        let horizontal_speed = Vec3::new(velocity.0.x, 0.0, velocity.0.z).length();
        acceleration += *forward * tuning.glide_forward_accel;
        acceleration.y += horizontal_speed * tuning.glide_lift_from_speed;
    }

    let gravity_scale = if is_gliding {
        tuning.glide_gravity_scale
    } else {
        1.0
    };

    acceleration.y -= tuning.gravity * gravity_scale;
    velocity.0 += acceleration * dt;
    velocity.0 *= tuning.drag.powf(dt);

    if is_gliding {
        velocity.0.y = velocity.0.y.max(-tuning.glide_max_fall_speed);
    }

    if velocity.0.length() > tuning.max_speed {
        velocity.0 = velocity.0.normalize() * tuning.max_speed;
    }

    transform.translation += velocity.0 * dt;

    if transform.translation.y < tuning.floor_y {
        transform.translation.y = tuning.floor_y;
        velocity.0.y = velocity.0.y.max(0.0);
    }

    controller.mode = if transform.translation.y <= tuning.floor_y + 0.05 {
        FlightMode::Grounded
    } else if controller.launch_timer > 0.0 {
        FlightMode::Launching
    } else if is_gliding {
        FlightMode::Gliding
    } else {
        FlightMode::Airborne
    };

    if velocity.0.length_squared() > 0.5 {
        let horizontal_velocity = Vec3::new(velocity.0.x, 0.0, velocity.0.z);
        if horizontal_velocity.length_squared() > 0.1 {
            let target = Transform::from_translation(transform.translation)
                .looking_to(horizontal_velocity.normalize(), Vec3::Y)
                .rotation;
            transform.rotation = transform.rotation.slerp(target, tuning.turn_rate * dt);
        }
    }
}

fn animate_character(
    time: Res<Time>,
    player: Query<(&Velocity, &FlightController), With<Player>>,
    mut parts: Query<(&CharacterPart, &mut Transform, &mut Visibility)>,
) {
    let Ok((velocity, controller)) = player.single() else {
        return;
    };

    let speed = velocity.0.length();
    let cycle = (time.elapsed_secs() * (5.0 + speed * 0.08)).sin();

    for (part, mut transform, mut visibility) in &mut parts {
        let mut translation = part.base_translation;
        let mut rotation = part.base_rotation;

        match part.role {
            CharacterPartRole::Torso => {
                let pitch = match controller.mode {
                    FlightMode::Grounded => 0.0,
                    FlightMode::Airborne => -0.16,
                    FlightMode::Gliding => -0.32,
                    FlightMode::Launching => 0.12,
                };
                translation.y += cycle.abs() * 0.025;
                rotation *= Quat::from_rotation_x(pitch);
            }
            CharacterPartRole::Head => {
                translation.y += cycle.abs() * 0.02;
                rotation *= Quat::from_rotation_x(-0.08);
            }
            CharacterPartRole::Arm(side) => {
                let sign = side.sign();
                let pose = match controller.mode {
                    FlightMode::Grounded => sign * (0.18 + cycle * 0.16),
                    FlightMode::Airborne => sign * 1.0,
                    FlightMode::Gliding => sign * 1.35,
                    FlightMode::Launching => sign * 0.42,
                };
                let sweep = match controller.mode {
                    FlightMode::Gliding => -0.65,
                    FlightMode::Launching => 0.28,
                    FlightMode::Airborne => -0.25,
                    FlightMode::Grounded => 0.0,
                };
                rotation = Quat::from_rotation_z(pose) * Quat::from_rotation_x(sweep);
            }
            CharacterPartRole::Leg(side) => {
                let sign = side.sign();
                let pose = match controller.mode {
                    FlightMode::Grounded => sign * (0.08 + cycle * 0.08),
                    FlightMode::Airborne => sign * 0.18,
                    FlightMode::Gliding => sign * 0.24,
                    FlightMode::Launching => sign * 0.04,
                };
                let trail = match controller.mode {
                    FlightMode::Gliding => 0.48,
                    FlightMode::Airborne => 0.22,
                    FlightMode::Launching => -0.16,
                    FlightMode::Grounded => 0.0,
                };
                rotation = Quat::from_rotation_z(pose) * Quat::from_rotation_x(trail);
            }
            CharacterPartRole::Wing(side) => {
                *visibility = if controller.mode == FlightMode::Gliding {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };

                let sign = side.sign();
                let bank = (velocity.0.x * 0.012).clamp(-0.18, 0.18);
                let flutter = (time.elapsed_secs() * 12.0).sin() * 0.025;
                rotation = Quat::from_rotation_z(sign * (0.16 + bank))
                    * Quat::from_rotation_x(-0.08 + flutter);
            }
        }

        transform.translation = translation;
        transform.rotation = rotation;
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

    let travel_direction = if velocity.0.length_squared() > 1.0 {
        velocity.0.normalize()
    } else {
        *player_transform.forward()
    };
    let camera_anchor = player_transform.translation - travel_direction * follow.distance;
    let desired_position = camera_anchor + Vec3::Y * follow.height;
    let blend = 1.0 - (-follow.smoothing * time.delta_secs()).exp();

    camera_transform.translation = camera_transform.translation.lerp(desired_position, blend);
    camera_transform.look_at(player_transform.translation + Vec3::Y * 1.4, Vec3::Y);
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
        "mode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\nlaunch cooldown {:>4.1}s\nWASD steer  Space glider  E launch  Shift dive",
        controller.mode.label(),
        velocity.0.length(),
        transform.translation.y,
        velocity.0.x,
        velocity.0.y,
        velocity.0.z,
        controller.launch_cooldown_remaining
    );
}
