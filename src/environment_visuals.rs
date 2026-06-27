use crate::Player;
use crate::authored_assets::VisualAssetRegistry;
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::generated_content::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, cloud_cluster_mesh, cloud_filament_ribbon_detail_count,
    crosswind_flow_ribbon_centerline_offset, crosswind_flow_ribbon_mesh, mesh_y_range, mix_color,
    updraft_ribbon_mesh,
};
use bevy::camera::{CameraOutputMode, ClearColorConfig, Exposure};
use bevy::light::VolumetricFog;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use nau_engine::animation::{Side, wing_airflow_strength};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::environment::{
    GAMEPLAY_LIFT_ROUTE, LiftRouteNode, WindField, visual_crosswind_fields, wind_sway_motion,
};
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::SkyIsland;

const UPDRAFT_RIBBONS_PER_FIELD: usize = 6;
const UPDRAFT_GUIDE_RING_LEVELS: [f32; 7] = [-0.86, -0.56, -0.24, 0.08, 0.4, 0.72, 0.94];
const UPDRAFT_GUIDES_PER_RING: usize = 9;
const CROSSWIND_RIBBONS_PER_FIELD: usize = 7;
const CROSSWIND_GUIDES_PER_FIELD: usize = 60;
const WIND_VISUAL_COHERENCE_DT: f32 = 0.2;
const WIND_VISUAL_ALIGNMENT_MIN_DOT: f32 = 0.55;
const WIND_FIELD_METRIC_EPSILON: f32 = 0.001;

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
pub(crate) struct UpdraftColumn {
    field: WindField,
    base_translation: Vec3,
    phase: f32,
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
    phase: f32,
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
    phase: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WindGuideVisualMetrics {
    pub(crate) updraft_guide_count: usize,
    pub(crate) updraft_ribbon_count: usize,
    pub(crate) crosswind_guide_count: usize,
    pub(crate) crosswind_ribbon_count: usize,
    pub(crate) updraft_field_count: usize,
    pub(crate) updraft_fields_with_guides_count: usize,
    pub(crate) updraft_fields_with_ribbons_count: usize,
    pub(crate) updraft_fields_with_guides_and_ribbons_count: usize,
    pub(crate) updraft_flow_coherent_field_count: usize,
    pub(crate) crosswind_field_count: usize,
    pub(crate) crosswind_fields_with_guides_count: usize,
    pub(crate) crosswind_fields_with_ribbons_count: usize,
    pub(crate) crosswind_fields_with_guides_and_ribbons_count: usize,
    pub(crate) crosswind_flow_coherent_field_count: usize,
    pub(crate) max_updraft_visual_motion_m: f32,
    pub(crate) max_updraft_visual_rise_m: f32,
    pub(crate) max_updraft_visual_swirl_displacement_m: f32,
    pub(crate) max_updraft_visual_depth_span_m: f32,
    pub(crate) max_updraft_visual_scale_pulse: f32,
    pub(crate) max_crosswind_visual_motion_m: f32,
    pub(crate) max_crosswind_guide_flow_displacement_m: f32,
    pub(crate) max_crosswind_ribbon_flow_displacement_m: f32,
    pub(crate) max_crosswind_visual_lane_depth_span_m: f32,
    pub(crate) max_crosswind_visual_scale_pulse: f32,
    pub(crate) updraft_flow_coherent_visual_count: usize,
    pub(crate) crosswind_flow_coherent_visual_count: usize,
    pub(crate) max_updraft_visual_flow_alignment: f32,
    pub(crate) max_crosswind_visual_flow_alignment: f32,
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
        UpdraftColumn {
            field,
            base_translation: lift.center,
            phase: lift
                .center
                .dot(Vec3::new(0.037, 0.011, -0.029))
                .rem_euclid(std::f32::consts::TAU),
        },
        Name::new(format!("{} atmospheric lift haze", lift.name)),
    ));

    for ribbon_index in 0..UPDRAFT_RIBBONS_PER_FIELD {
        let phase = ribbon_index as f32 / UPDRAFT_RIBBONS_PER_FIELD as f32;
        let mesh_phase = phase * std::f32::consts::TAU;
        let base_rotation = Quat::from_rotation_y(mesh_phase * 0.35);
        commands.spawn((
            Mesh3d(meshes.add(updraft_ribbon_mesh(radius, height, mesh_phase))),
            MeshMaterial3d(ribbon_material.clone()),
            Transform {
                translation: lift.center,
                rotation: base_rotation,
                ..default()
            },
            UpdraftRibbon {
                field,
                spin_speed: 0.072 + ribbon_index as f32 * 0.018,
                base_translation: lift.center,
                base_rotation,
                phase,
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
            let translation = updraft_guide_position(&guide, 0.0);
            commands.spawn((
                Mesh3d(marker_mesh.clone()),
                MeshMaterial3d(marker_material.clone()),
                Transform {
                    translation,
                    scale: updraft_guide_scale(&guide, translation, 0.0),
                    ..default()
                },
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
    let marker_mesh = meshes.add(Cuboid::new(0.74, 0.07, 0.14));
    let base_rotation = rotation_from_x_to_direction(field.direction);

    for ribbon_index in 0..CROSSWIND_RIBBONS_PER_FIELD {
        let phase = ribbon_index as f32 / CROSSWIND_RIBBONS_PER_FIELD as f32;
        let ribbon_mesh = meshes.add(crosswind_flow_ribbon_mesh(
            ribbon_length,
            phase * std::f32::consts::TAU,
        ));
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

pub(crate) fn update_updraft_columns(
    time: Res<Time>,
    mut columns: Query<(&UpdraftColumn, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (column, mut transform) in &mut columns {
        let flow = column
            .field
            .flow_at(column.base_translation, elapsed)
            .unwrap_or_else(|| column.field.flow_at(column.field.center, elapsed).unwrap());
        let breath = (elapsed * 0.48 + column.phase).sin() * 0.5 + 0.5;
        let counter_breath = (elapsed * 0.37 + column.phase * 1.7).cos() * 0.5 + 0.5;
        let radial_scale = 1.0 + flow.variation * 0.18 + breath * 0.055;

        transform.translation = column.base_translation
            + Vec3::Y * ((elapsed * 0.43 + column.phase).sin() * 0.62 + flow.variation * 0.34);
        transform.rotation = Quat::from_rotation_y(elapsed * 0.085 + column.phase * 0.14);
        transform.scale = Vec3::new(
            radial_scale,
            1.0 + flow.gust_strength * 0.028,
            1.0 + flow.variation * 0.15 + counter_breath * 0.045,
        );
    }
}

pub(crate) fn update_updraft_guides(
    time: Res<Time>,
    mut guides: Query<(&UpdraftGuide, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (guide, mut transform) in &mut guides {
        let translation = updraft_guide_position(guide, elapsed);
        transform.translation = translation;
        transform.rotation = Quat::from_rotation_y(guide.phase + elapsed * guide.angular_speed);
        transform.scale = updraft_guide_scale(guide, translation, elapsed);
    }
}

pub(crate) fn update_updraft_ribbons(
    time: Res<Time>,
    mut ribbons: Query<(&UpdraftRibbon, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (ribbon, mut transform) in &mut ribbons {
        *transform = updraft_ribbon_transform(ribbon, elapsed);
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
        *transform = crosswind_ribbon_transform(ribbon, elapsed);
    }
}

fn updraft_ribbon_transform(ribbon: &UpdraftRibbon, elapsed: f32) -> Transform {
    let flow = ribbon
        .field
        .flow_at(ribbon.base_translation, elapsed)
        .unwrap_or_else(|| ribbon.field.flow_at(ribbon.field.center, elapsed).unwrap());
    let phase = ribbon.phase * std::f32::consts::TAU;
    let field_height = (ribbon.field.half_extents.y * 2.0).max(1.0);
    let vertical_ratio = (flow.vector.y.max(0.0) / ribbon.field.visual_speed.max(1.0)).min(1.4);
    let progress = (ribbon.phase
        + elapsed * ribbon.field.visual_speed.max(1.0) / field_height
            * (0.48 + vertical_ratio * 0.08))
        .fract();
    let vertical_scroll =
        (progress - 0.5) * ribbon.field.half_extents.y * (0.38 + vertical_ratio * 0.08);
    let horizontal_flow = Vec3::new(flow.vector.x, 0.0, flow.vector.z);
    let radial_axis = horizontal_or(
        horizontal_flow,
        Vec3::new(phase.cos(), 0.0, phase.sin()).normalize_or_zero(),
    );
    let breathing = (elapsed * 1.18 + phase).sin();
    let scale_wave = (elapsed * 1.64 + phase * 0.7).sin();
    let radial_breath = radial_axis * (flow.variation * 0.74 + breathing * 0.36);
    let horizontal_drift = horizontal_flow * 0.16;
    let flow_pulse = 0.78 + flow.gust_strength * 0.42;

    Transform {
        translation: ribbon.base_translation
            + horizontal_drift
            + Vec3::Y * (vertical_scroll + flow.variation * 0.42)
            + radial_breath,
        rotation: ribbon.base_rotation
            * Quat::from_rotation_y(elapsed * ribbon.spin_speed * flow_pulse)
            * Quat::from_rotation_z(breathing * flow.variation * 0.15),
        scale: Vec3::new(
            1.0 + flow.variation * 0.14 + scale_wave * 0.08,
            1.0 + flow.gust_strength * 0.09 + vertical_ratio * 0.07 + scale_wave.abs() * 0.045,
            1.0 + flow.variation * 0.1 - scale_wave * 0.055,
        ),
    }
}

fn crosswind_ribbon_transform(ribbon: &CrosswindRibbon, elapsed: f32) -> Transform {
    let flow = ribbon
        .field
        .flow_at(ribbon.base_translation, elapsed)
        .unwrap_or_else(|| ribbon.field.flow_at(ribbon.field.center, elapsed).unwrap());
    let lateral = wind_lateral_axis(ribbon.field.direction);
    let phase = ribbon.phase * std::f32::consts::TAU;
    let directional_half_extent = ribbon.field.half_extents.x.max(1.0);
    let path_length = (directional_half_extent * 2.0).max(1.0);
    let stream_variation = 0.88 + ribbon.phase * 0.24;
    let progress = (ribbon.phase
        + elapsed * flow.speed_mps.max(1.0) / path_length * 0.84 * stream_variation)
        .fract();
    let advected = (progress - 0.5) * directional_half_extent * 1.18;
    let wave = (elapsed * 0.82 + phase).sin();
    let lift_wave = (elapsed * 1.16 + ribbon.phase * 4.1).cos();
    let flow_axis = horizontal_or(flow.vector, ribbon.field.direction);
    let gust_front = ((elapsed * 1.95 + phase * 0.63).sin()).max(0.0);
    let length_pulse =
        (0.78 + flow.gust_strength * 0.42 + flow.variation * 0.22 + gust_front * 0.1)
            .clamp(1.0, 1.55);
    let width_pulse = (0.72 + flow.variation * 0.56 + wave.abs() * 0.08).clamp(0.82, 1.24);

    Transform {
        translation: ribbon.base_translation
            + flow_axis * (advected + gust_front * flow.variation * 0.76)
            + lateral * (flow.variation * 0.9 + wave * 0.42)
            + Vec3::Y * (lift_wave * 0.24 + flow.variation * 0.24),
        rotation: rotation_from_x_to_direction(flow_axis)
            * Quat::from_rotation_x(phase + elapsed * 0.6)
            * Quat::from_rotation_z(wave * 0.2),
        scale: Vec3::new(length_pulse, width_pulse, width_pulse),
    }
}

pub(crate) fn updraft_ribbon_scene_sample_positions(
    ribbon: &UpdraftRibbon,
    transform: &Transform,
) -> [Vec3; 3] {
    const STRANDS: f32 = 1.45;
    const STOPS: [f32; 3] = [0.62, 0.78, 0.90];

    let radius = ribbon.field.half_extents.x.min(ribbon.field.half_extents.z);
    let height = ribbon.field.half_extents.y * 2.0;
    let ribbon_radius = radius * 0.42;
    let phase = ribbon.phase * std::f32::consts::TAU;

    STOPS.map(|t| {
        let angle = phase + t * std::f32::consts::TAU * STRANDS;
        let y = -height * 0.5 + t * height;
        let breathing = 1.0 + 0.08 * (angle * 2.0 + phase).sin();
        let local_position =
            Vec3::new(angle.cos(), 0.0, angle.sin()) * ribbon_radius * breathing + Vec3::Y * y;
        transform_local_point(transform, local_position)
    })
}

pub(crate) fn crosswind_ribbon_scene_sample_positions(
    ribbon: &CrosswindRibbon,
    transform: &Transform,
) -> [Vec3; 3] {
    const STOPS: [f32; 3] = [0.16, 0.5, 0.84];

    let length = (ribbon.field.half_extents.x * 1.28).max(3.0);
    STOPS.map(|t| {
        let x = (t - 0.5) * length;
        let curve = crosswind_flow_ribbon_centerline_offset(
            length,
            ribbon.phase * std::f32::consts::TAU,
            t,
        );
        transform_local_point(transform, Vec3::X * x + curve)
    })
}

fn transform_local_point(transform: &Transform, local_position: Vec3) -> Vec3 {
    transform.translation + transform.rotation * (local_position * transform.scale)
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

#[derive(Clone, Copy, Debug, Default)]
struct VisualSpan {
    min: f32,
    max: f32,
    observed: bool,
}

impl VisualSpan {
    fn observe(&mut self, value: f32) {
        if self.observed {
            self.min = self.min.min(value);
            self.max = self.max.max(value);
        } else {
            self.min = value;
            self.max = value;
            self.observed = true;
        }
    }

    fn span(self) -> f32 {
        if self.observed {
            self.max - self.min
        } else {
            0.0
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct CrosswindDepthSpan {
    lateral: VisualSpan,
    vertical: VisualSpan,
}

impl CrosswindDepthSpan {
    fn observe(&mut self, field: WindField, position: Vec3) {
        let offset = position - field.center;
        self.lateral
            .observe(offset.dot(wind_lateral_axis(field.direction)));
        self.vertical.observe(offset.y);
    }

    fn span_m(self) -> f32 {
        Vec2::new(self.lateral.span(), self.vertical.span()).length()
    }
}

#[derive(Clone, Copy, Debug)]
struct WindFieldVisualCoverage {
    field: WindField,
    guide_count: usize,
    ribbon_count: usize,
    coherent_visual_count: usize,
}

impl WindFieldVisualCoverage {
    fn new(field: WindField) -> Self {
        Self {
            field,
            guide_count: 0,
            ribbon_count: 0,
            coherent_visual_count: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct WindFieldVisualCoverageCounts {
    field_count: usize,
    fields_with_guides_count: usize,
    fields_with_ribbons_count: usize,
    fields_with_guides_and_ribbons_count: usize,
    flow_coherent_field_count: usize,
}

pub(crate) fn wind_guide_visual_metrics<'a>(
    elapsed_secs: f32,
    updraft_guides: impl Iterator<Item = (&'a UpdraftGuide, &'a Transform)>,
    updraft_ribbons: impl Iterator<Item = (&'a UpdraftRibbon, &'a Transform)>,
    crosswind_guides: impl Iterator<Item = (&'a CrosswindGuide, &'a Transform)>,
    crosswind_ribbons: impl Iterator<Item = (&'a CrosswindRibbon, &'a Transform)>,
) -> WindGuideVisualMetrics {
    let expected_updraft_fields = expected_updraft_visual_fields();
    let expected_crosswind_fields = visual_crosswind_fields();
    wind_guide_visual_metrics_for_expected_fields(
        elapsed_secs,
        &expected_updraft_fields,
        &expected_crosswind_fields,
        updraft_guides,
        updraft_ribbons,
        crosswind_guides,
        crosswind_ribbons,
    )
}

fn expected_updraft_visual_fields() -> [WindField; GAMEPLAY_LIFT_ROUTE.len()] {
    GAMEPLAY_LIFT_ROUTE.map(|node| node.visual_field())
}

fn wind_guide_visual_metrics_for_expected_fields<'a>(
    elapsed_secs: f32,
    expected_updraft_fields: &[WindField],
    expected_crosswind_fields: &[WindField],
    updraft_guides: impl Iterator<Item = (&'a UpdraftGuide, &'a Transform)>,
    updraft_ribbons: impl Iterator<Item = (&'a UpdraftRibbon, &'a Transform)>,
    crosswind_guides: impl Iterator<Item = (&'a CrosswindGuide, &'a Transform)>,
    crosswind_ribbons: impl Iterator<Item = (&'a CrosswindRibbon, &'a Transform)>,
) -> WindGuideVisualMetrics {
    let mut metrics = WindGuideVisualMetrics::default();
    let mut updraft_depth = VisualSpan::default();
    let mut crosswind_depth = CrosswindDepthSpan::default();
    let mut updraft_coverage =
        wind_field_visual_coverage_for_expected_fields(expected_updraft_fields);
    let mut crosswind_coverage =
        wind_field_visual_coverage_for_expected_fields(expected_crosswind_fields);

    for (guide, transform) in updraft_guides {
        metrics.updraft_guide_count += 1;
        let field_index =
            observe_expected_wind_field_visual_coverage(&updraft_coverage, guide.field);
        if let Some(field_index) = field_index {
            updraft_coverage[field_index].guide_count += 1;
        }
        updraft_depth.observe(transform.translation.y - guide.field.center.y);
        let baseline = updraft_guide_position(guide, 0.0);
        let displacement = transform.translation - baseline;
        metrics.max_updraft_visual_motion_m = metrics
            .max_updraft_visual_motion_m
            .max(displacement.length());
        metrics.max_updraft_visual_scale_pulse = metrics.max_updraft_visual_scale_pulse.max(
            scale_delta(transform.scale, updraft_guide_scale(guide, baseline, 0.0)),
        );
        metrics.max_updraft_visual_rise_m = metrics
            .max_updraft_visual_rise_m
            .max(displacement.y.max(0.0));
        metrics.max_updraft_visual_swirl_displacement_m = metrics
            .max_updraft_visual_swirl_displacement_m
            .max(updraft_swirl_displacement(
                guide.field,
                baseline,
                displacement,
            ));
        let coherent = record_updraft_flow_coherence(
            &mut metrics,
            guide.field,
            transform.translation,
            updraft_guide_position(guide, elapsed_secs + WIND_VISUAL_COHERENCE_DT),
            elapsed_secs,
        );
        if let (Some(field_index), true) = (field_index, coherent) {
            updraft_coverage[field_index].coherent_visual_count += 1;
        }
    }
    for (ribbon, transform) in updraft_ribbons {
        let baseline = updraft_ribbon_transform(ribbon, 0.0);
        let displacement = transform.translation - baseline.translation;
        metrics.updraft_ribbon_count += 1;
        let field_index =
            observe_expected_wind_field_visual_coverage(&updraft_coverage, ribbon.field);
        if let Some(field_index) = field_index {
            updraft_coverage[field_index].ribbon_count += 1;
        }
        observe_updraft_ribbon_depth(&mut updraft_depth, ribbon, transform);
        metrics.max_updraft_visual_scale_pulse = metrics
            .max_updraft_visual_scale_pulse
            .max(scale_delta(transform.scale, baseline.scale));
        metrics.max_updraft_visual_motion_m = metrics
            .max_updraft_visual_motion_m
            .max(displacement.length());
        metrics.max_updraft_visual_rise_m = metrics
            .max_updraft_visual_rise_m
            .max(displacement.y.max(0.0));
        metrics.max_updraft_visual_swirl_displacement_m =
            metrics.max_updraft_visual_swirl_displacement_m.max(
                updraft_swirl_displacement_on_axis(updraft_ribbon_swirl_axis(ribbon), displacement),
            );
        let coherent = record_updraft_flow_coherence(
            &mut metrics,
            ribbon.field,
            transform.translation,
            updraft_ribbon_transform(ribbon, elapsed_secs + WIND_VISUAL_COHERENCE_DT).translation,
            elapsed_secs,
        );
        if let (Some(field_index), true) = (field_index, coherent) {
            updraft_coverage[field_index].coherent_visual_count += 1;
        }
    }
    for (guide, transform) in crosswind_guides {
        metrics.crosswind_guide_count += 1;
        let field_index =
            observe_expected_wind_field_visual_coverage(&crosswind_coverage, guide.field);
        if let Some(field_index) = field_index {
            crosswind_coverage[field_index].guide_count += 1;
        }
        crosswind_depth.observe(guide.field, transform.translation);
        let baseline = crosswind_guide_position(guide, 0.0);
        let displacement = transform.translation - baseline;
        metrics.max_crosswind_visual_scale_pulse = metrics.max_crosswind_visual_scale_pulse.max(
            scale_delta(transform.scale, crosswind_guide_scale(guide, baseline, 0.0)),
        );
        metrics.max_crosswind_visual_motion_m = metrics
            .max_crosswind_visual_motion_m
            .max(displacement.length());
        metrics.max_crosswind_guide_flow_displacement_m = metrics
            .max_crosswind_guide_flow_displacement_m
            .max(displacement.dot(guide.field.direction).max(0.0));
        let coherent = record_crosswind_flow_coherence(
            &mut metrics,
            guide.field,
            transform.translation,
            crosswind_guide_position(guide, elapsed_secs + WIND_VISUAL_COHERENCE_DT),
            elapsed_secs,
        );
        if let (Some(field_index), true) = (field_index, coherent) {
            crosswind_coverage[field_index].coherent_visual_count += 1;
        }
    }
    for (ribbon, transform) in crosswind_ribbons {
        let baseline = crosswind_ribbon_transform(ribbon, 0.0);
        let displacement = transform.translation - baseline.translation;
        metrics.crosswind_ribbon_count += 1;
        let field_index =
            observe_expected_wind_field_visual_coverage(&crosswind_coverage, ribbon.field);
        if let Some(field_index) = field_index {
            crosswind_coverage[field_index].ribbon_count += 1;
        }
        crosswind_depth.observe(ribbon.field, transform.translation);
        metrics.max_crosswind_visual_scale_pulse = metrics
            .max_crosswind_visual_scale_pulse
            .max(scale_delta(transform.scale, baseline.scale));
        metrics.max_crosswind_visual_motion_m = metrics
            .max_crosswind_visual_motion_m
            .max(displacement.length());
        metrics.max_crosswind_ribbon_flow_displacement_m = metrics
            .max_crosswind_ribbon_flow_displacement_m
            .max(displacement.dot(ribbon.field.direction).max(0.0));
        let coherent = record_crosswind_flow_coherence(
            &mut metrics,
            ribbon.field,
            transform.translation,
            crosswind_ribbon_transform(ribbon, elapsed_secs + WIND_VISUAL_COHERENCE_DT).translation,
            elapsed_secs,
        );
        if let (Some(field_index), true) = (field_index, coherent) {
            crosswind_coverage[field_index].coherent_visual_count += 1;
        }
    }

    let updraft_counts = wind_field_visual_coverage_counts(&updraft_coverage);
    metrics.updraft_field_count = updraft_counts.field_count;
    metrics.updraft_fields_with_guides_count = updraft_counts.fields_with_guides_count;
    metrics.updraft_fields_with_ribbons_count = updraft_counts.fields_with_ribbons_count;
    metrics.updraft_fields_with_guides_and_ribbons_count =
        updraft_counts.fields_with_guides_and_ribbons_count;
    metrics.updraft_flow_coherent_field_count = updraft_counts.flow_coherent_field_count;

    let crosswind_counts = wind_field_visual_coverage_counts(&crosswind_coverage);
    metrics.crosswind_field_count = crosswind_counts.field_count;
    metrics.crosswind_fields_with_guides_count = crosswind_counts.fields_with_guides_count;
    metrics.crosswind_fields_with_ribbons_count = crosswind_counts.fields_with_ribbons_count;
    metrics.crosswind_fields_with_guides_and_ribbons_count =
        crosswind_counts.fields_with_guides_and_ribbons_count;
    metrics.crosswind_flow_coherent_field_count = crosswind_counts.flow_coherent_field_count;

    metrics.max_updraft_visual_depth_span_m = updraft_depth.span();
    metrics.max_crosswind_visual_lane_depth_span_m = crosswind_depth.span_m();
    metrics
}

fn wind_field_visual_coverage_for_expected_fields(
    fields: &[WindField],
) -> Vec<WindFieldVisualCoverage> {
    fields
        .iter()
        .copied()
        .map(WindFieldVisualCoverage::new)
        .collect()
}

fn observe_expected_wind_field_visual_coverage(
    coverage: &[WindFieldVisualCoverage],
    field: WindField,
) -> Option<usize> {
    coverage
        .iter()
        .position(|entry| same_wind_field_for_metrics(entry.field, field))
}

fn wind_field_visual_coverage_counts(
    coverage: &[WindFieldVisualCoverage],
) -> WindFieldVisualCoverageCounts {
    WindFieldVisualCoverageCounts {
        field_count: coverage.len(),
        fields_with_guides_count: coverage
            .iter()
            .filter(|entry| entry.guide_count > 0)
            .count(),
        fields_with_ribbons_count: coverage
            .iter()
            .filter(|entry| entry.ribbon_count > 0)
            .count(),
        fields_with_guides_and_ribbons_count: coverage
            .iter()
            .filter(|entry| entry.guide_count > 0 && entry.ribbon_count > 0)
            .count(),
        flow_coherent_field_count: coverage
            .iter()
            .filter(|entry| entry.coherent_visual_count > 0)
            .count(),
    }
}

fn same_wind_field_for_metrics(a: WindField, b: WindField) -> bool {
    a.kind == b.kind
        && a.center.distance_squared(b.center) <= WIND_FIELD_METRIC_EPSILON.powi(2)
        && a.half_extents.distance_squared(b.half_extents) <= WIND_FIELD_METRIC_EPSILON.powi(2)
        && a.direction.distance_squared(b.direction) <= WIND_FIELD_METRIC_EPSILON.powi(2)
        && (a.visual_speed - b.visual_speed).abs() <= WIND_FIELD_METRIC_EPSILON
}

fn observe_updraft_ribbon_depth(
    depth: &mut VisualSpan,
    ribbon: &UpdraftRibbon,
    transform: &Transform,
) {
    let half_height = ribbon.field.half_extents.y * transform.scale.y.max(0.1);
    depth.observe(transform.translation.y - ribbon.field.center.y - half_height);
    depth.observe(transform.translation.y - ribbon.field.center.y + half_height);
}

fn scale_delta(scale: Vec3, baseline: Vec3) -> f32 {
    (scale.x - baseline.x)
        .abs()
        .max((scale.y - baseline.y).abs())
        .max((scale.z - baseline.z).abs())
}

fn record_updraft_flow_coherence(
    metrics: &mut WindGuideVisualMetrics,
    field: WindField,
    current: Vec3,
    next: Vec3,
    elapsed_secs: f32,
) -> bool {
    let Some(alignment) = visual_flow_alignment(field, current, next, elapsed_secs, true) else {
        return false;
    };

    metrics.max_updraft_visual_flow_alignment =
        metrics.max_updraft_visual_flow_alignment.max(alignment);
    if alignment >= WIND_VISUAL_ALIGNMENT_MIN_DOT {
        metrics.updraft_flow_coherent_visual_count += 1;
        return true;
    }

    false
}

fn record_crosswind_flow_coherence(
    metrics: &mut WindGuideVisualMetrics,
    field: WindField,
    current: Vec3,
    next: Vec3,
    elapsed_secs: f32,
) -> bool {
    let Some(alignment) = visual_flow_alignment(field, current, next, elapsed_secs, false) else {
        return false;
    };

    metrics.max_crosswind_visual_flow_alignment =
        metrics.max_crosswind_visual_flow_alignment.max(alignment);
    if alignment >= WIND_VISUAL_ALIGNMENT_MIN_DOT {
        metrics.crosswind_flow_coherent_visual_count += 1;
        return true;
    }

    false
}

fn visual_flow_alignment(
    field: WindField,
    current: Vec3,
    next: Vec3,
    elapsed_secs: f32,
    include_vertical: bool,
) -> Option<f32> {
    let displacement = next - current;
    let max_step = field.half_extents.max_element().max(1.0) * 0.5;
    if displacement.length_squared() <= 0.0001 || displacement.length() > max_step {
        return None;
    }

    let flow = field
        .flow_at(current, elapsed_secs)
        .or_else(|| field.flow_at(field.center, elapsed_secs))?;
    let motion = if include_vertical {
        displacement
    } else {
        Vec3::new(displacement.x, 0.0, displacement.z)
    };
    let flow_vector = if include_vertical {
        flow.vector
    } else {
        Vec3::new(flow.vector.x, 0.0, flow.vector.z)
    };

    if motion.length_squared() <= 0.0001 || flow_vector.length_squared() <= 0.0001 {
        return None;
    }

    Some(
        motion
            .normalize()
            .dot(flow_vector.normalize())
            .clamp(-1.0, 1.0),
    )
}

fn updraft_guide_position(guide: &UpdraftGuide, elapsed: f32) -> Vec3 {
    let field = guide.field;
    let height_span = (field.half_extents.y * 1.84).max(1.0);
    let base_progress = (guide.height_offset / height_span + 0.5).clamp(0.0, 1.0);
    let rise_speed = field.visual_speed.max(1.0) / height_span * 0.68;
    let progress = (base_progress + guide.phase * 0.037 + elapsed * rise_speed).fract();
    let height_offset = (progress - 0.5) * height_span;
    let flow_probe = guide.center + Vec3::Y * height_offset;
    let flow = guide.field.flow_at(flow_probe, elapsed);
    let variation = flow.map_or(0.0, |sample| sample.variation);
    let gust = flow.map_or(1.0, |sample| sample.gust_strength);
    let angle = guide.phase
        + elapsed * guide.angular_speed * (0.72 + gust * 1.02)
        + progress * std::f32::consts::TAU * 0.92;
    let radius = guide.radius
        * (0.74 + variation * 0.54 + gust * 0.08 + (elapsed * 0.9 + guide.phase).sin() * 0.11)
            .clamp(0.7, 1.36);
    let base = guide.center
        + Vec3::new(
            angle.cos() * radius,
            height_offset + (elapsed * 1.4 + guide.phase).sin() * 0.34,
            angle.sin() * radius,
        );
    let flow_offset = flow.map_or(Vec3::ZERO, |sample| {
        Vec3::new(sample.vector.x, 0.0, sample.vector.z) * 0.22
            + Vec3::Y * (sample.vector.y.max(0.0) * 0.035 + sample.variation * 0.62)
    });

    base + flow_offset
}

fn updraft_guide_scale(guide: &UpdraftGuide, position: Vec3, elapsed: f32) -> Vec3 {
    let flow = guide
        .field
        .flow_at(position, elapsed)
        .unwrap_or_else(|| guide.field.flow_at(guide.field.center, elapsed).unwrap());
    let vertical_ratio = (flow.vector.y.max(0.0) / guide.field.visual_speed.max(1.0)).min(1.4);
    let phase = guide.phase + flow.variation * 1.7;
    let pulse = (elapsed * 2.4 + phase).sin();
    let core =
        (0.72 + flow.gust_strength * 0.25 + flow.variation * 0.24 + pulse * 0.1).clamp(0.78, 1.42);
    let stretch = (0.72 + vertical_ratio * 0.32 + flow.variation * 0.2 + pulse.max(0.0) * 0.08)
        .clamp(0.82, 1.48);

    Vec3::new(core, stretch, core)
}

fn crosswind_guide_position(guide: &CrosswindGuide, elapsed: f32) -> Vec3 {
    let field = guide.field;
    let path_length = (field.half_extents.x * 2.0).max(1.0);
    let stream_variation = 0.86 + (guide.stream_index % 7) as f32 * 0.035;
    let progress = (guide.phase
        + elapsed * field.visual_speed.max(1.0) / path_length * 0.9 * stream_variation)
        .fract();
    let lane_origin = field.stream_origin(guide.stream_index, guide.stream_count);
    let base = lane_origin + field.direction * (progress * path_length);
    let flow = field
        .flow_at(base, elapsed)
        .unwrap_or_else(|| field.flow_at(field.center, elapsed).unwrap());
    let lateral = wind_lateral_axis(field.direction);
    let phase = guide.phase * std::f32::consts::TAU;
    let flow_axis = horizontal_or(flow.vector, field.direction);
    let shear = (flow_axis - field.direction) * (field.half_extents.x * 0.08);
    let gust_front = ((elapsed * 2.05 + phase).sin()).max(0.0);

    base + shear
        + flow_axis * (gust_front * flow.variation * 0.86)
        + lateral * ((elapsed * 1.34 + phase).sin() * 0.56 + flow.variation * 0.52)
        + Vec3::Y * ((elapsed * 1.7 + phase).cos() * 0.34 + flow.variation * 0.16)
}

fn crosswind_guide_scale(guide: &CrosswindGuide, position: Vec3, elapsed: f32) -> Vec3 {
    let flow = guide
        .field
        .flow_at(position, elapsed)
        .unwrap_or_else(|| guide.field.flow_at(guide.field.center, elapsed).unwrap());
    let phase = guide.phase * std::f32::consts::TAU;
    let pulse = (elapsed * 1.78 + phase + flow.variation * 1.4).sin();
    let length_pulse =
        (0.72 + flow.gust_strength * 0.48 + flow.variation * 0.34 + pulse.max(0.0) * 0.14)
            .clamp(1.04, 1.72);
    let width_pulse = (0.66 + flow.variation * 0.58 - pulse * 0.07).clamp(0.78, 1.24);

    Vec3::new(length_pulse, width_pulse, width_pulse)
}

fn updraft_swirl_displacement(field: WindField, position: Vec3, displacement: Vec3) -> f32 {
    let radial = Vec3::new(
        position.x - field.center.x,
        0.0,
        position.z - field.center.z,
    );
    if radial.length_squared() <= 0.0001 {
        return 0.0;
    }

    let tangent = Vec3::new(-radial.z, 0.0, radial.x).normalize();
    updraft_swirl_displacement_on_axis(tangent, displacement)
}

fn updraft_ribbon_swirl_axis(ribbon: &UpdraftRibbon) -> Vec3 {
    let angle = ribbon.phase * std::f32::consts::TAU;
    Vec3::new(-angle.sin(), 0.0, angle.cos()).normalize_or_zero()
}

fn updraft_swirl_displacement_on_axis(tangent: Vec3, displacement: Vec3) -> f32 {
    Vec3::new(displacement.x, 0.0, displacement.z)
        .dot(tangent)
        .abs()
}

fn rotation_from_x_to_direction(direction: Vec3) -> Quat {
    Quat::from_rotation_arc(Vec3::X, direction.normalize_or_zero())
}

fn wind_lateral_axis(direction: Vec3) -> Vec3 {
    Vec3::new(-direction.z, 0.0, direction.x).normalize_or_zero()
}

fn horizontal_or(v: Vec3, fallback: Vec3) -> Vec3 {
    let horizontal = Vec3::new(v.x, 0.0, v.z);
    if horizontal.length_squared() > 0.0001 {
        horizontal.normalize()
    } else {
        fallback.normalize_or_zero()
    }
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
    fn updraft_guide_scale_pulses_with_sampled_flow() {
        let guide = test_updraft_guide();
        let start = updraft_guide_position(&guide, 0.0);
        let later = updraft_guide_position(&guide, 1.7);
        let start_scale = updraft_guide_scale(&guide, start, 0.0);
        let later_scale = updraft_guide_scale(&guide, later, 1.7);

        assert!(
            scale_delta(later_scale, start_scale) > 0.08,
            "expected updraft motes to visibly pulse with sampled flow, start={start_scale:?}, later={later_scale:?}"
        );
    }

    #[test]
    fn wind_guide_metrics_capture_updraft_streamline_motion() {
        let guide = test_updraft_guide();
        let translation = updraft_guide_position(&guide, 2.0);
        let transform = Transform {
            translation,
            scale: updraft_guide_scale(&guide, translation, 2.0),
            ..default()
        };
        let metrics = wind_guide_visual_metrics(
            2.0,
            [(&guide, &transform)].into_iter(),
            std::iter::empty::<(&UpdraftRibbon, &Transform)>(),
            std::iter::empty::<(&CrosswindGuide, &Transform)>(),
            std::iter::empty::<(&CrosswindRibbon, &Transform)>(),
        );

        assert_eq!(metrics.updraft_guide_count, 1);
        assert!(metrics.max_updraft_visual_motion_m > 3.0);
        assert!(metrics.max_updraft_visual_rise_m > 3.0);
        assert!(metrics.max_updraft_visual_swirl_displacement_m > 0.2);
        assert!(metrics.max_updraft_visual_scale_pulse > 0.08);
        assert_eq!(metrics.updraft_flow_coherent_visual_count, 1);
        assert!(metrics.max_updraft_visual_flow_alignment > 0.55);
    }

    #[test]
    fn wind_guide_metrics_count_updraft_field_visual_coverage() {
        let elapsed = 2.0;
        let covered_field = WindField::updraft(Vec3::ZERO, Vec3::new(8.0, 16.0, 8.0), 12.0);
        let covered_guide = UpdraftGuide {
            field: covered_field,
            center: covered_field.center,
            radius: 4.0,
            height_offset: -10.0,
            phase: 0.31,
            angular_speed: 0.34,
        };
        let covered_guide_translation = updraft_guide_position(&covered_guide, elapsed);
        let covered_guide_transform = Transform {
            translation: covered_guide_translation,
            scale: updraft_guide_scale(&covered_guide, covered_guide_translation, elapsed),
            ..default()
        };
        let covered_ribbon = UpdraftRibbon {
            field: covered_field,
            spin_speed: 0.05,
            base_translation: covered_field.center,
            base_rotation: Quat::IDENTITY,
            phase: 0.2,
        };
        let covered_ribbon_transform = updraft_ribbon_transform(&covered_ribbon, elapsed);
        let guide_only_field =
            WindField::updraft(Vec3::new(32.0, 0.0, 0.0), Vec3::new(8.0, 16.0, 8.0), 12.0);
        let guide_only = UpdraftGuide {
            field: guide_only_field,
            center: guide_only_field.center,
            radius: 4.0,
            height_offset: -8.0,
            phase: 0.48,
            angular_speed: 0.3,
        };
        let guide_only_translation = updraft_guide_position(&guide_only, elapsed);
        let guide_only_transform = Transform {
            translation: guide_only_translation,
            scale: updraft_guide_scale(&guide_only, guide_only_translation, elapsed),
            ..default()
        };
        let missing_expected_field =
            WindField::updraft(Vec3::new(64.0, 0.0, 0.0), Vec3::new(8.0, 16.0, 8.0), 12.0);
        let unexpected_field =
            WindField::updraft(Vec3::new(96.0, 0.0, 0.0), Vec3::new(8.0, 16.0, 8.0), 12.0);
        let unexpected_guide = UpdraftGuide {
            field: unexpected_field,
            center: unexpected_field.center,
            radius: 4.0,
            height_offset: -8.0,
            phase: 0.18,
            angular_speed: 0.28,
        };
        let unexpected_guide_translation = updraft_guide_position(&unexpected_guide, elapsed);
        let unexpected_guide_transform = Transform {
            translation: unexpected_guide_translation,
            scale: updraft_guide_scale(&unexpected_guide, unexpected_guide_translation, elapsed),
            ..default()
        };
        let unexpected_ribbon = UpdraftRibbon {
            field: unexpected_field,
            spin_speed: 0.05,
            base_translation: unexpected_field.center,
            base_rotation: Quat::IDENTITY,
            phase: 0.44,
        };
        let unexpected_ribbon_transform = updraft_ribbon_transform(&unexpected_ribbon, elapsed);

        let metrics = wind_guide_visual_metrics_for_expected_fields(
            elapsed,
            &[covered_field, guide_only_field, missing_expected_field],
            &[],
            [
                (&covered_guide, &covered_guide_transform),
                (&guide_only, &guide_only_transform),
                (&unexpected_guide, &unexpected_guide_transform),
            ]
            .into_iter(),
            [
                (&covered_ribbon, &covered_ribbon_transform),
                (&unexpected_ribbon, &unexpected_ribbon_transform),
            ]
            .into_iter(),
            std::iter::empty::<(&CrosswindGuide, &Transform)>(),
            std::iter::empty::<(&CrosswindRibbon, &Transform)>(),
        );

        assert_eq!(metrics.updraft_field_count, 3);
        assert_eq!(metrics.updraft_fields_with_guides_count, 2);
        assert_eq!(metrics.updraft_fields_with_ribbons_count, 1);
        assert_eq!(metrics.updraft_fields_with_guides_and_ribbons_count, 1);
        assert_eq!(metrics.updraft_flow_coherent_field_count, 2);
    }

    #[test]
    fn wind_guide_metrics_capture_crosswind_flow_direction_motion() {
        let field = WindField::crosswind(
            Vec3::ZERO,
            Vec3::new(12.0, 6.0, 8.0),
            Vec3::new(-1.0, 0.0, 0.35),
            10.0,
        );
        let guide = CrosswindGuide {
            field,
            stream_index: 4,
            stream_count: 16,
            phase: 0.12,
        };
        let transform = Transform::from_translation(crosswind_guide_position(&guide, 2.0));
        let metrics = wind_guide_visual_metrics(
            2.0,
            std::iter::empty::<(&UpdraftGuide, &Transform)>(),
            std::iter::empty::<(&UpdraftRibbon, &Transform)>(),
            [(&guide, &transform)].into_iter(),
            std::iter::empty::<(&CrosswindRibbon, &Transform)>(),
        );

        assert_eq!(metrics.crosswind_guide_count, 1);
        assert!(metrics.max_crosswind_visual_motion_m > 3.0);
        assert!(metrics.max_crosswind_guide_flow_displacement_m > 3.0);
        assert_eq!(metrics.crosswind_flow_coherent_visual_count, 1);
        assert!(metrics.max_crosswind_visual_flow_alignment > 0.55);
    }

    #[test]
    fn wind_guide_metrics_count_crosswind_field_coherence_once_per_field() {
        let elapsed = 1.5;
        let coherent_field = WindField::crosswind(
            Vec3::ZERO,
            Vec3::new(12.0, 6.0, 8.0),
            Vec3::new(-1.0, 0.0, 0.35),
            10.0,
        );
        let coherent_guide = CrosswindGuide {
            field: coherent_field,
            stream_index: 4,
            stream_count: 16,
            phase: 0.12,
        };
        let coherent_guide_transform =
            Transform::from_translation(crosswind_guide_position(&coherent_guide, elapsed));
        let coherent_ribbon = CrosswindRibbon {
            field: coherent_field,
            base_translation: coherent_field.center,
            phase: 0.18,
        };
        let coherent_ribbon_transform = crosswind_ribbon_transform(&coherent_ribbon, elapsed);
        let opposing_field = WindField::crosswind(
            Vec3::new(36.0, 0.0, 0.0),
            Vec3::new(12.0, 6.0, 8.0),
            Vec3::X,
            10.0,
        );
        let opposing_guide = CrosswindGuide {
            field: opposing_field,
            stream_index: 3,
            stream_count: 16,
            phase: 0.36,
        };
        let opposing_future =
            crosswind_guide_position(&opposing_guide, elapsed + WIND_VISUAL_COHERENCE_DT);
        let opposing_transform =
            Transform::from_translation(opposing_future + opposing_field.direction * 4.0);
        let missing_expected_field = WindField::crosswind(
            Vec3::new(72.0, 0.0, 0.0),
            Vec3::new(12.0, 6.0, 8.0),
            Vec3::X,
            10.0,
        );
        let unexpected_field = WindField::crosswind(
            Vec3::new(108.0, 0.0, 0.0),
            Vec3::new(12.0, 6.0, 8.0),
            Vec3::X,
            10.0,
        );
        let unexpected_guide = CrosswindGuide {
            field: unexpected_field,
            stream_index: 5,
            stream_count: 16,
            phase: 0.2,
        };
        let unexpected_guide_transform =
            Transform::from_translation(crosswind_guide_position(&unexpected_guide, elapsed));
        let unexpected_ribbon = CrosswindRibbon {
            field: unexpected_field,
            base_translation: unexpected_field.center,
            phase: 0.33,
        };
        let unexpected_ribbon_transform = crosswind_ribbon_transform(&unexpected_ribbon, elapsed);

        let metrics = wind_guide_visual_metrics_for_expected_fields(
            elapsed,
            &[],
            &[coherent_field, opposing_field, missing_expected_field],
            std::iter::empty::<(&UpdraftGuide, &Transform)>(),
            std::iter::empty::<(&UpdraftRibbon, &Transform)>(),
            [
                (&coherent_guide, &coherent_guide_transform),
                (&opposing_guide, &opposing_transform),
                (&unexpected_guide, &unexpected_guide_transform),
            ]
            .into_iter(),
            [
                (&coherent_ribbon, &coherent_ribbon_transform),
                (&unexpected_ribbon, &unexpected_ribbon_transform),
            ]
            .into_iter(),
        );

        assert_eq!(metrics.crosswind_field_count, 3);
        assert_eq!(metrics.crosswind_fields_with_guides_count, 2);
        assert_eq!(metrics.crosswind_fields_with_ribbons_count, 1);
        assert_eq!(metrics.crosswind_fields_with_guides_and_ribbons_count, 1);
        assert_eq!(metrics.crosswind_flow_coherent_field_count, 1);
        assert!(
            metrics.crosswind_flow_coherent_visual_count
                > metrics.crosswind_flow_coherent_field_count
        );
    }

    #[test]
    fn crosswind_guides_vary_by_stream_without_losing_downwind_flow() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::new(18.0, 8.0, 10.0), Vec3::X, 14.0);
        let leading = CrosswindGuide {
            field,
            stream_index: 1,
            stream_count: 16,
            phase: 0.07,
        };
        let trailing = CrosswindGuide {
            field,
            stream_index: 11,
            stream_count: 16,
            phase: 0.63,
        };
        let elapsed = 0.6;
        let leading_displacement =
            crosswind_guide_position(&leading, elapsed) - crosswind_guide_position(&leading, 0.0);
        let trailing_displacement =
            crosswind_guide_position(&trailing, elapsed) - crosswind_guide_position(&trailing, 0.0);

        assert!(
            leading_displacement.dot(field.direction) > 1.0
                && trailing_displacement.dot(field.direction) > 1.0,
            "expected both streams to keep moving downwind, leading={leading_displacement:?}, trailing={trailing_displacement:?}"
        );
        assert!(
            (leading_displacement.length() - trailing_displacement.length()).abs() > 0.3,
            "expected stream-specific motion variation, leading={leading_displacement:?}, trailing={trailing_displacement:?}"
        );
    }

    #[test]
    fn wind_guide_metrics_do_not_treat_lateral_jitter_as_crosswind_flow() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::new(12.0, 6.0, 8.0), Vec3::X, 10.0);
        let guide = CrosswindGuide {
            field,
            stream_index: 4,
            stream_count: 16,
            phase: 0.12,
        };
        let baseline = crosswind_guide_position(&guide, 0.0);
        let transform =
            Transform::from_translation(baseline + wind_lateral_axis(field.direction) * 4.0);
        let metrics = wind_guide_visual_metrics(
            0.0,
            std::iter::empty::<(&UpdraftGuide, &Transform)>(),
            std::iter::empty::<(&UpdraftRibbon, &Transform)>(),
            [(&guide, &transform)].into_iter(),
            std::iter::empty::<(&CrosswindRibbon, &Transform)>(),
        );

        assert!(metrics.max_crosswind_visual_motion_m > 3.0);
        assert_eq!(metrics.max_crosswind_guide_flow_displacement_m, 0.0);
        assert_eq!(metrics.max_crosswind_ribbon_flow_displacement_m, 0.0);
    }

    #[test]
    fn wind_guide_metrics_capture_crosswind_ribbon_flow_direction_motion() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::new(12.0, 6.0, 8.0), Vec3::X, 10.0);
        let ribbon = CrosswindRibbon {
            field,
            base_translation: Vec3::ZERO,
            phase: 0.0,
        };
        let transform = crosswind_ribbon_transform(&ribbon, 1.5);
        let metrics = wind_guide_visual_metrics(
            1.5,
            std::iter::empty::<(&UpdraftGuide, &Transform)>(),
            std::iter::empty::<(&UpdraftRibbon, &Transform)>(),
            std::iter::empty::<(&CrosswindGuide, &Transform)>(),
            [(&ribbon, &transform)].into_iter(),
        );

        assert_eq!(metrics.crosswind_ribbon_count, 1);
        assert!(metrics.max_crosswind_visual_motion_m > 1.0);
        assert!(metrics.max_crosswind_ribbon_flow_displacement_m > 1.0);
        assert_eq!(metrics.crosswind_flow_coherent_visual_count, 1);
        assert!(metrics.max_crosswind_visual_flow_alignment > 0.55);
    }

    #[test]
    fn wind_guide_metrics_capture_visual_depth_and_pulse() {
        let updraft_field = WindField::updraft(Vec3::ZERO, Vec3::new(18.0, 30.0, 18.0), 16.0);
        let updraft_ribbon = UpdraftRibbon {
            field: updraft_field,
            spin_speed: 0.08,
            base_translation: Vec3::ZERO,
            base_rotation: Quat::IDENTITY,
            phase: 0.2,
        };
        let updraft_transform = updraft_ribbon_transform(&updraft_ribbon, 2.25);
        let crosswind_field =
            WindField::crosswind(Vec3::ZERO, Vec3::new(30.0, 18.0, 18.0), Vec3::X, 14.0);
        let crosswind_leading = CrosswindGuide {
            field: crosswind_field,
            stream_index: 0,
            stream_count: 16,
            phase: 0.12,
        };
        let leading_translation = crosswind_guide_position(&crosswind_leading, 2.25);
        let leading_transform = Transform {
            translation: leading_translation,
            scale: crosswind_guide_scale(&crosswind_leading, leading_translation, 2.25),
            ..default()
        };
        let crosswind_trailing = CrosswindGuide {
            field: crosswind_field,
            stream_index: 15,
            stream_count: 16,
            phase: 0.62,
        };
        let trailing_translation = crosswind_guide_position(&crosswind_trailing, 2.25);
        let trailing_transform = Transform {
            translation: trailing_translation,
            scale: crosswind_guide_scale(&crosswind_trailing, trailing_translation, 2.25),
            ..default()
        };

        let metrics = wind_guide_visual_metrics(
            2.25,
            std::iter::empty::<(&UpdraftGuide, &Transform)>(),
            [(&updraft_ribbon, &updraft_transform)].into_iter(),
            [
                (&crosswind_leading, &leading_transform),
                (&crosswind_trailing, &trailing_transform),
            ]
            .into_iter(),
            std::iter::empty::<(&CrosswindRibbon, &Transform)>(),
        );

        assert!(metrics.max_updraft_visual_depth_span_m > 55.0);
        assert!(metrics.max_updraft_visual_scale_pulse > 0.06);
        assert!(metrics.max_crosswind_visual_lane_depth_span_m > 18.0);
        assert!(metrics.max_crosswind_visual_scale_pulse > 0.1);
    }

    #[test]
    fn wind_guide_metrics_ignore_ribbon_initial_phase_offsets() {
        let updraft_field = WindField::updraft(Vec3::ZERO, Vec3::new(8.0, 16.0, 8.0), 12.0);
        let updraft_guide = test_updraft_guide();
        let updraft_guide_translation = updraft_guide_position(&updraft_guide, 0.0);
        let updraft_guide_transform = Transform {
            translation: updraft_guide_translation,
            scale: updraft_guide_scale(&updraft_guide, updraft_guide_translation, 0.0),
            ..default()
        };
        let updraft_ribbon = UpdraftRibbon {
            field: updraft_field,
            spin_speed: 0.05,
            base_translation: Vec3::ZERO,
            base_rotation: Quat::IDENTITY,
            phase: 0.2,
        };
        let crosswind_field =
            WindField::crosswind(Vec3::ZERO, Vec3::new(12.0, 6.0, 8.0), Vec3::X, 10.0);
        let crosswind_guide = CrosswindGuide {
            field: crosswind_field,
            stream_index: 4,
            stream_count: 16,
            phase: 0.12,
        };
        let crosswind_guide_translation = crosswind_guide_position(&crosswind_guide, 0.0);
        let crosswind_guide_transform = Transform {
            translation: crosswind_guide_translation,
            scale: crosswind_guide_scale(&crosswind_guide, crosswind_guide_translation, 0.0),
            ..default()
        };
        let crosswind_ribbon = CrosswindRibbon {
            field: crosswind_field,
            base_translation: Vec3::ZERO,
            phase: 0.18,
        };
        let updraft_transform = updraft_ribbon_transform(&updraft_ribbon, 0.0);
        let crosswind_transform = crosswind_ribbon_transform(&crosswind_ribbon, 0.0);
        let metrics = wind_guide_visual_metrics(
            0.0,
            [(&updraft_guide, &updraft_guide_transform)].into_iter(),
            [(&updraft_ribbon, &updraft_transform)].into_iter(),
            [(&crosswind_guide, &crosswind_guide_transform)].into_iter(),
            [(&crosswind_ribbon, &crosswind_transform)].into_iter(),
        );

        assert_eq!(metrics.max_updraft_visual_motion_m, 0.0);
        assert_eq!(metrics.max_updraft_visual_rise_m, 0.0);
        assert_eq!(metrics.max_updraft_visual_swirl_displacement_m, 0.0);
        assert_eq!(metrics.max_updraft_visual_scale_pulse, 0.0);
        assert_eq!(metrics.max_crosswind_visual_motion_m, 0.0);
        assert_eq!(metrics.max_crosswind_ribbon_flow_displacement_m, 0.0);
        assert_eq!(metrics.max_crosswind_visual_scale_pulse, 0.0);
    }

    #[test]
    fn crosswind_ribbon_transform_advects_along_shared_flow() {
        let field = WindField::crosswind(
            Vec3::ZERO,
            Vec3::new(14.0, 6.0, 8.0),
            Vec3::new(0.4, 0.0, 1.0),
            10.0,
        );
        let ribbon = CrosswindRibbon {
            field,
            base_translation: Vec3::ZERO,
            phase: 0.18,
        };
        let start = crosswind_ribbon_transform(&ribbon, 0.0);
        let later = crosswind_ribbon_transform(&ribbon, 2.0);
        let displacement = later.translation - start.translation;

        assert!(
            displacement.dot(field.direction) > 4.0,
            "expected ribbon to advect downwind, displacement={displacement:?}"
        );
        assert!(
            start.rotation.angle_between(later.rotation) > 0.1,
            "expected ribbon orientation to flutter with dynamic flow"
        );
        assert!(
            scale_delta(later.scale, start.scale) > 0.06,
            "expected ribbon to visibly pulse while advecting, start={:?}, later={:?}",
            start.scale,
            later.scale
        );
    }

    #[test]
    fn crosswind_ribbon_rotation_tracks_diagonal_flow_once() {
        let field = WindField::crosswind(
            Vec3::ZERO,
            Vec3::new(14.0, 6.0, 8.0),
            Vec3::new(-1.0, 0.0, 0.35),
            10.0,
        );
        let ribbon = CrosswindRibbon {
            field,
            base_translation: Vec3::ZERO,
            phase: 0.18,
        };
        let transform = crosswind_ribbon_transform(&ribbon, 2.0);
        let local_x = transform.rotation * Vec3::X;

        assert!(
            local_x.dot(field.direction) > 0.92,
            "expected ribbon local X to align with field flow, local_x={local_x:?}, direction={:?}",
            field.direction
        );
    }

    #[test]
    fn updraft_ribbon_transform_scrolls_and_breathes() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(8.0, 16.0, 8.0), 12.0);
        let ribbon = UpdraftRibbon {
            field,
            spin_speed: 0.05,
            base_translation: Vec3::ZERO,
            base_rotation: Quat::IDENTITY,
            phase: 0.2,
        };
        let start = updraft_ribbon_transform(&ribbon, 0.0);
        let later = updraft_ribbon_transform(&ribbon, 2.0);
        let displacement = later.translation - start.translation;

        assert!(
            displacement.y.abs() > 1.0,
            "expected ribbon to visibly scroll through the updraft, displacement={displacement:?}"
        );
        assert!(
            Vec2::new(displacement.x, displacement.z).length() > 0.2,
            "expected ribbon to breathe laterally with updraft swirl, displacement={displacement:?}"
        );
        assert!(
            scale_delta(later.scale, start.scale) > 0.06,
            "expected updraft ribbon to pulse with sampled flow, start={:?}, later={:?}",
            start.scale,
            later.scale
        );
    }

    #[test]
    fn updraft_ribbon_scene_samples_cover_readable_upper_spiral() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(12.0, 30.0, 10.0), 18.0);
        let ribbon = UpdraftRibbon {
            field,
            spin_speed: 0.05,
            base_translation: Vec3::ZERO,
            base_rotation: Quat::IDENTITY,
            phase: 0.25,
        };
        let transform = Transform::from_translation(Vec3::new(4.0, 20.0, -3.0));

        let samples = updraft_ribbon_scene_sample_positions(&ribbon, &transform);
        assert_eq!(samples.len(), 3);
        assert!(samples[0].y > transform.translation.y);
        assert!(samples[0].y < samples[1].y);
        assert!(samples[1].y < samples[2].y);
        assert!(
            samples
                .iter()
                .all(|position| position.xz().distance(transform.translation.xz()) > 2.0)
        );
    }

    #[test]
    fn crosswind_ribbon_scene_samples_cover_flow_length() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::new(20.0, 5.0, 4.0), Vec3::X, 22.0);
        let ribbon = CrosswindRibbon {
            field,
            base_translation: Vec3::ZERO,
            phase: 0.17,
        };
        let transform = Transform::from_translation(Vec3::new(10.0, 4.0, -2.0));

        let samples = crosswind_ribbon_scene_sample_positions(&ribbon, &transform);
        assert_eq!(samples.len(), 3);
        assert!(samples[0].x < transform.translation.x);
        assert!(samples[2].x > transform.translation.x);
        let straight_midpoint = (samples[0] + samples[2]) * 0.5;
        assert!(
            samples[1].distance(straight_midpoint) > 0.45,
            "expected curved crosswind ribbon samples to leave the straight centerline"
        );
    }

    #[test]
    fn wind_guide_metrics_capture_updraft_ribbon_rise() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(8.0, 16.0, 8.0), 12.0);
        let ribbon = UpdraftRibbon {
            field,
            spin_speed: 0.05,
            base_translation: Vec3::ZERO,
            base_rotation: Quat::IDENTITY,
            phase: 0.2,
        };
        let transform = updraft_ribbon_transform(&ribbon, 3.5);
        let metrics = wind_guide_visual_metrics(
            3.5,
            std::iter::empty::<(&UpdraftGuide, &Transform)>(),
            [(&ribbon, &transform)].into_iter(),
            std::iter::empty::<(&CrosswindGuide, &Transform)>(),
            std::iter::empty::<(&CrosswindRibbon, &Transform)>(),
        );

        assert_eq!(metrics.updraft_ribbon_count, 1);
        assert!(metrics.max_updraft_visual_motion_m > 0.5);
        assert!(metrics.max_updraft_visual_rise_m > 0.5);
        assert!(metrics.max_updraft_visual_swirl_displacement_m > 0.1);
        assert_eq!(metrics.updraft_flow_coherent_visual_count, 1);
        assert!(metrics.max_updraft_visual_flow_alignment > 0.55);
    }
}
