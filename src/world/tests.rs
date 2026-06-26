use super::*;
use crate::environment::GAMEPLAY_LIFT_ROUTE;
use crate::movement::{FlightController, FlightMode, FlightState};
use bevy::prelude::{Vec2, Vec3};

#[test]
fn route_reports_highest_island_surface_under_player() {
    let route = SkyRoute::default();
    let launch_surface = route.ground_at(START_POSITION);

    assert_eq!(launch_surface.floor_y, START_FLOOR_Y);
    assert_eq!(launch_surface.island_name, Some("launch mesa"));
}

#[test]
fn route_surface_follows_island_relief() {
    let route = SkyRoute::default();
    let island = route.islands()[0];
    let ridge_position = Vec3::new(
        island.center.x + island.half_extents.x * 0.46,
        START_FLOOR_Y,
        island.center.z + island.half_extents.y * 0.18,
    );
    let edge_position = Vec3::new(
        island.center.x + island.half_extents.x * 0.84,
        START_FLOOR_Y,
        island.center.z - island.half_extents.y * 0.22,
    );

    let center_surface = route.ground_at(island.center);
    let ridge_surface = route.ground_at(ridge_position);
    let edge_surface = route.ground_at(edge_position);

    assert_eq!(center_surface.floor_y, START_FLOOR_Y);
    assert_ne!(ridge_surface.floor_y, center_surface.floor_y);
    assert_ne!(edge_surface.floor_y, center_surface.floor_y);
    assert!(
        ridge_surface.floor_y <= island.floor_y() + TERRAIN_MAX_RISE_M
            && edge_surface.floor_y >= island.floor_y() - TERRAIN_MAX_DROP_M
    );
}

#[test]
fn target_distance_reaches_zero_near_landing_island_center() {
    let route = SkyRoute::default();
    let target = route.target_island().expect("target island exists");

    assert_eq!(route.target_distance(target.center), 0.0);
    assert!(route.target_distance(START_POSITION) > 200.0);
}

#[test]
fn route_can_track_named_recovery_branch_islands() {
    let route = SkyRoute::default();
    let branch = route
        .island_named("sunlit terrace")
        .expect("recovery branch exists");

    assert!(is_recovery_branch_island(branch.name));
    assert_eq!(
        route.target_distance_to(branch.center, Some(branch.name)),
        0.0
    );
    assert!(
        route.target_distance_to(START_POSITION, Some(branch.name))
            > route.target_distance(START_POSITION)
    );
}

#[test]
fn route_objectives_track_main_and_branch_targets() {
    let route = SkyRoute::default();
    let main = route.route_objectives(None);
    let branch = route.route_objectives(Some("sunlit terrace"));

    assert_eq!(main.len(), 2);
    assert_eq!(main[0].label, "near route updraft");
    assert_eq!(main[1].label, "landing garden");
    assert_eq!(branch.len(), 3);
    assert_eq!(branch[1].label, "distant recovery updraft");
    assert_eq!(branch[2].label, "sunlit terrace");
}

#[test]
fn route_objective_completion_tracks_flythrough_and_landing() {
    let route = SkyRoute::default();
    let objectives = route.route_objectives(None);
    let target = route.target_island().expect("target island exists");

    assert!(objectives[0].is_complete(&route, GAMEPLAY_LIFT_ROUTE[0].center, FlightMode::Gliding));
    assert!(!objectives[1].is_complete(&route, target.center + Vec3::Y * 8.0, FlightMode::Gliding));
    assert!(objectives[1].is_complete(&route, target.center, FlightMode::Grounded));
}

#[test]
fn route_has_archipelago_scale_and_distant_landmarks() {
    let route = SkyRoute::default();
    let farthest_z = route
        .islands()
        .iter()
        .map(|island| island.center.z)
        .fold(0.0_f32, f32::min);

    assert!(route.islands().len() >= 12);
    assert!(farthest_z < -800.0);
}

#[test]
fn route_uses_all_declared_terrain_archetypes() {
    let route = SkyRoute::default();
    let mut archetype_mask = 0_u32;

    for island in route.islands() {
        let declared_archetype = IslandTerrainArchetype::for_name(island.name)
            .expect("route island names must declare a terrain archetype");
        assert_eq!(island.terrain_archetype, declared_archetype);
        archetype_mask |= 1_u32 << island.terrain_archetype.index();
        assert!(!island.terrain_archetype.label().is_empty());
    }

    assert_eq!(
        archetype_mask.count_ones() as usize,
        IslandTerrainArchetype::COUNT
    );
}

#[test]
fn island_horizontal_containment_follows_playable_silhouette() {
    let island = SkyIsland::new(
        "storm porch",
        Vec3::ZERO,
        Vec2::new(42.0, 28.0),
        15.0,
        false,
    );
    let mut smallest_scale = f32::INFINITY;
    let mut smallest_angle = 0.0;
    for step in 0..96 {
        let angle = step as f32 / 96.0 * std::f32::consts::TAU;
        let scale = island.playable_silhouette_scale(angle);
        if scale < smallest_scale {
            smallest_scale = scale;
            smallest_angle = angle;
        }
    }

    let inside = Vec3::new(
        smallest_angle.cos() * island.half_extents.x * (smallest_scale - 0.02),
        0.0,
        smallest_angle.sin() * island.half_extents.y * (smallest_scale - 0.02),
    );
    let outside = Vec3::new(
        smallest_angle.cos() * island.half_extents.x * (smallest_scale + 0.04),
        0.0,
        smallest_angle.sin() * island.half_extents.y * (smallest_scale + 0.04),
    );

    assert!(island.contains_horizontal(inside));
    assert!(!island.contains_horizontal(outside));
}

#[test]
fn streaming_lod_stats_track_active_window_and_distance_bands() {
    let route = SkyRoute::default();
    let stats = route.streaming_lod_stats(START_POSITION);

    assert_eq!(stats.player_chunk, StreamChunkCoord { x: 0, z: 0 });
    assert_eq!(stats.active_chunk_count, 25);
    assert!(stats.active_island_count < route.islands().len());
    assert!(stats.active_island_count >= 4);
    assert!(stats.near_lod_islands >= 2);
    assert!(stats.mid_lod_islands >= 3);
    assert!(stats.far_lod_islands >= 3);
}

#[test]
fn island_lod_band_uses_route_distance_thresholds() {
    let island = SkyIsland::new("test island", Vec3::ZERO, Vec2::new(10.0, 10.0), 4.0, false);

    assert_eq!(island.lod_band(Vec3::new(0.0, 0.0, 0.0)), LodBand::Near);
    assert_eq!(
        island.lod_band(Vec3::new(LOD_NEAR_DISTANCE_M + 1.0, 0.0, 0.0)),
        LodBand::Mid
    );
    assert_eq!(
        island.lod_band(Vec3::new(LOD_MID_DISTANCE_M + 1.0, 0.0, 0.0)),
        LodBand::Far
    );
}

#[test]
fn island_stream_activation_uses_chunk_window() {
    let island = SkyIsland::new("test island", Vec3::ZERO, Vec2::new(10.0, 10.0), 4.0, false);

    assert_eq!(
        island.stream_activation(START_POSITION),
        StreamActivation::Active
    );
    assert_eq!(
        island.stream_activation(Vec3::new(
            0.0,
            START_FLOOR_Y,
            STREAM_CHUNK_SIZE_M * (STREAM_ACTIVE_CHUNK_RADIUS + 2) as f32,
        )),
        StreamActivation::Inactive
    );
}

#[test]
fn ground_contact_marks_target_landing_as_grounded() {
    let route = SkyRoute::default();
    let target = route.target_island().expect("target island exists");
    let state = FlightState::new(
        Vec3::new(target.center.x, target.floor_y() - 2.0, target.center.z),
        Vec3::new(20.0, -10.0, 10.0),
        FlightController::default(),
    );

    let resolved = route.resolve_ground_contact(state);

    assert_eq!(resolved.position.y, target.floor_y());
    assert!(resolved.velocity.x < state.velocity.x);
    assert!(resolved.velocity.z < state.velocity.z);
    assert_eq!(resolved.controller.mode, FlightMode::Grounded);
    assert!(route.on_landing_target(resolved.position, resolved.controller.mode));
}

#[test]
fn already_grounded_route_contact_does_not_damp_wasd_motion() {
    let route = SkyRoute::default();
    let state = FlightState::new(
        START_POSITION,
        Vec3::new(8.0, 0.0, -4.0),
        FlightController::default(),
    );

    let resolved = route.resolve_ground_contact_after_step(state, true);

    assert_eq!(resolved.position.y, START_FLOOR_Y);
    assert_eq!(resolved.velocity.x, state.velocity.x);
    assert_eq!(resolved.velocity.z, state.velocity.z);
    assert_eq!(resolved.controller.mode, FlightMode::Grounded);
}

#[test]
fn route_landing_clears_stale_airborne_bank() {
    let route = SkyRoute::default();
    let state = FlightState::new(
        START_POSITION,
        Vec3::new(8.0, -1.0, -4.0),
        FlightController {
            mode: FlightMode::Gliding,
            bank_degrees: 18.0,
            launch_available: false,
            ..Default::default()
        },
    );

    let resolved = route.resolve_ground_contact_after_step(state, false);

    assert_eq!(resolved.controller.mode, FlightMode::Grounded);
    assert_eq!(resolved.controller.bank_degrees, 0.0);
}

#[test]
fn walking_off_island_clears_stale_grounded_mode() {
    let route = SkyRoute::default();
    let state = FlightState::new(
        Vec3::new(200.0, START_FLOOR_Y, 200.0),
        Vec3::new(6.0, 0.0, 0.0),
        FlightController::default(),
    );

    let resolved = route.resolve_ground_contact_after_step(state, true);

    assert_eq!(resolved.controller.mode, FlightMode::Airborne);
    assert_eq!(resolved.position.y, START_FLOOR_Y);
}

#[test]
fn island_visual_top_stays_close_to_player_footing() {
    let route = SkyRoute::default();
    let island = route.islands()[0];
    let sample = Vec3::new(
        island.center.x + island.half_extents.x * 0.35,
        island.center.y,
        island.center.z - island.half_extents.y * 0.25,
    );
    let visual_offset = island.terrain_surface_y_at(sample) - island.mesh_top_y_at(sample);

    assert!((0.15..=0.3).contains(&visual_offset));
}
