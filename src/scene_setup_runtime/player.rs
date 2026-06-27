use bevy::prelude::*;

use crate::Player;
use crate::authored_assets::{
    AuthoredVisualScene, AuthoredVisualSceneRole, GeneratedPlayerPlaceholder,
    mark_authored_scene_ready,
};
use crate::environment_visuals::GliderAirflowTrail;
use crate::generated_content::glider_airflow_trail_mesh;
use crate::player_runtime::{
    AuthoredGliderPose, authored_glider_scene_transform, authored_player_scene_transform,
};
use crate::scene_setup_runtime::constants::PLAYER_START;
use crate::scene_setup_runtime::materials::SceneMaterials;
use nau_engine::animation::{AnimationState, CharacterPart, CharacterPartRole, Side};
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
) -> PlayerSceneEntities {
    let torso_mesh = meshes.add(Capsule3d::new(0.4, 1.0));
    let head_mesh = meshes.add(Sphere::new(0.3));
    let arm_mesh = meshes.add(Cuboid::new(0.2, 0.82, 0.2));
    let leg_mesh = meshes.add(Cuboid::new(0.24, 0.9, 0.24));
    let wing_mesh = meshes.add(Cuboid::new(2.15, 0.05, 0.75));
    let glider_airflow_mesh = meshes.add(glider_airflow_trail_mesh());
    let mut player_scene_entity = None;
    let mut glider_scene_entity = None;

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
                    glider_airflow_mesh.clone(),
                    side,
                );
            }

            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.18, 0.18, 0.38))),
                MeshMaterial3d(scene_materials.accent.clone()),
                Transform::from_xyz(0.0, 1.15, -0.28),
                Visibility::Inherited,
                GeneratedPlayerPlaceholder,
            ));
        });

    PlayerSceneEntities {
        player_scene_entity,
        glider_scene_entity,
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_player_side(
    parent: &mut ChildSpawnerCommands,
    scene_materials: &SceneMaterials,
    arm_mesh: Handle<Mesh>,
    leg_mesh: Handle<Mesh>,
    wing_mesh: Handle<Mesh>,
    glider_airflow_mesh: Handle<Mesh>,
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

    let trail_translation = Vec3::new(sign * 1.74, 1.38, 0.86);
    let trail_rotation = Quat::from_rotation_z(sign * 0.08) * Quat::from_rotation_x(0.04);
    parent.spawn((
        Mesh3d(glider_airflow_mesh),
        MeshMaterial3d(scene_materials.glider_airflow.clone()),
        Transform {
            translation: trail_translation,
            rotation: trail_rotation,
            scale: Vec3::new(0.35, 1.0, 0.05),
        },
        Visibility::Hidden,
        GliderAirflowTrail {
            side,
            base_translation: trail_translation,
            base_rotation: trail_rotation,
        },
    ));
}
