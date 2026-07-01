use crate::Player;
use crate::authored_assets::VisualAssetRegistry;
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::eval_runtime::EvalRun;
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
#[cfg(test)]
use nau_engine::environment::wind_gust_front_progress;
use nau_engine::environment::{
    GAMEPLAY_LIFT_ROUTE, LiftRouteNode, WindField, WindFlowSample, visual_crosswind_fields,
    wind_gust_packet_strength, wind_sway_motion,
};
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::SkyIsland;

pub(crate) const UPDRAFT_RIBBONS_PER_FIELD: usize = 6;
pub(crate) const UPDRAFT_GUIDE_RING_LEVELS: [f32; 7] = [-0.86, -0.56, -0.24, 0.08, 0.4, 0.72, 0.94];
pub(crate) const UPDRAFT_GUIDES_PER_RING: usize = 15;
pub(crate) const CROSSWIND_RIBBONS_PER_FIELD: usize = 7;
pub(crate) const CROSSWIND_GUIDES_PER_FIELD: usize = 60;
pub(crate) const CROSSWIND_RIBBON_LENGTH_SCALE: f32 = 0.92;
pub(crate) const CROSSWIND_RIBBON_CENTER_ADVANCE: f32 = 0.95;
pub(crate) const WIND_VISUAL_COHERENCE_DT: f32 = 0.2;
pub(crate) const WIND_VISUAL_ALIGNMENT_MIN_DOT: f32 = 0.55;
const WIND_FIELD_METRIC_EPSILON: f32 = 0.001;
const WIND_VISUAL_LOOP_FADE_FRACTION: f32 = 0.24;
const WIND_VISUAL_QUALITY_MIN_SCALE: f32 = 0.5;

pub(crate) fn updraft_guide_ring_radius(field_radius: f32) -> f32 {
    (field_radius * 0.5).min(10.0)
}

pub(crate) fn updraft_guide_angular_speed(level_index: usize) -> f32 {
    0.16 + level_index as f32 * 0.02
}

#[derive(Component)]
pub(crate) struct CinematicSun;

fn wind_visual_elapsed_secs(time: &Time, eval_run: Option<&EvalRun>) -> f32 {
    eval_wind_visual_elapsed_secs(
        time.elapsed_secs(),
        eval_run.map(|run| (run.frame, run.scenario.fixed_dt)),
    )
}

fn eval_wind_visual_elapsed_secs(
    wall_clock_elapsed_secs: f32,
    eval_frame_and_dt: Option<(u32, f32)>,
) -> f32 {
    eval_frame_and_dt
        .map(|(frame, fixed_dt)| frame as f32 * fixed_dt)
        .unwrap_or(wall_clock_elapsed_secs)
}

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
    pub(crate) field: WindField,
    pub(crate) center: Vec3,
    pub(crate) radius: f32,
    pub(crate) height_offset: f32,
    pub(crate) phase: f32,
    pub(crate) angular_speed: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct UpdraftRibbon {
    pub(crate) field: WindField,
    pub(crate) spin_speed: f32,
    pub(crate) base_translation: Vec3,
    pub(crate) base_rotation: Quat,
    pub(crate) phase: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct CrosswindGuide {
    pub(crate) field: WindField,
    pub(crate) stream_index: usize,
    pub(crate) stream_count: usize,
    pub(crate) phase: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct CrosswindRibbon {
    pub(crate) field: WindField,
    pub(crate) base_translation: Vec3,
    pub(crate) phase: f32,
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
    pub(crate) crosswind_ribbon_flow_coherent_sample_count: usize,
    pub(crate) max_updraft_visual_flow_alignment: f32,
    pub(crate) max_crosswind_visual_flow_alignment: f32,
    pub(crate) max_crosswind_ribbon_visual_flow_alignment: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ObservedWindVisualMotionMetrics {
    pub(crate) observed_updraft_flow_coherent_visual_count: usize,
    pub(crate) observed_crosswind_flow_coherent_visual_count: usize,
    pub(crate) observed_crosswind_ribbon_flow_coherent_sample_count: usize,
    pub(crate) max_observed_updraft_visual_frame_motion_m: f32,
    pub(crate) max_observed_updraft_visual_frame_rise_m: f32,
    pub(crate) max_observed_updraft_visual_frame_swirl_displacement_m: f32,
    pub(crate) max_observed_crosswind_visual_frame_motion_m: f32,
    pub(crate) max_observed_crosswind_guide_frame_flow_displacement_m: f32,
    pub(crate) max_observed_crosswind_ribbon_frame_flow_displacement_m: f32,
    pub(crate) max_observed_updraft_visual_speed_mps: f32,
    pub(crate) max_observed_crosswind_visual_speed_mps: f32,
    pub(crate) max_observed_wind_visual_acceleration_mps2: f32,
    pub(crate) observed_wind_visual_jump_count: u32,
    pub(crate) max_observed_updraft_visual_flow_alignment: f32,
    pub(crate) max_observed_crosswind_visual_flow_alignment: f32,
    pub(crate) max_observed_crosswind_ribbon_visual_flow_alignment: f32,
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
    let ring_radius = updraft_guide_ring_radius(radius);

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
                angular_speed: updraft_guide_angular_speed(level_index),
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
    let ribbon_length = (field.half_extents.x * CROSSWIND_RIBBON_LENGTH_SCALE).max(3.0);
    let marker_mesh = meshes.add(Cuboid::new(0.74, 0.07, 0.14));
    let base_rotation = rotation_from_x_to_direction(field.direction);

    for ribbon_index in 0..CROSSWIND_RIBBONS_PER_FIELD {
        let phase = ribbon_index as f32 / CROSSWIND_RIBBONS_PER_FIELD as f32;
        let ribbon_mesh = meshes.add(crosswind_flow_ribbon_mesh(
            ribbon_length,
            phase * std::f32::consts::TAU,
        ));
        let origin = field.stream_origin(ribbon_index, CROSSWIND_RIBBONS_PER_FIELD);
        let base_translation =
            origin + field.direction * (field.half_extents.x * CROSSWIND_RIBBON_CENTER_ADVANCE);
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
                rotation: crosswind_guide_rotation(&guide, translation, 0.0),
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
    eval_run: Option<Res<EvalRun>>,
    mut columns: Query<(&UpdraftColumn, &mut Transform)>,
) {
    let elapsed = wind_visual_elapsed_secs(&time, eval_run.as_deref());

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
    eval_run: Option<Res<EvalRun>>,
    mut guides: Query<(&UpdraftGuide, &mut Transform)>,
) {
    let elapsed = wind_visual_elapsed_secs(&time, eval_run.as_deref());

    for (guide, mut transform) in &mut guides {
        let translation = updraft_guide_position(guide, elapsed);
        transform.translation = translation;
        transform.rotation = Quat::from_rotation_y(guide.phase + elapsed * guide.angular_speed);
        transform.scale = updraft_guide_scale(guide, translation, elapsed);
    }
}

pub(crate) fn update_updraft_ribbons(
    time: Res<Time>,
    eval_run: Option<Res<EvalRun>>,
    mut ribbons: Query<(&UpdraftRibbon, &mut Transform)>,
) {
    let elapsed = wind_visual_elapsed_secs(&time, eval_run.as_deref());

    for (ribbon, mut transform) in &mut ribbons {
        *transform = updraft_ribbon_transform(ribbon, elapsed);
    }
}

pub(crate) fn update_crosswind_guides(
    time: Res<Time>,
    eval_run: Option<Res<EvalRun>>,
    mut guides: Query<(&CrosswindGuide, &mut Transform)>,
) {
    let elapsed = wind_visual_elapsed_secs(&time, eval_run.as_deref());

    for (guide, mut transform) in &mut guides {
        let translation = crosswind_guide_position(guide, elapsed);
        transform.translation = translation;
        transform.rotation = crosswind_guide_rotation(guide, translation, elapsed);
        transform.scale = crosswind_guide_scale(guide, translation, elapsed);
    }
}

pub(crate) fn update_crosswind_ribbons(
    time: Res<Time>,
    eval_run: Option<Res<EvalRun>>,
    mut ribbons: Query<(&CrosswindRibbon, &mut Transform)>,
) {
    let elapsed = wind_visual_elapsed_secs(&time, eval_run.as_deref());

    for (ribbon, mut transform) in &mut ribbons {
        *transform = crosswind_ribbon_transform(ribbon, elapsed);
    }
}

pub(crate) fn wind_visual_quality_visible(scale: Vec3) -> bool {
    scale.min_element() >= WIND_VISUAL_QUALITY_MIN_SCALE
}

pub(crate) fn updraft_ribbon_transform(ribbon: &UpdraftRibbon, elapsed: f32) -> Transform {
    let sample = updraft_ribbon_flow_sample(ribbon, elapsed);
    debug_assert!(ribbon.field.contains(sample.probe_position));
    let flow = sample.flow;
    let phase = ribbon.phase * std::f32::consts::TAU;
    let vertical_ratio = (flow.vector.y.max(0.0) / ribbon.field.visual_speed.max(1.0)).min(1.4);
    let progress = sample.progress;
    let gust_packet = travelling_gust_packet(
        elapsed,
        ribbon.phase + flow.variation * 0.21,
        progress,
        flow.gust_strength,
    )
    .max(flow.gust_packet_strength * 0.45)
    .max(flow.layered_gust_strength * 0.38);
    let vertical_scroll = sample.vertical_scroll;
    let horizontal_flow = Vec3::new(flow.vector.x, 0.0, flow.vector.z);
    let radial_axis = sample.radial_axis;
    let tangent_axis = Vec3::new(-radial_axis.z, 0.0, radial_axis.x).normalize_or_zero();
    let breathing = (elapsed * 1.18 + phase).sin();
    let scale_wave = (elapsed * 1.64 + phase * 0.7).sin();
    let thermal_roll =
        (elapsed * 0.76 + phase * 1.21 + progress * std::f32::consts::TAU * 1.64).sin();
    let lift_roll = (elapsed * 0.58 + phase * 0.67 + progress * std::f32::consts::TAU * 1.9).cos();
    let visibility = wind_loop_visibility(progress);
    let radial_breath = radial_axis
        * (flow.variation * 0.08
            + breathing * 0.05
            + gust_packet * 0.04
            + flow.layered_gust_strength * 0.035);
    let tangential_depth = tangent_axis
        * (thermal_roll * (0.025 + flow.variation * 0.035)
            + gust_packet * 0.025
            + flow.layered_gust_strength * 0.025);
    let horizontal_drift = horizontal_flow * 0.03;
    Transform {
        translation: ribbon.base_translation
            + horizontal_drift
            + Vec3::Y
                * (vertical_scroll
                    + flow.variation * 0.08
                    + gust_packet * 0.05
                    + flow.layered_gust_strength * 0.04
                    + lift_roll * (0.02 + flow.variation * 0.02))
            + radial_breath
            + tangential_depth,
        rotation: ribbon.base_rotation
            * Quat::from_rotation_y(elapsed * ribbon.spin_speed * 1.02)
            * Quat::from_rotation_x(thermal_roll * 0.012)
            * Quat::from_rotation_z(
                breathing * flow.variation * 0.035
                    + gust_packet * 0.015
                    + flow.layered_gust_strength * 0.012,
            ),
        scale: updraft_ribbon_scale_with_visibility(
            Vec3::new(
                1.0 + flow.variation * 0.04
                    + scale_wave * 0.025
                    + gust_packet * 0.02
                    + flow.layered_gust_strength * 0.015,
                1.0 + flow.gust_strength * 0.02
                    + vertical_ratio * 0.02
                    + scale_wave.abs() * 0.01
                    + gust_packet * 0.01
                    + flow.layered_gust_strength * 0.01,
                1.0 + flow.variation * 0.035 - scale_wave * 0.02
                    + gust_packet * 0.015
                    + flow.layered_gust_strength * 0.012,
            ),
            visibility,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct UpdraftRibbonFlowSample {
    flow: WindFlowSample,
    progress: f32,
    vertical_scroll: f32,
    radial_axis: Vec3,
    probe_position: Vec3,
}

fn updraft_ribbon_flow_sample(ribbon: &UpdraftRibbon, elapsed: f32) -> UpdraftRibbonFlowSample {
    let base_flow = ribbon
        .field
        .flow_at(ribbon.base_translation, elapsed)
        .unwrap_or_else(|| ribbon.field.flow_at(ribbon.field.center, elapsed).unwrap());
    let field_height = (ribbon.field.half_extents.y * 2.0).max(1.0);
    let vertical_ratio =
        (base_flow.vector.y.max(0.0) / ribbon.field.visual_speed.max(1.0)).min(1.4);
    let progress =
        (ribbon.phase + elapsed * ribbon.field.visual_speed.max(1.0) / field_height * 0.36).fract();
    let vertical_scroll =
        (progress - 0.5) * ribbon.field.half_extents.y * (0.385 + vertical_ratio * 0.06);
    let phase = ribbon.phase * std::f32::consts::TAU;
    let lane_angle = phase + progress * std::f32::consts::TAU * 0.72;
    let radial_axis = Vec3::new(lane_angle.cos(), 0.0, lane_angle.sin()).normalize_or_zero();
    let radius = ribbon.field.half_extents.x.min(ribbon.field.half_extents.z) * 0.42;
    let probe_position = ribbon.base_translation + Vec3::Y * vertical_scroll + radial_axis * radius;
    let flow = ribbon
        .field
        .flow_at(probe_position, elapsed)
        .unwrap_or(base_flow);

    UpdraftRibbonFlowSample {
        flow,
        progress,
        vertical_scroll,
        radial_axis,
        probe_position,
    }
}

pub(crate) fn crosswind_ribbon_transform(ribbon: &CrosswindRibbon, elapsed: f32) -> Transform {
    let base_flow = ribbon
        .field
        .flow_at(ribbon.base_translation, elapsed)
        .unwrap_or_else(|| ribbon.field.flow_at(ribbon.field.center, elapsed).unwrap());
    let lateral = wind_lateral_axis(ribbon.field.direction);
    let phase = ribbon.phase * std::f32::consts::TAU;
    let directional_half_extent = ribbon.field.half_extents.x.max(1.0);
    let path_length = (directional_half_extent * 2.0).max(1.0);
    let stream_variation = 0.88 + ribbon.phase * 0.24;
    let progress = (ribbon.phase
        + elapsed * ribbon.field.visual_speed.max(1.0) / path_length * 0.78 * stream_variation)
        .fract();
    let advected = (progress - 0.5) * directional_half_extent;
    let advected_center = ribbon.base_translation + ribbon.field.direction * advected;
    let flow = ribbon
        .field
        .flow_at(advected_center, elapsed)
        .unwrap_or(base_flow);
    let gust_packet = travelling_gust_packet(
        elapsed,
        ribbon.phase + flow.variation * 0.19,
        progress,
        flow.gust_strength,
    )
    .max(flow.gust_packet_strength * 0.35)
    .max(flow.layered_gust_strength * 0.34);
    let wave = (elapsed * 0.58 + phase).sin();
    let lift_wave = (elapsed * 0.82 + ribbon.phase * 4.1).cos();
    let depth_wave = (elapsed * 0.62 + phase * 1.17 + progress * std::f32::consts::TAU * 1.1).sin();
    let vertical_roll =
        (elapsed * 0.48 + phase * 0.83 + progress * std::f32::consts::TAU * 1.24).cos();
    let flow_axis = ribbon.field.direction;
    let gust_front = flow
        .gust_packet_strength
        .max(flow.layered_gust_strength * 0.72)
        .max(gust_packet * 0.5);
    let length_pulse =
        (1.03 + flow.variation * 0.06 + gust_front * 0.035 + flow.layered_gust_strength * 0.025)
            .clamp(1.02, 1.18);
    let width_pulse =
        (0.76 + flow.variation * 0.42 + wave.abs() * 0.06 + gust_front * 0.05).clamp(0.84, 1.18);
    let visibility = wind_loop_visibility(progress);

    Transform {
        translation: ribbon.base_translation
            + flow_axis
                * (advected
                    + gust_front * flow.variation * 0.1
                    + gust_packet * flow.variation * 0.1
                    + flow.layered_gust_strength * flow.variation * 0.08)
            + lateral
                * (flow.variation * 0.9
                    + wave * 0.28
                    + gust_packet * 0.16
                    + flow.layered_gust_strength * 0.16
                    + depth_wave * (0.12 + flow.variation * 0.14))
            + Vec3::Y
                * (lift_wave * 0.16
                    + flow.variation * 0.18
                    + gust_packet * 0.1
                    + flow.layered_gust_strength * 0.12
                    + vertical_roll * (0.04 + flow.variation * 0.06)),
        rotation: rotation_from_x_to_direction(flow_axis)
            * Quat::from_rotation_x(phase + elapsed * 0.14)
            * Quat::from_rotation_y(depth_wave * 0.035)
            * Quat::from_rotation_z(
                wave * 0.08
                    + vertical_roll * 0.025
                    + gust_packet * 0.025
                    + flow.layered_gust_strength * 0.025,
            ),
        scale: crosswind_ribbon_scale_with_visibility(
            Vec3::new(
                length_pulse,
                width_pulse + gust_packet * 0.045 + flow.layered_gust_strength * 0.035,
                width_pulse + gust_packet * 0.045 + flow.layered_gust_strength * 0.035,
            ),
            visibility,
        ),
    }
}

fn updraft_ribbon_scale_with_visibility(scale: Vec3, visibility: f32) -> Vec3 {
    Vec3::new(scale.x * visibility, scale.y, scale.z * visibility)
}

fn crosswind_ribbon_scale_with_visibility(scale: Vec3, visibility: f32) -> Vec3 {
    Vec3::new(scale.x, scale.y * visibility, scale.z * visibility)
}

pub(crate) fn updraft_ribbon_scene_sample_positions(
    ribbon: &UpdraftRibbon,
    transform: &Transform,
) -> [Vec3; 3] {
    const STRANDS: f32 = 1.45;
    const STOPS: [f32; 3] = [0.56, 0.68, 0.80];

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
    const STOPS: [f32; 3] = [0.28, 0.5, 0.72];

    let length = (ribbon.field.half_extents.x * CROSSWIND_RIBBON_LENGTH_SCALE).max(3.0);
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
        let next_transform =
            crosswind_ribbon_transform(ribbon, elapsed_secs + WIND_VISUAL_COHERENCE_DT);
        let coherent = record_crosswind_flow_coherence(
            &mut metrics,
            ribbon.field,
            transform.translation,
            next_transform.translation,
            elapsed_secs,
        );
        let ribbon_coherent = record_crosswind_ribbon_advected_flow_coherence(
            &mut metrics,
            ribbon.field,
            crosswind_ribbon_scene_sample_positions(ribbon, transform),
            crosswind_ribbon_scene_sample_positions(ribbon, &next_transform),
            elapsed_secs,
        );
        if let (Some(field_index), true) = (field_index, coherent || ribbon_coherent) {
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
    let Some(alignment) = visual_flow_alignment(field, current, next, elapsed_secs, true, true)
    else {
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
    let Some(alignment) = visual_flow_alignment(field, current, next, elapsed_secs, false, true)
    else {
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

fn record_crosswind_ribbon_advected_flow_coherence(
    metrics: &mut WindGuideVisualMetrics,
    field: WindField,
    current_samples: [Vec3; 3],
    next_samples: [Vec3; 3],
    elapsed_secs: f32,
) -> bool {
    let mut coherent_samples = 0;
    for (current, next) in current_samples.into_iter().zip(next_samples) {
        let Some(alignment) =
            visual_flow_alignment(field, current, next, elapsed_secs, false, false)
        else {
            continue;
        };

        metrics.max_crosswind_ribbon_visual_flow_alignment = metrics
            .max_crosswind_ribbon_visual_flow_alignment
            .max(alignment);
        if alignment >= WIND_VISUAL_ALIGNMENT_MIN_DOT {
            coherent_samples += 1;
            metrics.crosswind_ribbon_flow_coherent_sample_count += 1;
        }
    }

    coherent_samples >= 2
}

pub(crate) fn observe_updraft_guide_frame_motion(
    metrics: &mut ObservedWindVisualMotionMetrics,
    guide: &UpdraftGuide,
    previous: &Transform,
    current: &Transform,
    elapsed_secs: f32,
    dt_secs: f32,
    previous_velocity: Option<Vec3>,
) {
    let frame_motion = ObservedFrameMotion::new(
        guide.field,
        previous.translation,
        current.translation,
        elapsed_secs,
        dt_secs,
        previous_velocity,
    );
    record_observed_updraft_frame_motion(metrics, frame_motion, |displacement| {
        updraft_swirl_displacement(guide.field, previous.translation, displacement)
    });
}

pub(crate) fn observe_updraft_ribbon_frame_motion(
    metrics: &mut ObservedWindVisualMotionMetrics,
    ribbon: &UpdraftRibbon,
    previous: &Transform,
    current: &Transform,
    elapsed_secs: f32,
    dt_secs: f32,
    previous_velocity: Option<Vec3>,
) {
    let frame_motion = ObservedFrameMotion::new(
        ribbon.field,
        previous.translation,
        current.translation,
        elapsed_secs,
        dt_secs,
        previous_velocity,
    );
    record_observed_updraft_frame_motion(metrics, frame_motion, |displacement| {
        updraft_swirl_displacement_on_axis(updraft_ribbon_swirl_axis(ribbon), displacement)
    });
}

pub(crate) fn observe_crosswind_guide_frame_motion(
    metrics: &mut ObservedWindVisualMotionMetrics,
    guide: &CrosswindGuide,
    previous: &Transform,
    current: &Transform,
    elapsed_secs: f32,
    dt_secs: f32,
    previous_velocity: Option<Vec3>,
) {
    let frame_motion = ObservedFrameMotion::new(
        guide.field,
        previous.translation,
        current.translation,
        elapsed_secs,
        dt_secs,
        previous_velocity,
    );
    record_observed_crosswind_frame_motion(metrics, frame_motion, false);
}

pub(crate) fn observe_crosswind_ribbon_frame_motion(
    metrics: &mut ObservedWindVisualMotionMetrics,
    ribbon: &CrosswindRibbon,
    previous: &Transform,
    current: &Transform,
    elapsed_secs: f32,
    dt_secs: f32,
    previous_velocity: Option<Vec3>,
) {
    let frame_motion = ObservedFrameMotion::new(
        ribbon.field,
        previous.translation,
        current.translation,
        elapsed_secs,
        dt_secs,
        previous_velocity,
    );
    record_observed_crosswind_frame_motion(metrics, frame_motion, true);
    for (previous_sample, current_sample) in
        crosswind_ribbon_scene_sample_positions(ribbon, previous)
            .into_iter()
            .zip(crosswind_ribbon_scene_sample_positions(ribbon, current))
    {
        let Some(alignment) = observed_visual_flow_alignment(
            ribbon.field,
            previous_sample,
            current_sample,
            elapsed_secs,
            false,
            dt_secs,
        ) else {
            continue;
        };

        metrics.max_observed_crosswind_ribbon_visual_flow_alignment = metrics
            .max_observed_crosswind_ribbon_visual_flow_alignment
            .max(alignment);
        if alignment >= WIND_VISUAL_ALIGNMENT_MIN_DOT {
            metrics.observed_crosswind_ribbon_flow_coherent_sample_count += 1;
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ObservedFrameMotion {
    field: WindField,
    previous: Vec3,
    current: Vec3,
    elapsed_secs: f32,
    dt_secs: f32,
    previous_velocity: Option<Vec3>,
}

impl ObservedFrameMotion {
    fn new(
        field: WindField,
        previous: Vec3,
        current: Vec3,
        elapsed_secs: f32,
        dt_secs: f32,
        previous_velocity: Option<Vec3>,
    ) -> Self {
        Self {
            field,
            previous,
            current,
            elapsed_secs,
            dt_secs,
            previous_velocity,
        }
    }

    fn displacement(self) -> Vec3 {
        self.current - self.previous
    }
}

fn record_observed_updraft_frame_motion(
    metrics: &mut ObservedWindVisualMotionMetrics,
    frame_motion: ObservedFrameMotion,
    swirl_displacement: impl FnOnce(Vec3) -> f32,
) {
    let displacement = frame_motion.displacement();
    record_observed_visual_quality(
        metrics,
        frame_motion.field,
        displacement,
        frame_motion.dt_secs,
        frame_motion.previous_velocity,
        true,
    );
    let Some(alignment) = observed_visual_flow_alignment(
        frame_motion.field,
        frame_motion.previous,
        frame_motion.current,
        frame_motion.elapsed_secs,
        true,
        frame_motion.dt_secs,
    ) else {
        return;
    };

    metrics.max_observed_updraft_visual_frame_motion_m = metrics
        .max_observed_updraft_visual_frame_motion_m
        .max(displacement.length());
    metrics.max_observed_updraft_visual_frame_rise_m = metrics
        .max_observed_updraft_visual_frame_rise_m
        .max(displacement.y.max(0.0));
    metrics.max_observed_updraft_visual_frame_swirl_displacement_m = metrics
        .max_observed_updraft_visual_frame_swirl_displacement_m
        .max(swirl_displacement(displacement));
    metrics.max_observed_updraft_visual_flow_alignment = metrics
        .max_observed_updraft_visual_flow_alignment
        .max(alignment);
    if alignment >= WIND_VISUAL_ALIGNMENT_MIN_DOT {
        metrics.observed_updraft_flow_coherent_visual_count += 1;
    }
}

fn record_observed_crosswind_frame_motion(
    metrics: &mut ObservedWindVisualMotionMetrics,
    frame_motion: ObservedFrameMotion,
    ribbon: bool,
) {
    let displacement = frame_motion.displacement();
    record_observed_visual_quality(
        metrics,
        frame_motion.field,
        displacement,
        frame_motion.dt_secs,
        frame_motion.previous_velocity,
        false,
    );
    let Some(alignment) = observed_visual_flow_alignment(
        frame_motion.field,
        frame_motion.previous,
        frame_motion.current,
        frame_motion.elapsed_secs,
        false,
        frame_motion.dt_secs,
    ) else {
        return;
    };

    metrics.max_observed_crosswind_visual_frame_motion_m = metrics
        .max_observed_crosswind_visual_frame_motion_m
        .max(displacement.length());
    let flow_displacement_m = displacement.dot(frame_motion.field.direction).max(0.0);
    if ribbon {
        metrics.max_observed_crosswind_ribbon_frame_flow_displacement_m = metrics
            .max_observed_crosswind_ribbon_frame_flow_displacement_m
            .max(flow_displacement_m);
    } else {
        metrics.max_observed_crosswind_guide_frame_flow_displacement_m = metrics
            .max_observed_crosswind_guide_frame_flow_displacement_m
            .max(flow_displacement_m);
    }
    metrics.max_observed_crosswind_visual_flow_alignment = metrics
        .max_observed_crosswind_visual_flow_alignment
        .max(alignment);
    if alignment >= WIND_VISUAL_ALIGNMENT_MIN_DOT {
        metrics.observed_crosswind_flow_coherent_visual_count += 1;
    }
}

pub(crate) fn observed_wind_visual_velocity(
    previous: Vec3,
    current: Vec3,
    dt_secs: f32,
) -> Option<Vec3> {
    let dt = observed_visual_dt(dt_secs);
    (dt > 0.0).then_some((current - previous) / dt)
}

fn record_observed_visual_quality(
    metrics: &mut ObservedWindVisualMotionMetrics,
    field: WindField,
    displacement: Vec3,
    dt_secs: f32,
    previous_velocity: Option<Vec3>,
    updraft: bool,
) {
    let dt = observed_visual_dt(dt_secs);
    if dt <= 0.0 {
        return;
    }

    let velocity = displacement / dt;
    let speed = velocity.length();
    if updraft {
        metrics.max_observed_updraft_visual_speed_mps =
            metrics.max_observed_updraft_visual_speed_mps.max(speed);
    } else {
        metrics.max_observed_crosswind_visual_speed_mps =
            metrics.max_observed_crosswind_visual_speed_mps.max(speed);
    }

    if let Some(previous_velocity) = previous_velocity {
        metrics.max_observed_wind_visual_acceleration_mps2 = metrics
            .max_observed_wind_visual_acceleration_mps2
            .max((velocity - previous_velocity).length() / dt);
    }

    if observed_visual_step_is_jump(field, displacement, dt_secs) {
        metrics.observed_wind_visual_jump_count += 1;
    }
}

fn observed_visual_flow_alignment(
    field: WindField,
    current: Vec3,
    next: Vec3,
    elapsed_secs: f32,
    include_vertical: bool,
    dt_secs: f32,
) -> Option<f32> {
    let displacement = next - current;
    if observed_visual_step_is_jump(field, displacement, dt_secs) {
        return None;
    }

    visual_flow_alignment(field, current, next, elapsed_secs, include_vertical, false)
}

fn observed_visual_step_is_jump(field: WindField, displacement: Vec3, dt_secs: f32) -> bool {
    displacement.length() > observed_visual_plausible_step(field, dt_secs)
}

fn observed_visual_plausible_step(field: WindField, dt_secs: f32) -> f32 {
    let dt = observed_visual_dt(dt_secs);
    (field.visual_speed.max(1.0) * dt * 8.0 + 0.4)
        .min(field.half_extents.max_element().max(1.0) * 0.25)
}

fn observed_visual_dt(dt_secs: f32) -> f32 {
    dt_secs.clamp(1.0 / 240.0, 0.25)
}

pub(crate) fn visual_flow_alignment(
    field: WindField,
    current: Vec3,
    next: Vec3,
    elapsed_secs: f32,
    include_vertical: bool,
    allow_center_fallback: bool,
) -> Option<f32> {
    let displacement = next - current;
    let max_step = field.half_extents.max_element().max(1.0) * 0.5;
    if displacement.length_squared() <= 0.0001 || displacement.length() > max_step {
        return None;
    }
    if !allow_center_fallback && !field.contains(next) {
        return None;
    }

    let flow = match field.flow_at(current, elapsed_secs) {
        Some(flow) => flow,
        None if allow_center_fallback => field.flow_at(field.center, elapsed_secs)?,
        None => return None,
    };
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

pub(crate) fn updraft_guide_position(guide: &UpdraftGuide, elapsed: f32) -> Vec3 {
    let field = guide.field;
    let height_span = (field.half_extents.y * 1.84).max(1.0);
    let progress = updraft_guide_progress(guide, elapsed);
    let height_offset = (progress - 0.5) * height_span;
    let flow_probe = guide.center + Vec3::Y * height_offset;
    let flow = guide.field.flow_at(flow_probe, elapsed);
    let variation = flow.map_or(0.0, |sample| sample.variation);
    let gust = flow.map_or(1.0, |sample| sample.gust_strength);
    let gust_packet = flow.map_or(0.0, |sample| {
        sample
            .gust_packet_strength
            .max(sample.layered_gust_strength.max(
                travelling_gust_packet(elapsed, guide.phase, progress, sample.gust_strength) * 0.45,
            ))
    });
    let angle = guide.phase
        + elapsed * guide.angular_speed * 0.92
        + progress * std::f32::consts::TAU * 0.42
        + variation * 0.025
        + gust_packet * 0.02;
    let radius = guide.radius
        * (0.82
            + variation * 0.22
            + gust * 0.035
            + gust_packet * 0.035
            + (elapsed * 0.46 + guide.phase).sin() * 0.045)
            .clamp(0.78, 1.14);
    let base = guide.center
        + Vec3::new(
            angle.cos() * radius,
            height_offset + (elapsed * 0.68 + guide.phase).sin() * 0.22,
            angle.sin() * radius,
        );
    let flow_offset = flow.map_or(Vec3::ZERO, |sample| {
        Vec3::new(sample.vector.x, 0.0, sample.vector.z) * 0.025
            + Vec3::Y
                * (sample.vector.y.max(0.0) * 0.008 + sample.variation * 0.1 + gust_packet * 0.055)
            + Vec3::new(sample.vector.z, 0.0, -sample.vector.x).normalize_or_zero()
                * sample.layered_gust_strength
                * 0.035
    });

    base + flow_offset
}

pub(crate) fn updraft_guide_scale(guide: &UpdraftGuide, position: Vec3, elapsed: f32) -> Vec3 {
    let flow = guide
        .field
        .flow_at(position, elapsed)
        .unwrap_or_else(|| guide.field.flow_at(guide.field.center, elapsed).unwrap());
    let vertical_ratio = (flow.vector.y.max(0.0) / guide.field.visual_speed.max(1.0)).min(1.4);
    let height_span = (guide.field.half_extents.y * 1.84).max(1.0);
    let progress = ((position.y - guide.center.y) / height_span + 0.5).clamp(0.0, 1.0);
    let visibility = wind_loop_visibility(updraft_guide_progress(guide, elapsed));
    let gust_packet = flow
        .gust_packet_strength
        .max(flow.layered_gust_strength)
        .max(travelling_gust_packet(elapsed, guide.phase, progress, flow.gust_strength) * 0.8);
    let phase = guide.phase + flow.variation * 1.7;
    let pulse = (elapsed * 1.16 + phase).sin();
    let core = (0.72
        + flow.gust_strength * 0.25
        + flow.variation * 0.24
        + gust_packet * 0.12
        + pulse * 0.1)
        .clamp(0.78, 1.42);
    let stretch = (0.72 + vertical_ratio * 0.32 + flow.variation * 0.2 + pulse.max(0.0) * 0.08)
        + gust_packet * 0.14;
    let stretch = stretch.clamp(0.82, 1.48);

    Vec3::new(core, stretch, core) * visibility
}

pub(crate) fn crosswind_guide_position(guide: &CrosswindGuide, elapsed: f32) -> Vec3 {
    let field = guide.field;
    let path_length = (field.half_extents.x * 2.0).max(1.0);
    let progress = crosswind_guide_progress(guide, elapsed);
    let lane_origin = field.stream_origin(guide.stream_index, guide.stream_count);
    let base = lane_origin + field.direction * (progress * path_length);
    let flow = field
        .flow_at(base, elapsed)
        .unwrap_or_else(|| field.flow_at(field.center, elapsed).unwrap());
    let lateral = wind_lateral_axis(field.direction);
    let phase = guide.phase * std::f32::consts::TAU;
    let flow_axis = horizontal_or(flow.vector, field.direction);
    let shear = (flow_axis - field.direction) * (field.half_extents.x * 0.025);
    let gust_front = ((elapsed * 0.94 + phase).sin())
        .max(0.0)
        .max(flow.gust_packet_strength * 0.35)
        .max(flow.layered_gust_strength * 0.26);
    let gust_packet = flow
        .gust_packet_strength
        .max(flow.layered_gust_strength)
        .max(travelling_gust_packet(elapsed, guide.phase, progress, flow.gust_strength) * 0.24);
    let lane_roll = (elapsed * 0.42 + phase * 1.17 + progress * std::f32::consts::TAU * 1.02).sin();
    let lift_roll = (elapsed * 0.36 + phase * 0.71 + progress * std::f32::consts::TAU * 1.16).cos();
    let depth_packet = travelling_gust_packet(
        elapsed,
        guide.phase + guide.stream_index as f32 * 0.061,
        progress,
        flow.gust_strength,
    );

    base + shear
        + flow_axis * (gust_front * flow.variation * 0.24 + gust_packet * flow.variation * 0.26)
        + lateral
            * ((elapsed * 0.62 + phase).sin() * 0.3
                + flow.variation * 0.22
                + gust_packet * 0.14
                + flow.layered_gust_strength * 0.12
                + lane_roll * (0.13 + flow.variation * 0.18)
                + depth_packet * 0.1)
        + Vec3::Y
            * ((elapsed * 0.74 + phase).cos() * 0.12
                + flow.variation * 0.08
                + gust_packet * 0.08
                + flow.layered_gust_strength * 0.08
                + lift_roll * (0.04 + flow.variation * 0.06)
                + depth_packet * 0.06)
}

fn crosswind_guide_rotation(guide: &CrosswindGuide, position: Vec3, elapsed: f32) -> Quat {
    let flow = guide
        .field
        .flow_at(position, elapsed)
        .unwrap_or_else(|| guide.field.flow_at(guide.field.center, elapsed).unwrap());
    let flow_axis = horizontal_or(flow.vector, guide.field.direction);
    let phase = guide.phase * std::f32::consts::TAU;
    let roll = elapsed * 0.78 + phase + flow.variation * 0.32;

    rotation_from_x_to_direction(flow_axis) * Quat::from_rotation_x(roll)
}

pub(crate) fn crosswind_guide_scale(guide: &CrosswindGuide, position: Vec3, elapsed: f32) -> Vec3 {
    let flow = guide
        .field
        .flow_at(position, elapsed)
        .unwrap_or_else(|| guide.field.flow_at(guide.field.center, elapsed).unwrap());
    let phase = guide.phase * std::f32::consts::TAU;
    let visibility = wind_loop_visibility(crosswind_guide_progress(guide, elapsed));
    let pulse = (elapsed * 0.92 + phase + flow.variation * 1.4).sin();
    let gust_packet = flow.gust_packet_strength.max(flow.layered_gust_strength);
    let length_pulse = (0.72
        + flow.gust_strength * 0.48
        + flow.variation * 0.34
        + pulse.max(0.0) * 0.14
        + gust_packet * 0.2
        + flow.layered_gust_strength * 0.1)
        .clamp(1.04, 1.88);
    let width_pulse =
        (0.66 + flow.variation * 0.58 - pulse * 0.07 + gust_packet * 0.12).clamp(0.78, 1.36);

    Vec3::new(length_pulse, width_pulse, width_pulse) * visibility
}

fn updraft_guide_progress(guide: &UpdraftGuide, elapsed: f32) -> f32 {
    let height_span = (guide.field.half_extents.y * 1.84).max(1.0);
    let base_progress = (guide.height_offset / height_span + 0.5).clamp(0.0, 1.0);
    let rise_speed = guide.field.visual_speed.max(1.0) / height_span * 0.38;
    (base_progress + guide.phase * 0.037 + elapsed * rise_speed).fract()
}

fn crosswind_guide_progress(guide: &CrosswindGuide, elapsed: f32) -> f32 {
    let path_length = (guide.field.half_extents.x * 2.0).max(1.0);
    let stream_variation = 0.86 + (guide.stream_index % 7) as f32 * 0.035;
    (guide.phase
        + elapsed * guide.field.visual_speed.max(1.0) / path_length * 0.46 * stream_variation)
        .fract()
}

fn wind_loop_visibility(progress: f32) -> f32 {
    let wrapped = progress.rem_euclid(1.0);
    let edge = wrapped.min(1.0 - wrapped);
    smooth_unit((edge / WIND_VISUAL_LOOP_FADE_FRACTION).clamp(0.0, 1.0))
}

fn smooth_unit(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

#[cfg(test)]
fn travelling_gust_front(elapsed: f32, phase: f32, speed_scale: f32) -> f32 {
    wind_gust_front_progress(elapsed, phase, speed_scale)
}

fn travelling_gust_packet(elapsed: f32, phase: f32, progress: f32, speed_scale: f32) -> f32 {
    wind_gust_packet_strength(elapsed, phase, progress, speed_scale)
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

    #[test]
    fn wind_visual_eval_clock_ignores_wall_clock_elapsed_time() {
        let elapsed = eval_wind_visual_elapsed_secs(99.0, Some((24, 1.0 / 60.0)));
        assert!((elapsed - 0.4).abs() < 0.0001);
        assert_eq!(eval_wind_visual_elapsed_secs(2.5, None), 2.5);
    }

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
    fn observed_updraft_frame_motion_requires_actual_transform_delta() {
        let guide = test_updraft_guide();
        let elapsed = 2.0;
        let previous = Transform::from_translation(updraft_guide_position(&guide, elapsed));
        let current =
            Transform::from_translation(updraft_guide_position(&guide, elapsed + 1.0 / 60.0));
        let mut moving_metrics = ObservedWindVisualMotionMetrics::default();
        observe_updraft_guide_frame_motion(
            &mut moving_metrics,
            &guide,
            &previous,
            &current,
            elapsed,
            1.0 / 60.0,
            None,
        );

        assert_eq!(
            moving_metrics.observed_updraft_flow_coherent_visual_count, 1,
            "expected observed in-field updraft motion to count as flow coherent"
        );
        assert!(moving_metrics.max_observed_updraft_visual_frame_motion_m > 0.02);
        assert!(moving_metrics.max_observed_updraft_visual_frame_rise_m > 0.005);
        assert!(moving_metrics.max_observed_updraft_visual_flow_alignment > 0.55);

        let mut frozen_metrics = ObservedWindVisualMotionMetrics::default();
        observe_updraft_guide_frame_motion(
            &mut frozen_metrics,
            &guide,
            &previous,
            &previous,
            elapsed,
            1.0 / 60.0,
            None,
        );
        assert_eq!(
            frozen_metrics.observed_updraft_flow_coherent_visual_count,
            0
        );
        assert_eq!(
            frozen_metrics.max_observed_updraft_visual_frame_motion_m,
            0.0
        );
    }

    #[test]
    fn observed_updraft_frame_motion_rejects_off_field_fallbacks() {
        let guide = test_updraft_guide();
        let outside = guide.field.center + Vec3::new(guide.field.half_extents.x + 4.0, 0.0, 0.0);
        let previous = Transform::from_translation(outside);
        let current = Transform::from_translation(outside + Vec3::Y * 0.2);
        let mut metrics = ObservedWindVisualMotionMetrics::default();

        observe_updraft_guide_frame_motion(
            &mut metrics,
            &guide,
            &previous,
            &current,
            1.0,
            1.0 / 60.0,
            None,
        );

        assert_eq!(metrics.observed_updraft_flow_coherent_visual_count, 0);
        assert_eq!(metrics.max_observed_updraft_visual_flow_alignment, 0.0);
        assert_eq!(metrics.max_observed_updraft_visual_frame_motion_m, 0.0);
    }

    #[test]
    fn updraft_guide_scale_pulses_with_sampled_flow() {
        let guide = test_updraft_guide();
        let start = updraft_guide_position(&guide, 0.0);
        let later = updraft_guide_position(&guide, 1.7);
        let start_scale = updraft_guide_scale(&guide, start, 0.0);
        let later_scale = updraft_guide_scale(&guide, later, 1.7);

        assert!(
            scale_delta(later_scale, start_scale) > 0.01,
            "expected updraft motes to visibly pulse with sampled flow, start={start_scale:?}, later={later_scale:?}"
        );
    }

    #[test]
    fn travelling_gust_packet_forms_moving_clumped_front() {
        let elapsed = 2.0;
        let phase = 0.27;
        let speed_scale = 9.0;
        let front = travelling_gust_front(elapsed, phase, speed_scale);
        let quiet_progress = (front + 0.35).fract();
        let later_elapsed = elapsed + 1.0;
        let later_front = travelling_gust_front(later_elapsed, phase, speed_scale);
        let advanced = (later_front - front).rem_euclid(1.0);

        assert!(
            travelling_gust_packet(elapsed, phase, front, speed_scale) > 0.98,
            "expected packet to peak at the traveling front"
        );
        assert!(
            travelling_gust_packet(elapsed, phase, quiet_progress, speed_scale) < 0.03,
            "expected packet to fade outside the clumped front"
        );
        assert!(
            advanced > 0.18,
            "expected gust front to advance through the stream, advanced={advanced}"
        );
        assert!(
            travelling_gust_packet(later_elapsed, phase, front, speed_scale) < 0.08,
            "expected the original front position to quiet after the packet travels"
        );
        assert!(
            travelling_gust_packet(later_elapsed, phase, later_front, speed_scale) > 0.98,
            "expected the advanced front to carry the packet peak"
        );
    }

    #[test]
    fn updraft_guide_scale_uses_clumped_gust_packet() {
        let guide = test_updraft_guide();
        let elapsed = 1.25;
        let height_span = (guide.field.half_extents.y * 1.84).max(1.0);
        let center_flow = guide.field.flow_at(guide.field.center, elapsed).unwrap();
        let front = center_flow.gust_front_progress;
        let peak_position = guide.center + Vec3::Y * ((front - 0.5) * height_span);
        let quiet_position =
            guide.center + Vec3::Y * (((front + 0.35).fract() - 0.5) * height_span);
        let peak_scale = updraft_guide_scale(&guide, peak_position, elapsed);
        let quiet_scale = updraft_guide_scale(&guide, quiet_position, elapsed);

        assert!(
            scale_delta(peak_scale, quiet_scale) > 0.07,
            "expected clumped gust packet to visibly pulse updraft mote scale, peak={peak_scale:?}, quiet={quiet_scale:?}"
        );
    }

    #[test]
    fn updraft_guide_scale_uses_layered_gust_energy_between_primary_fronts() {
        let guide = test_updraft_guide();
        let elapsed = 1.25;
        let height_span = (guide.field.half_extents.y * 1.84).max(1.0);
        let mut layered_position = None;
        let mut calm_position = None;

        for index in 0..=140 {
            let progress = index as f32 / 140.0;
            let position = guide.center + Vec3::Y * ((progress - 0.5) * height_span);
            let flow = guide.field.flow_at(position, elapsed).unwrap();

            if flow.gust_packet_strength < 0.12 && flow.layered_gust_strength > 0.25 {
                layered_position = Some(position);
            }
            if flow.gust_packet_strength < 0.08 && flow.layered_gust_strength < 0.04 {
                calm_position = Some(position);
            }
        }

        let layered_position = layered_position
            .expect("expected a secondary updraft gust layer between primary fronts");
        let calm_position = calm_position.expect("expected a calm gap between updraft gust layers");
        let layered_scale = updraft_guide_scale(&guide, layered_position, elapsed);
        let calm_scale = updraft_guide_scale(&guide, calm_position, elapsed);

        assert!(
            scale_delta(layered_scale, calm_scale) > 0.04,
            "expected layered gust energy to visibly pulse updraft mote scale, layered={layered_scale:?}, calm={calm_scale:?}"
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
        assert!(
            metrics.max_updraft_visual_scale_pulse > 0.01,
            "expected updraft scale pulse above readability floor, metrics={metrics:?}"
        );
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
        assert_eq!(
            metrics.crosswind_flow_coherent_visual_count, 1,
            "expected crosswind guide to stay flow coherent, metrics={metrics:?}"
        );
        assert!(metrics.max_crosswind_visual_flow_alignment > 0.55);
    }

    #[test]
    fn observed_crosswind_ribbon_motion_uses_real_scene_sample_delta() {
        let field = WindField::crosswind(
            Vec3::ZERO,
            Vec3::new(18.0, 8.0, 8.0),
            Vec3::new(-1.0, 0.0, 0.35),
            10.0,
        );
        let ribbon = CrosswindRibbon {
            field,
            base_translation: field.center,
            phase: 0.18,
        };
        let elapsed = 2.0;
        let previous = crosswind_ribbon_transform(&ribbon, elapsed);
        let current = crosswind_ribbon_transform(&ribbon, elapsed + 1.0 / 60.0);
        let mut metrics = ObservedWindVisualMotionMetrics::default();

        observe_crosswind_ribbon_frame_motion(
            &mut metrics,
            &ribbon,
            &previous,
            &current,
            elapsed,
            1.0 / 60.0,
            None,
        );

        assert_eq!(metrics.observed_crosswind_flow_coherent_visual_count, 1);
        assert!(
            metrics.observed_crosswind_ribbon_flow_coherent_sample_count >= 2,
            "expected ribbon scene samples to move coherently with the field"
        );
        assert!(metrics.max_observed_crosswind_visual_frame_motion_m > 0.02);
        assert!(metrics.max_observed_crosswind_ribbon_frame_flow_displacement_m > 0.01);
        assert!(metrics.max_observed_crosswind_visual_flow_alignment > 0.55);
        assert!(metrics.max_observed_crosswind_ribbon_visual_flow_alignment > 0.55);
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
        let lateral = wind_lateral_axis(field.direction);
        let leading_depth = Vec2::new(leading_displacement.dot(lateral), leading_displacement.y);
        let trailing_depth = Vec2::new(trailing_displacement.dot(lateral), trailing_displacement.y);

        assert!(
            leading_displacement.dot(field.direction) > 1.0
                && trailing_displacement.dot(field.direction) > 1.0,
            "expected both streams to keep moving downwind, leading={leading_displacement:?}, trailing={trailing_displacement:?}"
        );
        assert!(
            (leading_displacement.length() - trailing_displacement.length()).abs() > 0.3,
            "expected stream-specific motion variation, leading={leading_displacement:?}, trailing={trailing_displacement:?}"
        );
        assert!(
            leading_depth.distance(trailing_depth) > 0.25,
            "expected phase-staggered crosswind streams to move through different depth lanes, leading={leading_depth:?}, trailing={trailing_depth:?}"
        );
    }

    #[test]
    fn crosswind_guide_scale_uses_shared_gust_packet() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::new(18.0, 8.0, 10.0), Vec3::X, 14.0);
        let guide = CrosswindGuide {
            field,
            stream_index: 6,
            stream_count: 16,
            phase: 0.31,
        };
        let elapsed = 1.25;
        let lane_origin = field.stream_origin(guide.stream_index, guide.stream_count);
        let path_length = field.half_extents.x * 2.0;
        let mut peak_position = None;
        let mut calm_position = None;

        for index in 0..=140 {
            let progress = index as f32 / 140.0;
            let position = lane_origin + field.direction * (progress * path_length);
            let flow = field.flow_at(position, elapsed).unwrap();
            let packet = flow.gust_packet_strength.max(flow.layered_gust_strength);

            if packet > 0.9 && peak_position.is_none() {
                peak_position = Some(position);
            }
            if packet < 0.05 && calm_position.is_none() {
                calm_position = Some(position);
            }
            if peak_position.is_some() && calm_position.is_some() {
                break;
            }
        }

        let peak_position = peak_position.expect("expected a crosswind gust-packet peak");
        let calm_position = calm_position.expect("expected a calm crosswind lane gap");
        let peak_scale = crosswind_guide_scale(&guide, peak_position, elapsed);
        let calm_scale = crosswind_guide_scale(&guide, calm_position, elapsed);

        assert!(
            scale_delta(peak_scale, calm_scale) > 0.08,
            "expected shared gust packet to visibly pulse crosswind guide scale, peak={peak_scale:?}, calm={calm_scale:?}"
        );
    }

    #[test]
    fn crosswind_guide_rotation_faces_sampled_flow_direction() {
        let field = WindField::crosswind(
            Vec3::ZERO,
            Vec3::new(22.0, 9.0, 14.0),
            Vec3::new(1.0, 0.0, 0.2),
            18.0,
        );
        let guide = CrosswindGuide {
            field,
            stream_index: 13,
            stream_count: 60,
            phase: 0.42,
        };
        let elapsed = 1.35;
        let position = crosswind_guide_position(&guide, elapsed);
        let flow = field
            .flow_at(position, elapsed)
            .or_else(|| field.flow_at(field.center, elapsed))
            .unwrap();
        let expected_axis = horizontal_or(flow.vector, field.direction);
        let local_x = crosswind_guide_rotation(&guide, position, elapsed) * Vec3::X;

        assert!(
            local_x.dot(expected_axis) > 0.999,
            "expected guide to face sampled gust/shear flow, local_x={local_x:?}, expected_axis={expected_axis:?}"
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
            phase: 0.18,
        };
        let elapsed = 0.6;
        let transform = crosswind_ribbon_transform(&ribbon, elapsed);
        let next_translation =
            crosswind_ribbon_transform(&ribbon, elapsed + WIND_VISUAL_COHERENCE_DT).translation;
        let short_horizon_motion = next_translation - transform.translation;
        let sampled_flow = field
            .flow_at(transform.translation, elapsed)
            .or_else(|| field.flow_at(field.center, elapsed))
            .unwrap();
        let metrics = wind_guide_visual_metrics(
            elapsed,
            std::iter::empty::<(&UpdraftGuide, &Transform)>(),
            std::iter::empty::<(&UpdraftRibbon, &Transform)>(),
            std::iter::empty::<(&CrosswindGuide, &Transform)>(),
            [(&ribbon, &transform)].into_iter(),
        );

        assert_eq!(metrics.crosswind_ribbon_count, 1);
        assert!(metrics.max_crosswind_visual_motion_m > 1.0);
        assert!(metrics.max_crosswind_ribbon_flow_displacement_m > 1.0);
        assert_eq!(
            metrics.crosswind_flow_coherent_visual_count, 1,
            "expected crosswind ribbon to stay flow coherent, metrics={metrics:?}, short_horizon_motion={short_horizon_motion:?}, sampled_flow={sampled_flow:?}"
        );
        assert!(metrics.max_crosswind_visual_flow_alignment > 0.55);
        assert!(
            metrics.crosswind_ribbon_flow_coherent_sample_count >= 2,
            "expected ribbon front/mid/tail scene samples to track local flow, metrics={metrics:?}"
        );
        assert!(metrics.max_crosswind_ribbon_visual_flow_alignment > 0.55);
    }

    #[test]
    fn crosswind_ribbon_advected_flow_samples_must_stay_inside_field() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0), Vec3::X, 10.0);
        let current_samples = [
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(6.0, 0.0, 0.0),
        ];
        let next_samples = current_samples.map(|position| position + Vec3::X * 0.25);
        let mut metrics = WindGuideVisualMetrics::default();

        let coherent = record_crosswind_ribbon_advected_flow_coherence(
            &mut metrics,
            field,
            current_samples,
            next_samples,
            1.0,
        );

        assert!(
            !coherent,
            "off-field ribbon samples must not pass via field-center fallback"
        );
        assert_eq!(metrics.crosswind_ribbon_flow_coherent_sample_count, 0);
        assert_eq!(metrics.max_crosswind_ribbon_visual_flow_alignment, 0.0);
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
            scale_delta(later.scale, start.scale) > 0.04,
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
            displacement.y.abs() > 0.85,
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
    fn updraft_ribbon_samples_visible_thermal_lane_instead_of_center_only() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(12.0, 30.0, 10.0), 18.0);
        let mut max_layer_delta = 0.0_f32;
        let mut strongest_sample = None;

        for phase in [0.04, 0.18, 0.33, 0.52, 0.71, 0.89] {
            let ribbon = UpdraftRibbon {
                field,
                spin_speed: 0.05,
                base_translation: Vec3::ZERO,
                base_rotation: Quat::IDENTITY,
                phase,
            };

            for elapsed in [0.4, 1.15, 1.9, 2.65] {
                let center_flow = field.flow_at(ribbon.base_translation, elapsed).unwrap();
                let sample = updraft_ribbon_flow_sample(&ribbon, elapsed);
                assert!(field.contains(sample.probe_position));

                let layer_delta = (sample.flow.gust_packet_strength
                    - center_flow.gust_packet_strength)
                    .abs()
                    + (sample.flow.layered_gust_strength - center_flow.layered_gust_strength).abs()
                    + (sample.flow.variation - center_flow.variation).abs();
                if layer_delta > max_layer_delta {
                    max_layer_delta = layer_delta;
                    strongest_sample = Some((sample, center_flow));
                }
            }
        }

        let (sample, center_flow) =
            strongest_sample.expect("expected at least one ribbon flow sample");
        assert!(
            max_layer_delta > 0.16,
            "expected updraft ribbons to sample distinct visible thermal layers, center={center_flow:?}, sampled={:?}, probe={:?}",
            sample.flow,
            sample.probe_position
        );
    }

    #[test]
    fn updraft_ribbons_have_phase_staggered_thermal_depth() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(10.0, 18.0, 10.0), 14.0);
        let lower_phase = UpdraftRibbon {
            field,
            spin_speed: 0.06,
            base_translation: Vec3::ZERO,
            base_rotation: Quat::IDENTITY,
            phase: 0.08,
        };
        let upper_phase = UpdraftRibbon {
            phase: 0.58,
            ..lower_phase
        };
        let elapsed = 1.4;
        let lower_delta = updraft_ribbon_transform(&lower_phase, elapsed).translation
            - updraft_ribbon_transform(&lower_phase, 0.0).translation;
        let upper_delta = updraft_ribbon_transform(&upper_phase, elapsed).translation
            - updraft_ribbon_transform(&upper_phase, 0.0).translation;
        let horizontal_separation =
            Vec2::new(lower_delta.x - upper_delta.x, lower_delta.z - upper_delta.z).length();

        assert!(
            lower_delta.y.abs() > 0.6 && upper_delta.y.abs() > 0.6,
            "expected both thermal ribbons to keep scrolling vertically, lower={lower_delta:?}, upper={upper_delta:?}"
        );
        assert!(
            horizontal_separation > 0.3,
            "expected phase-staggered ribbons to occupy different thermal depth lanes, lower={lower_delta:?}, upper={upper_delta:?}"
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
