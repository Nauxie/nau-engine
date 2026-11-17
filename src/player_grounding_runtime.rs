use crate::authored_assets::{AuthoredVisualScene, AuthoredVisualSceneRole};
use crate::player_runtime::{AuthoredGliderPose, Player, authored_player_scene_transform};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy::transform::helper::TransformHelper;
use nau_engine::movement::{FlightController, FlightMode, smoothing_factor};
use nau_engine::world::{SkyRoute, TERRAIN_VISUAL_FOOTING_OFFSET_M};

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct GroundedFootContactDiagnostics {
    pub(crate) ready: bool,
    pub(crate) sole_vertex_count: usize,
    pub(crate) support_gap_m: f32,
}

#[derive(Default)]
pub(crate) struct GroundedVisualState {
    soles: Vec<SoleMesh>,
    offset_m: f32,
}

struct SoleMesh {
    entity: Entity,
    vertices: Vec<Vec3>,
}

/// Ground the visible soles, leaving the movement/collision root authoritative.
/// Both the character and its glider receive the same visual-frame correction.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn align_authored_player_grounding(
    time: Res<Time>,
    route: Res<SkyRoute>,
    meshes: Res<Assets<Mesh>>,
    player: Query<(&Transform, &FlightController), With<Player>>,
    names: Query<(Entity, &Name)>,
    children: Query<&Children>,
    mesh_handles: Query<&Mesh3d>,
    scenes: Query<(Entity, &AuthoredVisualScene)>,
    mut gliders: Query<&mut AuthoredGliderPose>,
    mut transforms: ParamSet<(TransformHelper, Query<&mut Transform, Without<Player>>)>,
    mut diagnostics: ResMut<GroundedFootContactDiagnostics>,
    mut state: Local<GroundedVisualState>,
) {
    let Ok((player_transform, controller)) = player.single() else {
        return;
    };
    if state.soles.len() != 2 {
        state.soles.clear();
        for (entity, name) in &names {
            if !matches!(
                name.as_str(),
                "Nau Left Leather Boot Sole" | "Nau Right Leather Boot Sole"
            ) {
                continue;
            }
            for mesh_entity in std::iter::once(entity).chain(children.iter_descendants(entity)) {
                let Ok(handle) = mesh_handles.get(mesh_entity) else {
                    continue;
                };
                let Some(mesh) = meshes.get(&handle.0) else {
                    continue;
                };
                let Some(VertexAttributeValues::Float32x3(positions)) =
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    continue;
                };
                state.soles.push(SoleMesh {
                    entity: mesh_entity,
                    vertices: positions.iter().copied().map(Vec3::from_array).collect(),
                });
            }
        }
    }
    if state.soles.len() != 2 {
        diagnostics.ready = false;
        return;
    }

    let grounded = controller.mode == FlightMode::Grounded;
    let (min_clearance_m, vertex_count) = if grounded {
        sole_clearance(
            &state.soles,
            &transforms.p0(),
            &route,
            player_transform.translation,
        )
    } else {
        (
            0.0,
            state.soles.iter().map(|sole| sole.vertices.len()).sum(),
        )
    };
    if !min_clearance_m.is_finite() || vertex_count == 0 {
        diagnostics.ready = false;
        return;
    }
    if controller.mode == FlightMode::Grounded {
        state.offset_m -= min_clearance_m;
    } else {
        state.offset_m *= 1.0 - smoothing_factor(12.0, time.delta_secs());
    }
    for (entity, scene) in &scenes {
        if scene.role == AuthoredVisualSceneRole::PlayerRuntime
            && let Ok(mut transform) = transforms.p1().get_mut(entity)
        {
            transform.translation.y =
                authored_player_scene_transform().translation.y + state.offset_m;
        }
    }
    for mut glider in &mut gliders {
        glider.grounding_offset_y = state.offset_m;
    }
    // Reread the corrected visible hierarchy; do not report the requested
    // correction as if it were an independently observed contact.
    let (final_clearance_m, final_vertex_count) = if grounded {
        sole_clearance(
            &state.soles,
            &transforms.p0(),
            &route,
            player_transform.translation,
        )
    } else {
        (0.0, vertex_count)
    };
    *diagnostics = GroundedFootContactDiagnostics {
        ready: true,
        sole_vertex_count: final_vertex_count,
        support_gap_m: if controller.mode == FlightMode::Grounded {
            final_clearance_m
        } else {
            0.0
        },
    };
}

fn sole_clearance(
    soles: &[SoleMesh],
    helper: &TransformHelper,
    route: &SkyRoute,
    player_position: Vec3,
) -> (f32, usize) {
    let supporting_surface = route.ground_at(player_position);
    let mut min_clearance_m = f32::INFINITY;
    let mut vertex_count = 0;
    {
        for sole in soles {
            let Ok(global) = helper.compute_global_transform(sole.entity) else {
                continue;
            };
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            for vertex in &sole.vertices {
                let world = global.transform_point(*vertex);
                min = min.min(world);
                max = max.max(world);
                vertex_count += 1;
            }
            // The highest terrain under this small sole footprint bounds the
            // clearance on slopes without sampling the terrain at every vertex.
            let mut floor_y = f32::NEG_INFINITY;
            for (x, z) in [
                (min.x, min.z),
                (min.x, max.z),
                (max.x, min.z),
                (max.x, max.z),
                ((min.x + max.x) * 0.5, (min.z + max.z) * 0.5),
            ] {
                let sample = route.ground_at(Vec3::new(x, player_position.y, z));
                // An animation can extend both feet over a cliff while the
                // collision root still has island support. Keep that support
                // plane instead of pulling the visible body to the world floor.
                let supported_floor_y = if sample.island_name == supporting_surface.island_name {
                    sample.floor_y
                } else {
                    supporting_surface.floor_y
                };
                floor_y = floor_y.max(supported_floor_y - TERRAIN_VISUAL_FOOTING_OFFSET_M);
            }
            min_clearance_m = min_clearance_m.min(min.y - floor_y);
        }
    }
    (min_clearance_m, vertex_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authored_assets::authored_player_pose_node_for_name;
    use crate::player_runtime::{
        authored_glider_scene_transform, reapply_authored_glider_pose,
        reapply_authored_player_pose_nodes,
    };
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use bevy::asset::RenderAssetUsages;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::mesh::PrimitiveTopology;
    use nau_engine::animation::{CharacterPart, PlayerPoseContext, part_pose_with_context};
    use nau_engine::asset_pipeline::VisualAssetKind;
    use nau_engine::movement::FlightInput;
    use nau_engine::world::PLAYER_COLLISION_HEIGHT_M;
    use serde_json::Value;
    use std::time::Duration;

    fn vector(value: Option<&Value>, fallback: Vec3) -> Vec3 {
        value.and_then(Value::as_array).map_or(fallback, |a| {
            Vec3::new(
                a[0].as_f64().unwrap() as f32,
                a[1].as_f64().unwrap() as f32,
                a[2].as_f64().unwrap() as f32,
            )
        })
    }

    fn fixture_world(
        position: Vec3,
    ) -> (World, Entity, Entity, Entity, Vec<(Entity, CharacterPart)>) {
        let gltf: Value =
            serde_json::from_str(include_str!("../assets/models/player/player.gltf")).unwrap();
        let uri = gltf["buffers"][0]["uri"].as_str().unwrap();
        let bytes = STANDARD.decode(uri.split_once(',').unwrap().1).unwrap();
        let mut world = World::new();
        world.insert_resource(SkyRoute::default());
        world.insert_resource(GroundedFootContactDiagnostics::default());
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(1.0 / 60.0));
        world.insert_resource(time);
        let player = world
            .spawn((
                Player,
                Transform::from_translation(position),
                FlightController::default(),
            ))
            .id();
        let scene = world
            .spawn((
                authored_player_scene_transform(),
                ChildOf(player),
                AuthoredVisualScene {
                    kind: VisualAssetKind::PlayerCharacter,
                    role: AuthoredVisualSceneRole::PlayerRuntime,
                },
            ))
            .id();
        let glider_transform = authored_glider_scene_transform();
        let glider = world
            .spawn((
                glider_transform,
                ChildOf(player),
                AuthoredGliderPose::new(&glider_transform),
            ))
            .id();
        let mut meshes = Assets::<Mesh>::default();
        let mut entities = Vec::new();
        let mut parts = Vec::new();
        for node in gltf["nodes"].as_array().unwrap() {
            let rotation = node["rotation"].as_array().map_or(Quat::IDENTITY, |a| {
                Quat::from_xyzw(
                    a[0].as_f64().unwrap() as f32,
                    a[1].as_f64().unwrap() as f32,
                    a[2].as_f64().unwrap() as f32,
                    a[3].as_f64().unwrap() as f32,
                )
            });
            let transform = Transform {
                translation: vector(node.get("translation"), Vec3::ZERO),
                rotation,
                scale: vector(node.get("scale"), Vec3::ONE),
            };
            let name = node["name"].as_str().unwrap();
            let entity = world.spawn((Name::new(name.to_owned()), transform)).id();
            if let Some(mut pose) = authored_player_pose_node_for_name(name) {
                pose.capture_rest_transform(&transform);
                parts.push((entity, pose.part));
                world.entity_mut(entity).insert(pose);
            }
            if let Some(index) = node["mesh"].as_u64() {
                let accessor_index =
                    gltf["meshes"][index as usize]["primitives"][0]["attributes"]["POSITION"]
                        .as_u64()
                        .unwrap() as usize;
                let accessor = &gltf["accessors"][accessor_index];
                let view = &gltf["bufferViews"][accessor["bufferView"].as_u64().unwrap() as usize];
                let offset = view["byteOffset"].as_u64().unwrap_or(0) as usize
                    + accessor["byteOffset"].as_u64().unwrap_or(0) as usize;
                let stride = view["byteStride"].as_u64().unwrap_or(12) as usize;
                let positions = (0..accessor["count"].as_u64().unwrap() as usize)
                    .map(|i| {
                        std::array::from_fn(|axis| {
                            f32::from_le_bytes(
                                bytes[offset + i * stride + axis * 4
                                    ..offset + i * stride + axis * 4 + 4]
                                    .try_into()
                                    .unwrap(),
                            )
                        })
                    })
                    .collect::<Vec<[f32; 3]>>();
                let mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                world.entity_mut(entity).insert(Mesh3d(meshes.add(mesh)));
            }
            entities.push(entity);
        }
        for (index, node) in gltf["nodes"].as_array().unwrap().iter().enumerate() {
            if let Some(children) = node["children"].as_array() {
                for child in children {
                    world
                        .entity_mut(entities[child.as_u64().unwrap() as usize])
                        .insert(ChildOf(entities[index]));
                }
            }
        }
        world.entity_mut(entities[0]).insert(ChildOf(scene));
        world.insert_resource(meshes);
        (world, player, scene, glider, parts)
    }

    fn bounds(
        helper: TransformHelper,
        meshes: Res<Assets<Mesh>>,
        nodes: Query<(Entity, &Mesh3d)>,
    ) -> (Vec3, Vec3) {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for (entity, handle) in &nodes {
            let global = helper.compute_global_transform(entity).unwrap();
            let Some(VertexAttributeValues::Float32x3(vertices)) = meshes
                .get(&handle.0)
                .unwrap()
                .attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("positions")
            };
            for vertex in vertices {
                let point = global.transform_point(Vec3::from_array(*vertex));
                min = min.min(point);
                max = max.max(point);
            }
        }
        (min, max)
    }

    fn actual_sole_vertex_clearance(
        helper: TransformHelper,
        meshes: Res<Assets<Mesh>>,
        nodes: Query<(Entity, &Name, &Mesh3d)>,
        player: Query<&Transform, With<Player>>,
        route: Res<SkyRoute>,
    ) -> f32 {
        let player_y = player.single().unwrap().translation.y;
        let mut minimum = f32::INFINITY;
        for (entity, name, handle) in &nodes {
            if !name.as_str().ends_with("Leather Boot Sole") {
                continue;
            }
            let global = helper.compute_global_transform(entity).unwrap();
            let Some(VertexAttributeValues::Float32x3(vertices)) = meshes
                .get(&handle.0)
                .unwrap()
                .attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("sole vertices")
            };
            for vertex in vertices {
                let point = global.transform_point(Vec3::from_array(*vertex));
                let floor = route
                    .ground_at(Vec3::new(point.x, player_y, point.z))
                    .floor_y
                    - TERRAIN_VISUAL_FOOTING_OFFSET_M;
                minimum = minimum.min(point.y - floor);
            }
        }
        minimum
    }

    fn simulate_authored_clip_overwrite(
        mut parts: Query<&mut Transform, With<crate::authored_assets::AuthoredPlayerPoseNode>>,
    ) {
        for mut transform in &mut parts {
            transform.translation += Vec3::Y * 3.0;
            transform.rotation = Quat::from_rotation_x(1.2);
        }
    }

    #[test]
    fn actual_fixture_rest_bounds_fit_collision_height_and_floor() {
        let (mut world, _, _, _, _) = fixture_world(Vec3::ZERO);
        let (min, max) = world.run_system_once(bounds).unwrap();
        assert!(
            (min.y + TERRAIN_VISUAL_FOOTING_OFFSET_M).abs() < 0.0001,
            "actual sole minimum {min:?}"
        );
        assert!(
            (max.y - min.y - PLAYER_COLLISION_HEIGHT_M).abs() < 0.0001,
            "actual actor height {}",
            max.y - min.y
        );
    }

    #[test]
    fn landing_recovery_feet_over_edge_keep_authoritative_island_support() {
        let position = Vec3::new(71.642_81, 27.872_957, 0.0);
        let (mut world, player, scene, _, parts) = fixture_world(position);
        world.get_mut::<Transform>(player).unwrap().rotation = Quat::from_rotation_y(0.698_131_74);
        assert_eq!(
            world.resource::<SkyRoute>().ground_at(position).island_name,
            Some("launch mesa")
        );
        let context = PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -8.0),
            FlightInput::default(),
            0.0,
        )
        .with_landing_recovery(0.2, 14.0);
        for (entity, part) in &parts {
            let pose = part_pose_with_context(part, context, 1.0);
            let mut transform = world.get_mut::<Transform>(*entity).unwrap();
            transform.translation = pose.translation;
            transform.rotation = pose.rotation;
        }
        let mut schedule = Schedule::default();
        schedule.add_systems(align_authored_player_grounding);
        schedule.run(&mut world);
        let offset = world.get::<Transform>(scene).unwrap().translation.y
            - authored_player_scene_transform().translation.y;
        assert!(
            offset.abs() < 0.35,
            "off-edge soles must not drop the body to world floor: {offset}"
        );
        assert_eq!(
            world.get::<Transform>(player).unwrap().translation,
            position
        );
        let actual_world_floor_gap = world.run_system_once(actual_sole_vertex_clearance).unwrap();
        assert!(
            actual_world_floor_gap > 10.0,
            "repro requires hanging feet over distant world floor: {actual_world_floor_gap}"
        );
        let diagnostic = world.resource::<GroundedFootContactDiagnostics>();
        assert!(diagnostic.ready && diagnostic.support_gap_m.abs() < 0.0001);
    }

    #[test]
    fn actual_sole_meshes_stay_grounded_through_gait_recovery_and_launch() {
        for (x, z) in [(0.0, 0.0), (2500.0, -340.0)] {
            let route = SkyRoute::default();
            let mut position = Vec3::new(x, 0.0, z);
            position.y = route.ground_at(position).floor_y;
            let (mut world, player, scene, glider, parts) = fixture_world(position);
            let mut schedule = Schedule::default();
            schedule.add_systems(
                (
                    align_authored_player_grounding,
                    simulate_authored_clip_overwrite,
                    reapply_authored_player_pose_nodes,
                    reapply_authored_glider_pose,
                )
                    .chain(),
            );
            for (speed, recovery) in [(0.0, false), (4.5, false), (10.0, false), (8.0, true)] {
                for frame in 0..24 {
                    let phase = frame as f32 * std::f32::consts::TAU / 24.0;
                    let mut context = PlayerPoseContext::new(
                        FlightMode::Grounded,
                        Vec3::new(0.0, 0.0, -speed),
                        FlightInput::default(),
                        0.0,
                    );
                    if recovery {
                        context = context.with_landing_recovery(0.2, 14.0);
                    }
                    for (entity, part) in &parts {
                        let pose = part_pose_with_context(part, context, phase);
                        {
                            let mut transform = world.get_mut::<Transform>(*entity).unwrap();
                            transform.translation = pose.translation;
                            transform.rotation = pose.rotation;
                        }
                        let mut cached = world
                            .get_mut::<crate::authored_assets::AuthoredPlayerPoseNode>(*entity)
                            .unwrap();
                        cached.smoothed_translation = pose.translation;
                        cached.smoothed_rotation = pose.rotation;
                    }
                    schedule.run(&mut world);
                    let diagnostic = *world.resource::<GroundedFootContactDiagnostics>();
                    assert!(
                        diagnostic.ready && diagnostic.sole_vertex_count > 100,
                        "actual boot geometry required"
                    );
                    assert!(
                        diagnostic.support_gap_m.abs() < 0.0001,
                        "{diagnostic:?}, speed {speed}, phase {phase}"
                    );
                    let actual_clearance =
                        world.run_system_once(actual_sole_vertex_clearance).unwrap();
                    assert!(
                        (-0.003..0.035).contains(&actual_clearance),
                        "actual sole/terrain clearance {actual_clearance}, speed {speed}, phase {phase}"
                    );
                    assert_eq!(
                        world.get::<Transform>(player).unwrap().translation,
                        position,
                        "visual correction must not move physics"
                    );
                    let frame_offset = world.get::<Transform>(scene).unwrap().translation.y
                        - authored_player_scene_transform().translation.y;
                    assert!(
                        (world
                            .get::<AuthoredGliderPose>(glider)
                            .unwrap()
                            .grounding_offset_y
                            - frame_offset)
                            .abs()
                            < 0.0001
                    );
                }
            }
            let before = world.get::<Transform>(scene).unwrap().translation.y;
            world.get_mut::<FlightController>(player).unwrap().mode = FlightMode::Launching;
            schedule.run(&mut world);
            let after = world.get::<Transform>(scene).unwrap().translation.y;
            assert!(
                (after - before).abs() < 0.04,
                "launch must release visual contact continuously"
            );
        }
    }
}
