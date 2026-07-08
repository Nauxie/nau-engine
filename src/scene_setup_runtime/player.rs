use bevy::prelude::*;

use crate::Player;
use crate::authored_assets::{
    AuthoredVisualScene, AuthoredVisualSceneRole, GeneratedPlayerPlaceholder,
    mark_authored_scene_ready,
};
use crate::environment_visuals::{PlayerAirflowVisual, PlayerAirflowVisualKind};
use crate::generated_content::player_airflow_streamline_mesh;
use crate::player_runtime::{
    AuthoredGliderPose, authored_glider_scene_transform, authored_player_scene_transform,
};
use crate::scene_setup_runtime::materials::SceneMaterials;
use nau_engine::animation::{AnimationState, CharacterPart, CharacterPartRole, ScarfSegment, Side};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::movement::{FlightController, Velocity};

pub(super) struct PlayerSceneEntities {
    pub(super) player_scene_entity: Option<Entity>,
    pub(super) glider_scene_entity: Option<Entity>,
}

pub(super) fn spawn_player_runtime(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    scene_materials: &SceneMaterials,
    player_scene_handle: Option<Handle<Scene>>,
    glider_scene_handle: Option<Handle<Scene>>,
    player_start: Vec3,
) -> PlayerSceneEntities {
    let torso_mesh = meshes.add(Capsule3d::new(0.4, 1.0));
    let head_mesh = meshes.add(Sphere::new(0.3));
    let arm_mesh = meshes.add(Cuboid::new(0.2, 0.82, 0.2));
    let leg_mesh = meshes.add(Cuboid::new(0.24, 0.9, 0.24));
    let wing_mesh = meshes.add(Cuboid::new(2.15, 0.05, 0.75));
    let player_airflow_mesh = meshes.add(player_airflow_streamline_mesh());
    let mut player_scene_entity = None;
    let mut glider_scene_entity = None;

    commands
        .spawn((
            Transform::from_translation(player_start),
            Player,
            Velocity::default(),
            FlightController::default(),
            AnimationState::default(),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            if let Some(scene_handle) = player_scene_handle.clone() {
                let mut scene = parent.spawn((
                    SceneRoot(scene_handle),
                    authored_player_scene_transform(),
                    Visibility::Hidden,
                    AuthoredVisualScene {
                        kind: VisualAssetKind::PlayerCharacter,
                        role: AuthoredVisualSceneRole::PlayerRuntime,
                    },
                    Name::new("authored player character scene"),
                ));
                scene.observe(mark_authored_scene_ready);
                player_scene_entity = Some(scene.id());
            }

            if let Some(scene_handle) = glider_scene_handle.clone() {
                let glider_transform = authored_glider_scene_transform();
                let mut scene = parent.spawn((
                    SceneRoot(scene_handle),
                    glider_transform,
                    Visibility::Hidden,
                    AuthoredGliderPose::new(&glider_transform),
                    AuthoredVisualScene {
                        kind: VisualAssetKind::Glider,
                        role: AuthoredVisualSceneRole::GliderRuntime,
                    },
                    Name::new("authored glider scene"),
                ));
                scene.observe(mark_authored_scene_ready);
                glider_scene_entity = Some(scene.id());
            }

            parent.spawn((
                Mesh3d(torso_mesh.clone()),
                MeshMaterial3d(scene_materials.suit.clone()),
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
                MeshMaterial3d(scene_materials.skin.clone()),
                Transform::from_xyz(0.0, 1.78, 0.0),
                Visibility::Inherited,
                CharacterPart::new(
                    CharacterPartRole::Head,
                    Vec3::new(0.0, 1.78, 0.0),
                    Quat::IDENTITY,
                ),
            ));

            for side in [Side::Left, Side::Right] {
                spawn_player_side(
                    parent,
                    scene_materials,
                    arm_mesh.clone(),
                    leg_mesh.clone(),
                    wing_mesh.clone(),
                    side,
                );
            }

            spawn_player_airflow_volume(parent, scene_materials, player_airflow_mesh.clone());

            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.18, 0.18, 0.38))),
                MeshMaterial3d(scene_materials.accent.clone()),
                Transform::from_xyz(0.0, 1.15, -0.28),
                Visibility::Inherited,
                GeneratedPlayerPlaceholder,
                CharacterPart::new(
                    CharacterPartRole::Scarf(ScarfSegment::Trail),
                    Vec3::new(0.0, 1.15, -0.28),
                    Quat::IDENTITY,
                ),
            ));
        });

    PlayerSceneEntities {
        player_scene_entity,
        glider_scene_entity,
    }
}

fn spawn_player_side(
    parent: &mut ChildSpawnerCommands,
    scene_materials: &SceneMaterials,
    arm_mesh: Handle<Mesh>,
    leg_mesh: Handle<Mesh>,
    wing_mesh: Handle<Mesh>,
    side: Side,
) {
    let sign = side.sign();
    let arm_translation = Vec3::new(sign * 0.58, 1.05, 0.0);
    let arm_rotation = Quat::from_rotation_z(sign * 0.18);
    let leg_translation = Vec3::new(sign * 0.22, 0.28, 0.0);
    let leg_rotation = Quat::from_rotation_z(sign * 0.08);

    parent.spawn((
        Mesh3d(arm_mesh),
        MeshMaterial3d(scene_materials.suit.clone()),
        Transform {
            translation: arm_translation,
            rotation: arm_rotation,
            ..default()
        },
        Visibility::Inherited,
        CharacterPart::new(CharacterPartRole::Arm(side), arm_translation, arm_rotation),
    ));

    parent.spawn((
        Mesh3d(leg_mesh),
        MeshMaterial3d(scene_materials.suit.clone()),
        Transform {
            translation: leg_translation,
            rotation: leg_rotation,
            ..default()
        },
        Visibility::Inherited,
        CharacterPart::new(CharacterPartRole::Leg(side), leg_translation, leg_rotation),
    ));

    let wing_translation = Vec3::new(sign * 1.02, 1.45, -0.46);
    let wing_rotation = Quat::from_rotation_z(sign * 0.16) * Quat::from_rotation_x(-0.08);

    parent.spawn((
        Mesh3d(wing_mesh),
        MeshMaterial3d(scene_materials.glider.clone()),
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

#[derive(Clone, Copy, Debug)]
struct PlayerAirflowVisualSpec {
    kind: PlayerAirflowVisualKind,
    lane_index: usize,
    lane_count: usize,
    base_angle: f32,
    base_height: f32,
    base_radius: f32,
    seed: f32,
}

fn spawn_player_airflow_volume(
    parent: &mut ChildSpawnerCommands,
    scene_materials: &SceneMaterials,
    mesh: Handle<Mesh>,
) {
    for spec in player_airflow_visual_specs() {
        parent.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(scene_materials.glider_airflow.clone()),
            Transform::from_scale(Vec3::splat(0.01)),
            Visibility::Hidden,
            PlayerAirflowVisual::new(
                spec.kind,
                spec.lane_index,
                spec.lane_count,
                spec.base_angle,
                spec.base_height,
                spec.base_radius,
                spec.seed,
            ),
        ));
    }
}

fn player_airflow_visual_specs() -> Vec<PlayerAirflowVisualSpec> {
    let mut specs = Vec::with_capacity(18);
    let mut ordinal = 0_usize;

    for index in 0..6 {
        let lane_fraction = index as f32 / 6.0;
        specs.push(PlayerAirflowVisualSpec {
            kind: PlayerAirflowVisualKind::BodyWrap,
            lane_index: index,
            lane_count: 6,
            base_angle: lane_fraction * std::f32::consts::TAU,
            base_height: 0.72 + (index % 3) as f32 * 0.34,
            base_radius: 1.06 + (index % 3) as f32 * 0.11,
            seed: player_airflow_seed(ordinal),
        });
        ordinal += 1;
    }

    for index in 0..3 {
        specs.push(PlayerAirflowVisualSpec {
            kind: PlayerAirflowVisualKind::FrontPressure,
            lane_index: index,
            lane_count: 3,
            base_angle: -std::f32::consts::FRAC_PI_2 + (index as f32 - 1.0) * 0.46,
            base_height: 0.82 + index as f32 * 0.18,
            base_radius: 1.02,
            seed: player_airflow_seed(ordinal),
        });
        ordinal += 1;
    }

    for index in 0..2 {
        let side_angle = if index % 2 == 0 {
            0.0
        } else {
            std::f32::consts::PI
        };
        specs.push(PlayerAirflowVisualSpec {
            kind: PlayerAirflowVisualKind::SideShear,
            lane_index: index,
            lane_count: 2,
            base_angle: side_angle,
            base_height: 0.92,
            base_radius: 1.32,
            seed: player_airflow_seed(ordinal),
        });
        ordinal += 1;
    }

    for index in 0..2 {
        let side_angle = if index % 2 == 0 {
            0.0
        } else {
            std::f32::consts::PI
        };
        specs.push(PlayerAirflowVisualSpec {
            kind: PlayerAirflowVisualKind::ShoulderVortex,
            lane_index: index,
            lane_count: 2,
            base_angle: side_angle,
            base_height: 1.28,
            base_radius: 1.18,
            seed: player_airflow_seed(ordinal),
        });
        ordinal += 1;
    }

    for index in 0..2 {
        let side_angle = if index % 2 == 0 {
            0.0
        } else {
            std::f32::consts::PI
        };
        specs.push(PlayerAirflowVisualSpec {
            kind: PlayerAirflowVisualKind::WingtipVortex,
            lane_index: index,
            lane_count: 2,
            base_angle: side_angle,
            base_height: 1.38,
            base_radius: 1.72,
            seed: player_airflow_seed(ordinal),
        });
        ordinal += 1;
    }

    for index in 0..3 {
        specs.push(PlayerAirflowVisualSpec {
            kind: PlayerAirflowVisualKind::WakeTurbulence,
            lane_index: index,
            lane_count: 3,
            base_angle: std::f32::consts::FRAC_PI_2 + (index as f32 - 1.0) * 0.54,
            base_height: 0.82 + index as f32 * 0.22,
            base_radius: 1.18,
            seed: player_airflow_seed(ordinal),
        });
        ordinal += 1;
    }

    specs
}

fn player_airflow_seed(ordinal: usize) -> f32 {
    (0.137 + ordinal as f32 * 0.618_034).fract()
}
