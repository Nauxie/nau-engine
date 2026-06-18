use bevy::prelude::*;

const PLAYER_START: Vec3 = Vec3::new(0.0, 24.0, 0.0);
const WORLD_RADIUS: f32 = 220.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.9)))
        .insert_resource(FlightTuning::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "nau-engine flight sandbox".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (fly_player, follow_camera, update_debug_readout))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec3);

#[derive(Component)]
struct FollowCamera {
    distance: f32,
    height: f32,
    smoothing: f32,
}

#[derive(Component)]
struct DebugReadout;

type CameraFollowFilter = (With<Camera3d>, Without<Player>);

#[derive(Resource)]
struct FlightTuning {
    forward_accel: f32,
    lateral_accel: f32,
    vertical_boost: f32,
    dive_accel: f32,
    gravity: f32,
    glide_gravity_scale: f32,
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
            vertical_boost: 24.0,
            dive_accel: 32.0,
            gravity: 18.0,
            glide_gravity_scale: 0.22,
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

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.55, 1.4))),
        MeshMaterial3d(materials.add(Color::srgb(0.95, 0.65, 0.18))),
        Transform::from_translation(PLAYER_START),
        Player,
        Velocity::default(),
    ));

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
    mut player: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let Ok((mut transform, mut velocity)) = player.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let forward = transform.forward();
    let right = transform.right();

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
    if keyboard.pressed(KeyCode::Space) {
        acceleration.y += tuning.vertical_boost;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
        acceleration.y -= tuning.dive_accel;
    }

    let is_gliding = keyboard.pressed(KeyCode::Space) && velocity.0.y <= 2.0;
    let gravity_scale = if is_gliding {
        tuning.glide_gravity_scale
    } else {
        1.0
    };

    acceleration.y -= tuning.gravity * gravity_scale;
    velocity.0 += acceleration * dt;
    velocity.0 *= tuning.drag.powf(dt);

    if velocity.0.length() > tuning.max_speed {
        velocity.0 = velocity.0.normalize() * tuning.max_speed;
    }

    transform.translation += velocity.0 * dt;

    if transform.translation.y < tuning.floor_y {
        transform.translation.y = tuning.floor_y;
        velocity.0.y = velocity.0.y.max(0.0);
    }

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
    player: Query<(&Transform, &Velocity), With<Player>>,
    mut readout: Query<&mut Text, With<DebugReadout>>,
) {
    let Ok((transform, velocity)) = player.single() else {
        return;
    };
    let Ok(mut text) = readout.single_mut() else {
        return;
    };

    **text = format!(
        "speed {:>5.1} m/s\naltitude {:>5.1} m\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\nWASD steer  Space glide/lift  Shift dive",
        velocity.0.length(),
        transform.translation.y,
        velocity.0.x,
        velocity.0.y,
        velocity.0.z
    );
}
