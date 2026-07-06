use crate::Player;
use crate::eval_runtime::RunMode;
use crate::world_collision_runtime::{WorldCollisionProxy, WorldCollisionProxyKind};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use nau_engine::environment::{LiftField, WindField, WindFieldKind};
use nau_engine::movement::Velocity;

#[derive(Resource, Default)]
pub(crate) struct DebugVisuals {
    pub(crate) enabled: bool,
    pub(crate) collision_proxies_enabled: bool,
}

impl DebugVisuals {
    pub(crate) fn for_run_mode(run_mode: RunMode, suppress_for_screenshot: bool) -> Self {
        Self {
            enabled: run_mode.debug_visuals_enabled() && !suppress_for_screenshot,
            collision_proxies_enabled: false,
        }
    }

    pub(crate) fn toggle_collision_proxies(&mut self) {
        self.collision_proxies_enabled = !self.collision_proxies_enabled;
    }
}

pub(crate) fn toggle_debug_visuals(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visuals: ResMut<DebugVisuals>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        visuals.enabled = !visuals.enabled;
    }
    if keyboard.just_pressed(KeyCode::F2) {
        visuals.toggle_collision_proxies();
    }
}

#[derive(SystemParam)]
pub(crate) struct DebugGizmoScene<'w, 's> {
    player: Query<'w, 's, (&'static Transform, &'static Velocity), With<Player>>,
    camera: Query<'w, 's, &'static Transform, (With<Camera3d>, Without<Player>)>,
    wind_fields: Query<'w, 's, &'static WindField>,
    lift_fields: Query<'w, 's, &'static LiftField>,
    collision_proxies: Query<'w, 's, &'static WorldCollisionProxy>,
}

pub(crate) fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    time: Res<Time>,
    visuals: Res<DebugVisuals>,
    scene: DebugGizmoScene,
) {
    if !visuals.enabled {
        return;
    }

    let Ok((player_transform, velocity)) = scene.player.single() else {
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

    if let Ok(camera_transform) = scene.camera.single() {
        gizmos.line(
            camera_transform.translation,
            origin,
            Color::srgb(1.0, 1.0, 1.0),
        );
    }

    let elapsed = time.elapsed_secs();
    for field in &scene.wind_fields {
        draw_wind_field(&mut gizmos, *field, elapsed);
    }
    for field in &scene.lift_fields {
        draw_lift_field(&mut gizmos, *field);
    }
    if visuals.collision_proxies_enabled {
        for proxy in &scene.collision_proxies {
            draw_collision_proxy(&mut gizmos, *proxy);
        }
    }
}

fn draw_wind_field(gizmos: &mut Gizmos, field: WindField, elapsed_secs: f32) {
    const STREAM_COUNT: usize = 16;

    let color = wind_field_color(field.kind);
    draw_wire_box(gizmos, field.center, field.half_extents, color);

    for index in 0..STREAM_COUNT {
        let start = field.stream_origin(index, STREAM_COUNT);
        let flow = field
            .flow_at(start, elapsed_secs)
            .unwrap_or_else(|| field.flow_at(field.center, elapsed_secs).unwrap());
        let stream = capped_vector(flow.vector, 0.65, 7.5);
        draw_vector(gizmos, start, stream, color);
        gizmos.line(start - stream * 0.35, start, color);
    }
}

fn draw_lift_field(gizmos: &mut Gizmos, field: LiftField) {
    const STREAM_COUNT: usize = 12;
    let color = Color::srgb(1.0, 0.82, 0.18);
    draw_wire_box(gizmos, field.center, field.half_extents, color);

    for index in 0..STREAM_COUNT {
        let t = if STREAM_COUNT <= 1 {
            0.0
        } else {
            index as f32 / (STREAM_COUNT - 1) as f32
        };
        let angle = t * std::f32::consts::TAU;
        let radius = if index % 2 == 0 { 0.35 } else { 0.72 };
        let start = field.center - Vec3::Y * field.half_extents.y
            + Vec3::new(
                angle.cos() * field.half_extents.x * radius,
                0.0,
                angle.sin() * field.half_extents.z * radius,
            );
        draw_vector(
            gizmos,
            start,
            Vec3::Y * field.lift_accel.min(field.max_upward_speed).max(2.0) * 0.32,
            color,
        );
    }
}

fn draw_collision_proxy(gizmos: &mut Gizmos, proxy: WorldCollisionProxy) {
    draw_wire_box(
        gizmos,
        proxy.center,
        proxy.half_extents,
        collision_color(proxy.kind),
    );
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

fn collision_color(kind: WorldCollisionProxyKind) -> Color {
    match kind {
        WorldCollisionProxyKind::TerrainRim => Color::srgb(1.0, 0.35, 0.10),
        WorldCollisionProxyKind::TerrainBody => Color::srgb(0.95, 0.68, 0.18),
        WorldCollisionProxyKind::Tree => Color::srgb(0.15, 0.85, 0.30),
        WorldCollisionProxyKind::Rock => Color::srgb(0.56, 0.62, 0.70),
        WorldCollisionProxyKind::Landmark => Color::srgb(0.25, 0.55, 1.0),
    }
}
