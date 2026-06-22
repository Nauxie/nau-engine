use bevy::prelude::*;
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, Side, advance_phase,
    part_pose, pose_blend,
};
use nau_engine::camera::{FollowCamera, camera_distance, camera_pitch_degrees, step_camera};
use nau_engine::diagnostics::frame_ms;
use nau_engine::environment::{WindField, WindFieldKind, visible_fields_at};
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightState, FlightTuning, Velocity,
    face_horizontal_velocity, step_flight,
};

const PLAYER_START: Vec3 = Vec3::new(0.0, 1.2, 0.0);
const WORLD_RADIUS: f32 = 220.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.9)))
        .insert_resource(FlightTuning::default())
        .insert_resource(DebugVisuals::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The NAU Engine Flight Sandbox".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_debug_visuals, fly_player))
        .add_systems(Update, (animate_character, follow_camera).after(fly_player))
        .add_systems(
            Update,
            (update_debug_readout, draw_debug_gizmos).after(follow_camera),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DebugReadout;

#[derive(Resource)]
struct DebugVisuals {
    enabled: bool,
}

impl Default for DebugVisuals {
    fn default() -> Self {
        Self { enabled: true }
    }
}

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

    commands.spawn((
        WindField::crosswind(
            Vec3::new(0.0, 5.0, 20.0),
            Vec3::new(20.0, 4.0, 8.0),
            Vec3::X,
            10.0,
        ),
        Name::new("Visual wind ribbon"),
    ));
    commands.spawn((
        WindField::updraft(Vec3::new(-28.0, 14.0, 24.0), Vec3::new(9.0, 14.0, 9.0), 8.0),
        Name::new("Visual updraft column"),
    ));
    commands.spawn((
        WindField::crosswind(
            Vec3::new(34.0, 10.0, -8.0),
            Vec3::new(18.0, 8.0, 10.0),
            Vec3::new(-1.0, 0.0, 0.35),
            7.0,
        ),
        Name::new("Visual crosswind volume"),
    ));

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

fn toggle_debug_visuals(keyboard: Res<ButtonInput<KeyCode>>, mut visuals: ResMut<DebugVisuals>) {
    if keyboard.just_pressed(KeyCode::F1) {
        visuals.enabled = !visuals.enabled;
    }
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

    let next = step_flight(
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
    time: Res<Time>,
    visuals: Res<DebugVisuals>,
    player: Query<(&Transform, &Velocity, &FlightController), With<Player>>,
    camera: Query<&Transform, CameraFollowFilter>,
    wind_fields: Query<&WindField>,
    mut readout: Query<&mut Text, With<DebugReadout>>,
) {
    let Ok((transform, velocity, controller)) = player.single() else {
        return;
    };
    let Ok(mut text) = readout.single_mut() else {
        return;
    };
    let (distance, pitch) = camera
        .single()
        .map(|camera_transform| {
            (
                camera_distance(camera_transform.translation, transform.translation),
                camera_pitch_degrees(camera_transform.rotation),
            )
        })
        .unwrap_or_default();
    let visible_wind_fields = visible_fields_at(transform.translation, wind_fields.iter().copied());
    let wind_field_count = wind_fields.iter().count();

    **text = format!(
        "frame {:>4.1} ms\nmode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\ncamera pitch {:>5.1} deg\ncamera distance {:>5.1} m\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\nvisual wind fields {} / {}\nlaunch cooldown {:>4.1}s\nlaunch ready {}\ndebug visuals {} (F1)\nWASD steer  Space glider  E launch  Shift dive",
        frame_ms(time.delta_secs()),
        controller.mode.label(),
        velocity.0.length(),
        transform.translation.y,
        pitch,
        distance,
        velocity.0.x,
        velocity.0.y,
        velocity.0.z,
        visible_wind_fields,
        wind_field_count,
        controller.launch_cooldown_remaining,
        if controller.launch_available {
            "yes"
        } else {
            "no"
        },
        if visuals.enabled { "on" } else { "off" }
    );
}

fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    visuals: Res<DebugVisuals>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    camera: Query<&Transform, CameraFollowFilter>,
    wind_fields: Query<&WindField>,
) {
    if !visuals.enabled {
        return;
    }

    let Ok((player_transform, velocity)) = player.single() else {
        return;
    };

    let origin = player_transform.translation + Vec3::Y * 1.4;
    draw_vector(
        &mut gizmos,
        origin,
        capped_vector(velocity.0, 0.16, 7.0),
        Color::srgb(0.0, 0.85, 1.0),
    );
    draw_vector(
        &mut gizmos,
        origin,
        *player_transform.forward() * 3.0,
        Color::srgb(1.0, 0.68, 0.16),
    );
    draw_vector(
        &mut gizmos,
        origin,
        *player_transform.right() * 2.0,
        Color::srgb(0.55, 0.6, 0.62),
    );

    if let Ok(camera_transform) = camera.single() {
        gizmos.line(
            camera_transform.translation,
            origin,
            Color::srgb(1.0, 1.0, 1.0),
        );
    }

    for field in &wind_fields {
        draw_wind_field(&mut gizmos, *field);
    }
}

fn draw_wind_field(gizmos: &mut Gizmos, field: WindField) {
    const STREAM_COUNT: usize = 16;

    let color = wind_field_color(field.kind);
    draw_wire_box(gizmos, field.center, field.half_extents, color);

    for index in 0..STREAM_COUNT {
        let start = field.stream_origin(index, STREAM_COUNT);
        let stream = capped_vector(field.flow_vector(), 0.65, 7.5);
        draw_vector(gizmos, start, stream, color);
        gizmos.line(start - stream * 0.35, start, color);
    }
}

fn draw_vector(gizmos: &mut Gizmos, start: Vec3, vector: Vec3, color: Color) {
    if vector.length_squared() > 0.0001 {
        gizmos.arrow(start, start + vector, color);
    }
}

fn capped_vector(vector: Vec3, scale: f32, max_length: f32) -> Vec3 {
    let scaled = vector * scale;
    let max_length_squared = max_length * max_length;

    if scaled.length_squared() <= max_length_squared {
        scaled
    } else {
        scaled.normalize() * max_length
    }
}

fn draw_wire_box(gizmos: &mut Gizmos, center: Vec3, half_extents: Vec3, color: Color) {
    const EDGES: [(usize, usize); 12] = [
        (0, 1),
        (1, 3),
        (3, 2),
        (2, 0),
        (4, 5),
        (5, 7),
        (7, 6),
        (6, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    let min = center - half_extents;
    let max = center + half_extents;
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ];

    for (start, end) in EDGES {
        gizmos.line(corners[start], corners[end], color);
    }
}

fn wind_field_color(kind: WindFieldKind) -> Color {
    match kind {
        WindFieldKind::Crosswind => Color::srgb(0.0, 0.82, 1.0),
        WindFieldKind::Updraft => Color::srgb(0.25, 1.0, 0.45),
    }
}
