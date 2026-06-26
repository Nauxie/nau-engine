use crate::Player;
use crate::authored_assets::VisualAssetRegistry;
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::generated_content::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, cloud_cluster_mesh, cloud_filament_ribbon_detail_count,
    mesh_y_range, mix_color, updraft_ribbon_mesh,
};
use bevy::camera::{CameraOutputMode, ClearColorConfig, Exposure};
use bevy::light::VolumetricFog;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use nau_engine::animation::{Side, wing_airflow_strength};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::environment::{LiftRouteNode, WindField, wind_sway_motion};
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::SkyIsland;

const UPDRAFT_RIBBONS_PER_FIELD: usize = 3;
const UPDRAFT_GUIDE_RING_LEVELS: [f32; 5] = [-0.78, -0.34, 0.1, 0.54, 0.9];
const UPDRAFT_GUIDES_PER_RING: usize = 7;
const CROSSWIND_RIBBONS_PER_FIELD: usize = 4;
const CROSSWIND_GUIDES_PER_FIELD: usize = 36;

#[derive(Component)]
pub(crate) struct CinematicSun;

#[derive(Resource, Clone, Copy, Debug)]
pub(crate) struct CinematicWeather {
    cycle_seconds: f32,
    haze_floor_m: f32,
    haze_ceiling_m: f32,
}

impl CinematicWeather {
    pub(crate) fn new(haze_ceiling_m: f32) -> Self {
        Self {
            cycle_seconds: 96.0,
            haze_floor_m: 240.0,
            haze_ceiling_m,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct WeatherDrift {
    origin: Vec3,
    axis: Vec3,
    amplitude: f32,
    bob: f32,
    speed: f32,
    phase: f32,
    spin_speed: f32,
    base_rotation: Quat,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct WindVisualMotion {
    phase: f32,
    amplitude_m: f32,
    bend_radians: f32,
    gust_speed: f32,
    wind_direction: Vec3,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct WindResponsiveVisual {
    pub(crate) base_translation: Vec3,
    pub(crate) base_rotation: Quat,
    pub(crate) base_scale: Vec3,
    pub(crate) motion: WindVisualMotion,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct GliderAirflowTrail {
    pub(crate) side: Side,
    pub(crate) base_translation: Vec3,
    pub(crate) base_rotation: Quat,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct UpdraftGuide {
    field: WindField,
    center: Vec3,
    radius: f32,
    height_offset: f32,
    phase: f32,
    angular_speed: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct UpdraftRibbon {
    field: WindField,
    spin_speed: f32,
    base_translation: Vec3,
    base_rotation: Quat,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct CrosswindGuide {
    field: WindField,
    stream_index: usize,
    stream_count: usize,
    phase: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct CrosswindRibbon {
    field: WindField,
    base_translation: Vec3,
    base_rotation: Quat,
    phase: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WindGuideVisualMetrics {
    pub(crate) updraft_guide_count: usize,
    pub(crate) updraft_ribbon_count: usize,
    pub(crate) crosswind_guide_count: usize,
    pub(crate) crosswind_ribbon_count: usize,
    pub(crate) max_updraft_visual_motion_m: f32,
    pub(crate) max_crosswind_visual_motion_m: f32,
}

pub(crate) fn spawn_updraft_guide(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    column_material: Handle<StandardMaterial>,
    ribbon_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    lift: LiftRouteNode,
) {
    let field = lift.visual_field();
    let radius = lift.half_extents.x.min(lift.half_extents.z);
    let height = lift.half_extents.y * 2.0;
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(radius * 0.34, height))),
        MeshMaterial3d(column_material),
        Transform::from_translation(lift.center),
        Name::new(format!("{} atmospheric lift haze", lift.name)),
    ));

    for ribbon_index in 0..UPDRAFT_RIBBONS_PER_FIELD {
        let phase = ribbon_index as f32 / UPDRAFT_RIBBONS_PER_FIELD as f32 * std::f32::consts::TAU;
        let base_rotation = Quat::from_rotation_y(phase * 0.35);
        commands.spawn((
            Mesh3d(meshes.add(updraft_ribbon_mesh(radius, height, phase))),
            MeshMaterial3d(ribbon_material.clone()),
            Transform {
                translation: lift.center,
                rotation: base_rotation,
                ..default()
            },
            UpdraftRibbon {
                field,
                spin_speed: 0.035 + ribbon_index as f32 * 0.012,
                base_translation: lift.center,
                base_rotation,
            },
            Name::new(format!("{} spiral airflow ribbon", lift.name)),
        ));
    }

    let marker_mesh = meshes.add(Sphere::new(0.32));
    let ring_radius = radius * 0.5;

    for (level_index, level) in UPDRAFT_GUIDE_RING_LEVELS.into_iter().enumerate() {
        for marker_index in 0..UPDRAFT_GUIDES_PER_RING {
            let phase = marker_index as f32 / UPDRAFT_GUIDES_PER_RING as f32
                * std::f32::consts::TAU
                + level_index as f32 * 0.46;
            let guide = UpdraftGuide {
                field,
                center: lift.center,
                radius: ring_radius,
                height_offset: level * lift.half_extents.y,
                phase,
                angular_speed: 0.26 + level_index as f32 * 0.035,
            };
            commands.spawn((
                Mesh3d(marker_mesh.clone()),
                MeshMaterial3d(marker_material.clone()),
                Transform::from_translation(updraft_guide_position(&guide, 0.0)),
                guide,
                Name::new(format!("{} guide mote", lift.name)),
            ));
        }
    }
}

pub(crate) fn spawn_crosswind_guide(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    ribbon_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    field: WindField,
    label: &str,
) {
    let ribbon_length = (field.half_extents.x * 1.28).max(3.0);
    let ribbon_mesh = meshes.add(Cuboid::new(ribbon_length, 0.09, 0.18));
    let marker_mesh = meshes.add(Cuboid::new(0.74, 0.07, 0.14));
    let base_rotation = rotation_from_x_to_direction(field.direction);

    for ribbon_index in 0..CROSSWIND_RIBBONS_PER_FIELD {
        let phase = ribbon_index as f32 / CROSSWIND_RIBBONS_PER_FIELD as f32;
        let origin = field.stream_origin(ribbon_index, CROSSWIND_RIBBONS_PER_FIELD);
        let base_translation = origin + field.direction * (field.half_extents.x * 0.62);
        commands.spawn((
            Mesh3d(ribbon_mesh.clone()),
            MeshMaterial3d(ribbon_material.clone()),
            Transform {
                translation: base_translation,
                rotation: base_rotation * Quat::from_rotation_x(phase * std::f32::consts::TAU),
                ..default()
            },
            CrosswindRibbon {
                field,
                base_translation,
                base_rotation,
                phase,
            },
            Name::new(format!("{label} crosswind flow ribbon")),
        ));
    }

    for stream_index in 0..CROSSWIND_GUIDES_PER_FIELD {
        let phase = (stream_index as f32 * 0.381_966).fract();
        let guide = CrosswindGuide {
            field,
            stream_index,
            stream_count: CROSSWIND_GUIDES_PER_FIELD,
            phase,
        };
        let translation = crosswind_guide_position(&guide, 0.0);
        commands.spawn((
            Mesh3d(marker_mesh.clone()),
            MeshMaterial3d(marker_material.clone()),
            Transform {
                translation,
                rotation: base_rotation * Quat::from_rotation_x(phase * std::f32::consts::TAU),
                scale: crosswind_guide_scale(&guide, translation, 0.0),
            },
            guide,
            Name::new(format!("{label} crosswind guide mote")),
        ));
    }
}

pub(crate) fn spawn_weather_layers(
    commands: &mut Commands,
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    cloud_material: Handle<StandardMaterial>,
    cloud_veil_material: Handle<StandardMaterial>,
    islands: &[SkyIsland],
) {
    for (index, island) in islands.iter().enumerate() {
        let phase = index as f32 * 0.73;
        let offset = Vec3::new(
            (phase * 2.1).sin() * island.half_extents.x * 0.75,
            42.0 + (index % 4) as f32 * 7.0,
            (phase * 1.7).cos() * island.half_extents.y * 0.85,
        );
        let origin = island.center + offset;
        let axis = Vec3::new(0.96, 0.0, 0.28).normalize();
        let scale = Vec3::new(
            island.half_extents.x * 0.45 + 18.0,
            3.8 + (index % 3) as f32 * 0.55,
            island.half_extents.y * 0.26 + 8.0,
        );
        let cloud_mesh_data = cloud_cluster_mesh(2_000 + index as u32 * 37, CLOUD_BANK_LOBES);
        let cloud_depth_m = mesh_y_range(&cloud_mesh_data) * scale.y;
        content_diagnostics.record_generated_weather_cloud(
            CLOUD_BANK_LOBES,
            cloud_mesh_data.count_vertices(),
            cloud_filament_ribbon_detail_count(CLOUD_BANK_LOBES),
            cloud_depth_m,
            true,
        );
        let cloud_mesh = meshes.add(cloud_mesh_data);

        commands.spawn((
            Mesh3d(cloud_mesh),
            MeshMaterial3d(cloud_material.clone()),
            Transform {
                translation: origin,
                scale,
                rotation: Quat::from_rotation_y(phase * 0.35),
            },
            WeatherDrift {
                origin,
                axis,
                amplitude: 5.5 + (index % 5) as f32 * 1.2,
                bob: 0.8 + (index % 3) as f32 * 0.25,
                speed: 0.07 + (index % 4) as f32 * 0.012,
                phase,
                spin_speed: 0.012 + (index % 4) as f32 * 0.004,
                base_rotation: Quat::from_rotation_y(phase * 0.35),
            },
            Name::new("drifting cloud bank"),
        ));

        if index % 2 == 0 {
            let veil_anchor = island.center
                + Vec3::new(
                    (phase * 1.3).cos() * island.half_extents.x,
                    78.0 + (index % 3) as f32 * 8.0,
                    (phase * 1.9).sin() * island.half_extents.y,
                );
            for puff_index in 0..3 {
                let puff_phase = phase + puff_index as f32 * 1.17;
                let layer_offset = Vec3::new(
                    (puff_phase * 0.9).cos() * (14.0 + puff_index as f32 * 5.0),
                    (puff_index as f32 - 1.0) * 1.7,
                    (puff_phase * 1.1).sin() * (9.0 + puff_index as f32 * 3.5),
                );
                let veil_origin = veil_anchor + layer_offset;
                let veil_rotation = Quat::from_euler(EulerRot::XYZ, -0.04, puff_phase * 0.27, 0.06);
                let veil_mesh_data = cloud_cluster_mesh(
                    3_000 + index as u32 * 53 + puff_index as u32 * 11,
                    CLOUD_VEIL_LOBES,
                );
                let veil_scale = Vec3::new(
                    island.half_extents.x * 0.36 + 14.0 + puff_index as f32 * 4.0,
                    0.52 + puff_index as f32 * 0.12,
                    island.half_extents.y * 0.13 + 6.0 + puff_index as f32 * 1.8,
                );
                let veil_depth_m = mesh_y_range(&veil_mesh_data) * veil_scale.y;
                content_diagnostics.record_generated_weather_cloud(
                    CLOUD_VEIL_LOBES,
                    veil_mesh_data.count_vertices(),
                    cloud_filament_ribbon_detail_count(CLOUD_VEIL_LOBES),
                    veil_depth_m,
                    false,
                );
                let veil_mesh = meshes.add(veil_mesh_data);

                commands.spawn((
                    Mesh3d(veil_mesh),
                    MeshMaterial3d(cloud_veil_material.clone()),
                    Transform {
                        translation: veil_origin,
                        scale: veil_scale,
                        rotation: veil_rotation,
                    },
                    WeatherDrift {
                        origin: veil_origin,
                        axis: Vec3::new(0.74, 0.0, -0.18).normalize(),
                        amplitude: 5.0 + (index % 5) as f32 * 0.7 + puff_index as f32 * 0.6,
                        bob: 0.22 + puff_index as f32 * 0.05,
                        speed: 0.021 + (index % 3) as f32 * 0.005,
                        phase: puff_phase,
                        spin_speed: 0.003 + puff_index as f32 * 0.001,
                        base_rotation: veil_rotation,
                    },
                    Name::new("high cirrus puff"),
                ));
            }
        }
    }
}

pub(crate) fn wind_visual_motion(
    island_index: usize,
    phase_offset: f32,
    amplitude_m: f32,
    bend_radians: f32,
    gust_speed: f32,
) -> WindVisualMotion {
    let island_phase = island_index as f32 * 0.91;
    let wind_direction =
        Vec3::new(0.9, 0.0, -0.34 + (island_phase * 0.8).sin() * 0.22).normalize_or_zero();

    WindVisualMotion {
        phase: island_phase + phase_offset,
        amplitude_m,
        bend_radians,
        gust_speed,
        wind_direction,
    }
}

pub(crate) fn update_cinematic_weather(
    time: Res<Time>,
    weather: Res<CinematicWeather>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<CinematicSun>>,
    mut camera_fx: Query<
        (
            &mut Camera,
            &mut Exposure,
            &mut DistanceFog,
            &mut VolumetricFog,
        ),
        With<Camera3d>,
    >,
) {
    let cycle = (time.elapsed_secs() / weather.cycle_seconds * std::f32::consts::TAU).sin();
    let warm = (cycle * 0.5 + 0.5).clamp(0.0, 1.0);
    let storm = ((time.elapsed_secs() * 0.037).sin() * 0.5 + 0.5).powf(2.2) * 0.34;
    let cool_light = Color::srgb(0.78, 0.84, 1.0);
    let warm_light = Color::srgb(1.0, 0.82, 0.55);
    let sky_clear = Color::srgb(0.46, 0.66, 0.92);
    let sky_weather = Color::srgb(0.38, 0.48, 0.64);

    let sky_color = mix_color(
        mix_color(sky_weather, sky_clear, warm),
        Color::srgb(0.56, 0.70, 0.88),
        0.18,
    );
    clear_color.0 = sky_color;
    ambient.color = mix_color(
        Color::srgb(0.48, 0.56, 0.72),
        Color::srgb(0.72, 0.68, 0.60),
        warm,
    );
    ambient.brightness = 260.0 + warm * 170.0 - storm * 80.0;

    for (mut light, mut transform) in &mut sun {
        light.color = mix_color(cool_light, warm_light, warm);
        light.illuminance = 34_000.0 + warm * 24_000.0 - storm * 7_000.0;
        let elevation = -0.62 - warm * 0.34;
        let yaw = -0.62 + cycle * 0.18;
        transform.rotation = Quat::from_euler(EulerRot::XYZ, elevation, yaw, 0.0);
    }

    for (mut camera, mut exposure, mut fog, mut volumetric_fog) in &mut camera_fx {
        camera.clear_color = ClearColorConfig::Custom(sky_color);
        camera.output_mode = CameraOutputMode::Write {
            blend_state: Some(BlendState::REPLACE),
            clear_color: ClearColorConfig::Custom(sky_color),
        };
        exposure.ev100 = 12.35 + warm * 0.42 - storm * 0.2;
        fog.color = mix_color(
            Color::srgba(0.44, 0.52, 0.66, 0.58),
            Color::srgba(0.60, 0.74, 0.92, 0.42),
            warm,
        );
        fog.directional_light_color = mix_color(
            Color::srgba(0.72, 0.78, 1.0, 0.36),
            Color::srgba(1.0, 0.78, 0.46, 0.58),
            warm,
        );
        fog.directional_light_exponent = 12.0 + warm * 14.0;
        fog.falloff = FogFalloff::Linear {
            start: weather.haze_floor_m - storm * 70.0,
            end: weather.haze_ceiling_m - storm * 150.0,
        };
        volumetric_fog.ambient_color = mix_color(
            Color::srgb(0.48, 0.56, 0.70),
            Color::srgb(0.76, 0.70, 0.60),
            warm,
        );
        volumetric_fog.ambient_intensity = 0.028 + warm * 0.022 + storm * 0.012;
        volumetric_fog.jitter = 0.42;
        volumetric_fog.step_count = 56;
    }
}

pub(crate) fn update_weather_drift(
    time: Res<Time>,
    mut clouds: Query<(&WeatherDrift, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (drift, mut transform) in &mut clouds {
        let sway = (elapsed * drift.speed + drift.phase).sin();
        let bob = (elapsed * drift.speed * 0.7 + drift.phase * 1.9).cos();
        transform.translation =
            drift.origin + drift.axis * sway * drift.amplitude + Vec3::Y * bob * drift.bob;
        transform.rotation =
            drift.base_rotation * Quat::from_rotation_y(elapsed * drift.spin_speed + sway * 0.08);
    }
}

pub(crate) fn update_wind_responsive_visuals(
    time: Res<Time>,
    mut visuals: Query<(&WindResponsiveVisual, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (visual, mut transform) in &mut visuals {
        let motion = wind_sway_motion(
            elapsed,
            visual.motion.phase,
            visual.motion.amplitude_m,
            visual.motion.bend_radians,
            visual.motion.gust_speed,
            visual.motion.wind_direction,
        );
        transform.translation = visual.base_translation + motion.offset;
        transform.rotation = visual.base_rotation * motion.rotation;
        transform.scale = visual.base_scale * motion.scale;
    }
}

pub(crate) fn update_updraft_guides(
    time: Res<Time>,
    mut guides: Query<(&UpdraftGuide, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (guide, mut transform) in &mut guides {
        transform.translation = updraft_guide_position(guide, elapsed);
        transform.rotation = Quat::from_rotation_y(guide.phase + elapsed * guide.angular_speed);
    }
}

pub(crate) fn update_updraft_ribbons(
    time: Res<Time>,
    mut ribbons: Query<(&UpdraftRibbon, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (ribbon, mut transform) in &mut ribbons {
        let flow = ribbon
            .field
            .flow_at(ribbon.base_translation, elapsed)
            .unwrap_or_else(|| ribbon.field.flow_at(ribbon.field.center, elapsed).unwrap());
        let flow_pulse = 0.75 + flow.gust_strength * 0.35;
        let horizontal_drift = Vec3::new(flow.vector.x, 0.0, flow.vector.z) * 0.045;
        transform.translation = ribbon.base_translation + horizontal_drift;
        transform.rotation =
            ribbon.base_rotation * Quat::from_rotation_y(elapsed * ribbon.spin_speed * flow_pulse);
        transform.scale = Vec3::splat(1.0 + flow.variation * 0.035);
    }
}

pub(crate) fn update_crosswind_guides(
    time: Res<Time>,
    mut guides: Query<(&CrosswindGuide, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (guide, mut transform) in &mut guides {
        let translation = crosswind_guide_position(guide, elapsed);
        transform.translation = translation;
        transform.rotation = rotation_from_x_to_direction(guide.field.direction)
            * Quat::from_rotation_x(elapsed * 1.8 + guide.phase * std::f32::consts::TAU);
        transform.scale = crosswind_guide_scale(guide, translation, elapsed);
    }
}

pub(crate) fn update_crosswind_ribbons(
    time: Res<Time>,
    mut ribbons: Query<(&CrosswindRibbon, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (ribbon, mut transform) in &mut ribbons {
        let flow = ribbon
            .field
            .flow_at(ribbon.base_translation, elapsed)
            .unwrap_or_else(|| ribbon.field.flow_at(ribbon.field.center, elapsed).unwrap());
        let lateral = wind_lateral_axis(ribbon.field.direction);
        let wave = (elapsed * 0.82 + ribbon.phase * std::f32::consts::TAU).sin();
        let lift_wave = (elapsed * 1.16 + ribbon.phase * 4.1).cos();
        transform.translation = ribbon.base_translation
            + ribbon.field.direction * (wave * flow.gust_strength * 0.48)
            + lateral * (flow.variation * 0.42 + wave * 0.18)
            + Vec3::Y * (lift_wave * 0.08);
        transform.rotation = ribbon.base_rotation
            * Quat::from_rotation_x(ribbon.phase * std::f32::consts::TAU)
            * Quat::from_rotation_z(wave * 0.08);
        let length_pulse =
            (0.86 + flow.gust_strength * 0.22 + flow.variation * 0.1).clamp(0.96, 1.24);
        let width_pulse = (0.82 + flow.variation * 0.34).clamp(0.86, 1.08);
        transform.scale = Vec3::new(length_pulse, width_pulse, width_pulse);
    }
}

pub(crate) fn update_glider_airflow_trails(
    time: Res<Time>,
    visual_assets: Res<VisualAssetRegistry>,
    player: Query<(&Velocity, &FlightController), With<Player>>,
    mut trails: Query<(&GliderAirflowTrail, &mut Transform, &mut Visibility)>,
) {
    let Ok((velocity, controller)) = player.single() else {
        return;
    };

    let airflow = wing_airflow_strength(controller.mode, velocity.0);
    let visible = airflow > 0.04 && !visual_assets.scene_ready(VisualAssetKind::Glider);
    let pulse = (time.elapsed_secs() * 9.0).sin() * 0.04 * airflow;

    for (trail, mut transform, mut visibility) in &mut trails {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        let sign = trail.side.sign();
        transform.translation =
            trail.base_translation + Vec3::new(sign * airflow * 0.08, pulse, airflow * 0.55);
        transform.rotation = trail.base_rotation
            * Quat::from_rotation_y(sign * airflow * 0.14)
            * Quat::from_rotation_x(-airflow * 0.07);
        transform.scale = Vec3::new(0.24 + airflow * 0.38, 1.0, 0.12 + airflow * 2.2);
    }
}

pub(crate) fn wind_responsive_visual_metrics<'a>(
    visuals: impl Iterator<Item = (&'a WindResponsiveVisual, &'a Transform)>,
) -> (usize, f32) {
    visuals.fold((0, 0.0_f32), |(count, max_offset), (visual, transform)| {
        (
            count + 1,
            max_offset.max(transform.translation.distance(visual.base_translation)),
        )
    })
}

pub(crate) fn wind_guide_visual_metrics<'a>(
    updraft_guides: impl Iterator<Item = (&'a UpdraftGuide, &'a Transform)>,
    updraft_ribbons: impl Iterator<Item = (&'a UpdraftRibbon, &'a Transform)>,
    crosswind_guides: impl Iterator<Item = (&'a CrosswindGuide, &'a Transform)>,
    crosswind_ribbons: impl Iterator<Item = (&'a CrosswindRibbon, &'a Transform)>,
) -> WindGuideVisualMetrics {
    let mut metrics = WindGuideVisualMetrics::default();

    for (guide, transform) in updraft_guides {
        metrics.updraft_guide_count += 1;
        metrics.max_updraft_visual_motion_m = metrics.max_updraft_visual_motion_m.max(
            transform
                .translation
                .distance(updraft_guide_position(guide, 0.0)),
        );
    }
    for (ribbon, transform) in updraft_ribbons {
        metrics.updraft_ribbon_count += 1;
        metrics.max_updraft_visual_motion_m = metrics
            .max_updraft_visual_motion_m
            .max(transform.translation.distance(ribbon.base_translation));
    }
    for (guide, transform) in crosswind_guides {
        metrics.crosswind_guide_count += 1;
        metrics.max_crosswind_visual_motion_m = metrics.max_crosswind_visual_motion_m.max(
            transform
                .translation
                .distance(crosswind_guide_position(guide, 0.0)),
        );
    }
    for (ribbon, transform) in crosswind_ribbons {
        metrics.crosswind_ribbon_count += 1;
        metrics.max_crosswind_visual_motion_m = metrics
            .max_crosswind_visual_motion_m
            .max(transform.translation.distance(ribbon.base_translation));
    }

    metrics
}

fn updraft_guide_position(guide: &UpdraftGuide, elapsed: f32) -> Vec3 {
    let field = guide.field;
    let height_span = (field.half_extents.y * 1.84).max(1.0);
    let base_progress = (guide.height_offset / height_span + 0.5).clamp(0.0, 1.0);
    let rise_speed = field.visual_speed.max(1.0) / height_span * 0.42;
    let progress = (base_progress + guide.phase * 0.037 + elapsed * rise_speed).fract();
    let height_offset = (progress - 0.5) * height_span;
    let flow_probe = guide.center + Vec3::Y * height_offset;
    let flow = guide.field.flow_at(flow_probe, elapsed);
    let variation = flow.map_or(0.0, |sample| sample.variation);
    let gust = flow.map_or(1.0, |sample| sample.gust_strength);
    let angle = guide.phase
        + elapsed * guide.angular_speed * (0.75 + gust * 0.55)
        + progress * std::f32::consts::TAU * 0.65;
    let radius = guide.radius
        * (0.82 + variation * 0.32 + (elapsed * 0.9 + guide.phase).sin() * 0.05).clamp(0.72, 1.18);
    let base = guide.center
        + Vec3::new(
            angle.cos() * radius,
            height_offset + (elapsed * 1.4 + guide.phase).sin() * 0.18,
            angle.sin() * radius,
        );
    let flow_offset = flow.map_or(Vec3::ZERO, |sample| {
        Vec3::new(sample.vector.x, 0.0, sample.vector.z) * 0.075 + Vec3::Y * sample.variation * 0.32
    });

    base + flow_offset
}

fn crosswind_guide_position(guide: &CrosswindGuide, elapsed: f32) -> Vec3 {
    let field = guide.field;
    let path_length = (field.half_extents.x * 2.0).max(1.0);
    let progress =
        (guide.phase + elapsed * field.visual_speed.max(1.0) / path_length * 0.72).fract();
    let lane_origin = field.stream_origin(guide.stream_index, guide.stream_count);
    let base = lane_origin + field.direction * (progress * path_length);
    let flow = field
        .flow_at(base, elapsed)
        .unwrap_or_else(|| field.flow_at(field.center, elapsed).unwrap());
    let lateral = wind_lateral_axis(field.direction);
    let phase = guide.phase * std::f32::consts::TAU;
    base + lateral * ((elapsed * 1.34 + phase).sin() * 0.28 + flow.variation * 0.26)
        + Vec3::Y * ((elapsed * 1.7 + phase).cos() * 0.16)
}

fn crosswind_guide_scale(guide: &CrosswindGuide, position: Vec3, elapsed: f32) -> Vec3 {
    let flow = guide
        .field
        .flow_at(position, elapsed)
        .unwrap_or_else(|| guide.field.flow_at(guide.field.center, elapsed).unwrap());
    let length_pulse = (0.8 + flow.gust_strength * 0.34 + flow.variation * 0.22).clamp(0.98, 1.36);
    let width_pulse = (0.72 + flow.variation * 0.36).clamp(0.78, 1.04);

    Vec3::new(length_pulse, width_pulse, width_pulse)
}

fn rotation_from_x_to_direction(direction: Vec3) -> Quat {
    Quat::from_rotation_arc(Vec3::X, direction.normalize_or_zero())
}

fn wind_lateral_axis(direction: Vec3) -> Vec3 {
    Vec3::new(-direction.z, 0.0, direction.x).normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_updraft_guide() -> UpdraftGuide {
        UpdraftGuide {
            field: WindField::updraft(Vec3::ZERO, Vec3::new(8.0, 16.0, 8.0), 12.0),
            center: Vec3::ZERO,
            radius: 4.0,
            height_offset: -10.0,
            phase: 0.31,
            angular_speed: 0.34,
        }
    }

    #[test]
    fn updraft_guide_positions_rise_through_field() {
        let guide = test_updraft_guide();
        let start = updraft_guide_position(&guide, 0.0);
        let later = updraft_guide_position(&guide, 2.0);

        assert!(guide.field.contains(start));
        assert!(guide.field.contains(later));
        assert!(
            later.y > start.y + 3.0,
            "expected updraft mote to ride upward flow, start={start:?}, later={later:?}"
        );
        assert!(
            Vec2::new(later.x - start.x, later.z - start.z).length() > 1.0,
            "expected rising mote to swirl laterally, start={start:?}, later={later:?}"
        );
    }

    #[test]
    fn wind_guide_metrics_capture_updraft_streamline_motion() {
        let guide = test_updraft_guide();
        let transform = Transform::from_translation(updraft_guide_position(&guide, 2.0));
        let metrics = wind_guide_visual_metrics(
            [(&guide, &transform)].into_iter(),
            std::iter::empty::<(&UpdraftRibbon, &Transform)>(),
            std::iter::empty::<(&CrosswindGuide, &Transform)>(),
            std::iter::empty::<(&CrosswindRibbon, &Transform)>(),
        );

        assert_eq!(metrics.updraft_guide_count, 1);
        assert!(metrics.max_updraft_visual_motion_m > 3.0);
    }
}
