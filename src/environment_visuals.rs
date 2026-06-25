use crate::Player;
use crate::authored_assets::VisualAssetRegistry;
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::generated_content::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, cloud_cluster_mesh, mesh_y_range, mix_color,
    updraft_ribbon_mesh,
};
use bevy::camera::{CameraOutputMode, ClearColorConfig, Exposure};
use bevy::light::VolumetricFog;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use nau_engine::animation::{Side, wing_airflow_strength};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::environment::{LiftRouteNode, wind_sway_motion};
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::SkyIsland;

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
    center: Vec3,
    radius: f32,
    height_offset: f32,
    phase: f32,
    angular_speed: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct UpdraftRibbon {
    spin_speed: f32,
    base_rotation: Quat,
}

pub(crate) fn spawn_updraft_guide(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    column_material: Handle<StandardMaterial>,
    ribbon_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    lift: LiftRouteNode,
) {
    let radius = lift.half_extents.x.min(lift.half_extents.z);
    let height = lift.half_extents.y * 2.0;
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(radius * 0.34, height))),
        MeshMaterial3d(column_material),
        Transform::from_translation(lift.center),
        Name::new(format!("{} atmospheric lift haze", lift.name)),
    ));

    for ribbon_index in 0..3 {
        let phase = ribbon_index as f32 / 3.0 * std::f32::consts::TAU;
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
                spin_speed: 0.035 + ribbon_index as f32 * 0.012,
                base_rotation,
            },
            Name::new(format!("{} spiral airflow ribbon", lift.name)),
        ));
    }

    let marker_mesh = meshes.add(Sphere::new(0.32));
    let ring_radius = radius * 0.5;
    let ring_levels = [-0.78, -0.34, 0.1, 0.54, 0.9];
    let markers_per_ring = 7;

    for (level_index, level) in ring_levels.into_iter().enumerate() {
        for marker_index in 0..markers_per_ring {
            let phase = marker_index as f32 / markers_per_ring as f32 * std::f32::consts::TAU
                + level_index as f32 * 0.46;
            let guide = UpdraftGuide {
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
        transform.rotation =
            ribbon.base_rotation * Quat::from_rotation_y(elapsed * ribbon.spin_speed);
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

fn updraft_guide_position(guide: &UpdraftGuide, elapsed: f32) -> Vec3 {
    let angle = guide.phase + elapsed * guide.angular_speed;
    guide.center
        + Vec3::new(
            angle.cos() * guide.radius,
            guide.height_offset + (elapsed * 1.4 + guide.phase).sin() * 0.35,
            angle.sin() * guide.radius,
        )
}
