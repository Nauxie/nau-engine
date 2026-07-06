use super::*;
use crate::environment::GAMEPLAY_LIFT_ROUTE;
use crate::movement::{FlightController, FlightMode, FlightState};
use bevy::prelude::{Vec2, Vec3};
use std::collections::{BTreeSet, VecDeque};

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
    let stratos = route.route_objectives(Some("stratos shelf"));
    let summit = route.route_objectives(Some("summit anvil"));
    let plateau = route.route_objectives(Some("great sky plateau"));

    assert_eq!(main.len(), 2);
    assert_eq!(main[0].label, "launch terrace updraft");
    assert_eq!(main[1].label, "landing garden");
    assert_eq!(branch.len(), 3);
    assert_eq!(branch[1].label, "distant recovery updraft");
    assert_eq!(branch[2].label, "sunlit terrace");
    assert_eq!(stratos.len(), 5);
    assert_eq!(stratos[2].label, "upper thermal ring updraft");
    assert_eq!(stratos[3].label, "stratos shelf updraft");
    assert_eq!(stratos[4].label, "stratos shelf");
    assert_eq!(summit.len(), 6);
    assert_eq!(summit[3].label, "stratos shelf updraft");
    assert_eq!(summit[4].label, "summit anvil updraft");
    assert_eq!(summit[5].label, "summit anvil");
    assert_eq!(plateau.len(), 10);
    assert_eq!(plateau[6].label, "bluevault basin updraft");
    assert_eq!(plateau[7].label, "sunspire garden updraft");
    assert_eq!(plateau[8].label, "great sky plateau updraft");
    assert_eq!(plateau[9].label, "great sky plateau");
    assert!(
        plateau
            .iter()
            .all(|objective| objective.label != "upper crown updraft")
    );
}

#[test]
fn route_objectives_keep_recovery_updrafts_as_non_gating_support() {
    let route = SkyRoute::default();
    let plateau = route.route_objectives(Some("great sky plateau"));
    let upper_crown = route.route_objectives(Some("upper crown"));
    let recovery_updrafts = [
        "low reef updraft",
        "western catch updraft",
        "skyhook basin updraft",
        "cloudfall meadow updraft",
        "underbridge cay updraft",
        "bluevault shoulder recovery updraft",
        "cloudbreak stair recovery updraft",
        "plateau west rim recovery updraft",
    ];

    for objectives in [&plateau, &upper_crown] {
        for recovery_name in recovery_updrafts {
            assert!(
                objectives
                    .iter()
                    .all(|objective| objective.label != recovery_name),
                "{recovery_name} should not become a required route objective"
            );
        }
    }
    assert_eq!(
        plateau.last().expect("plateau objective").label,
        "great sky plateau"
    );
    assert_eq!(
        upper_crown.last().expect("upper crown objective").label,
        "upper crown"
    );
    assert_eq!(upper_crown.len(), 11);
}

#[test]
fn first_expedition_route_beats_define_ordered_playable_journey() {
    let route = SkyRoute::default();
    let beats = route.first_expedition_route_beats();
    let expected_kinds = [
        FirstExpeditionBeatKind::Launch,
        FirstExpeditionBeatKind::FirstGlide,
        FirstExpeditionBeatKind::LowRecovery,
        FirstExpeditionBeatKind::UnderRoutePass,
        FirstExpeditionBeatKind::LakeWaterfallLandmark,
        FirstExpeditionBeatKind::HighClimb,
        FirstExpeditionBeatKind::PlateauApproach,
        FirstExpeditionBeatKind::PlateauArrival,
    ];
    let expected_modes = [
        FirstExpeditionTraversalMode::GroundLaunch,
        FirstExpeditionTraversalMode::OpenGlide,
        FirstExpeditionTraversalMode::RecoveryLift,
        FirstExpeditionTraversalMode::UnderIslandGlide,
        FirstExpeditionTraversalMode::LandmarkGlide,
        FirstExpeditionTraversalMode::SustainedClimb,
        FirstExpeditionTraversalMode::ApproachGlide,
        FirstExpeditionTraversalMode::ArrivalLanding,
    ];
    let expected_bands = [
        FirstExpeditionAltitudeBand::Low,
        FirstExpeditionAltitudeBand::Low,
        FirstExpeditionAltitudeBand::Low,
        FirstExpeditionAltitudeBand::Low,
        FirstExpeditionAltitudeBand::Mid,
        FirstExpeditionAltitudeBand::High,
        FirstExpeditionAltitudeBand::High,
        FirstExpeditionAltitudeBand::Plateau,
    ];

    assert_eq!(beats.len(), expected_kinds.len());
    for (index, beat) in beats.iter().enumerate() {
        assert_eq!(beat.kind, expected_kinds[index]);
        assert_eq!(beat.traversal_mode, expected_modes[index]);
        assert_eq!(beat.altitude_band, expected_bands[index]);
        assert!(!beat.label.is_empty());
        assert!(!beat.kind.label().is_empty());
        assert!(!beat.traversal_mode.label().is_empty());
        assert!(!beat.altitude_band.label().is_empty());
        assert!(
            beat.altitude_band >= beats[index.saturating_sub(1)].altitude_band,
            "first expedition route should not drop to a lower intended altitude band"
        );
    }
}

#[test]
fn first_expedition_route_beats_resolve_to_world_content_without_debug_guidance() {
    let route = SkyRoute::default();
    let beats = route.first_expedition_route_beats();

    assert_eq!(beats.len(), 8);
    for beat in beats {
        assert!(route.island_named(beat.anchor_island_name).is_some());
        assert_route_beat_text_is_not_debug_only(beat.label);
        assert_route_beat_text_is_not_debug_only(beat.landmark_anchor);
        assert!(!beat.landmark_anchor.is_empty());
        assert!(!beat.recovery_affordance.label().is_empty());
        if let Some(lift_name) = beat.recovery_affordance.lift_name() {
            assert!(
                GAMEPLAY_LIFT_ROUTE
                    .iter()
                    .any(|node| node.name == lift_name),
                "{lift_name} should exist as a readable recovery lift"
            );
            assert_route_beat_text_is_not_debug_only(lift_name);
        }
        if let Some(island_name) = beat.recovery_affordance.island_name() {
            assert!(route.island_named(island_name).is_some());
            assert_route_beat_text_is_not_debug_only(island_name);
        }

        match beat.kind {
            FirstExpeditionBeatKind::Launch
            | FirstExpeditionBeatKind::FirstGlide
            | FirstExpeditionBeatKind::LakeWaterfallLandmark
            | FirstExpeditionBeatKind::PlateauArrival => {
                assert_eq!(
                    route.ground_at(beat.position).island_name,
                    Some(beat.anchor_island_name)
                );
            }
            FirstExpeditionBeatKind::LowRecovery
            | FirstExpeditionBeatKind::HighClimb
            | FirstExpeditionBeatKind::PlateauApproach => {
                let lift_name = beat
                    .recovery_affordance
                    .lift_name()
                    .expect("lift beat should expose a recovery updraft");
                let lift = GAMEPLAY_LIFT_ROUTE
                    .iter()
                    .find(|node| node.name == lift_name)
                    .expect("lift exists")
                    .lift_field();
                assert!(lift.contains(beat.position));
            }
            FirstExpeditionBeatKind::UnderRoutePass => {
                let under_route = route
                    .island_named(beat.anchor_island_name)
                    .expect("under-route island exists")
                    .under_route_segment()
                    .expect("under-route segment exists");
                assert_eq!(beat.position, under_route.midpoint);
                assert!(!route.is_grounded_at(beat.position));
            }
        }
    }
}

#[test]
fn first_expedition_optional_detours_define_recovery_and_upper_side_routes() {
    let route = SkyRoute::default();
    let detours = route.first_expedition_optional_detours();

    assert_eq!(detours.len(), 2);
    assert_eq!(
        detours.iter().map(|detour| detour.kind).collect::<Vec<_>>(),
        vec![
            FirstExpeditionDetourKind::LowAltitudeRecoveryLoop,
            FirstExpeditionDetourKind::HighRiskUpperPath,
        ]
    );
    assert_eq!(
        detours.iter().map(|detour| detour.risk).collect::<Vec<_>>(),
        vec![
            FirstExpeditionDetourRisk::Recovery,
            FirstExpeditionDetourRisk::HighRiskHighReward,
        ]
    );

    let low_loop = detours
        .iter()
        .find(|detour| detour.kind == FirstExpeditionDetourKind::LowAltitudeRecoveryLoop)
        .expect("low recovery loop exists");
    assert_eq!(
        low_loop.entry_beat_kind,
        FirstExpeditionBeatKind::FirstGlide
    );
    assert_eq!(
        low_loop.reconnect_beat_kind,
        FirstExpeditionBeatKind::LowRecovery
    );
    assert_eq!(low_loop.altitude_band, FirstExpeditionAltitudeBand::Low);
    assert_eq!(
        low_loop.island_names,
        ["low reef", "lowwind shelf", "western refuge"]
    );
    assert_eq!(
        low_loop.lift_names,
        [
            "low reef updraft",
            "western catch updraft",
            "distant recovery updraft"
        ]
    );

    let upper_path = detours
        .iter()
        .find(|detour| detour.kind == FirstExpeditionDetourKind::HighRiskUpperPath)
        .expect("upper challenge path exists");
    assert_eq!(
        upper_path.entry_beat_kind,
        FirstExpeditionBeatKind::HighClimb
    );
    assert_eq!(
        upper_path.reconnect_beat_kind,
        FirstExpeditionBeatKind::PlateauArrival
    );
    assert_eq!(upper_path.altitude_band, FirstExpeditionAltitudeBand::High);
    assert_eq!(
        upper_path.island_names,
        ["bluevault basin", "outer switchback", "far horizon perch"]
    );
    assert_eq!(
        upper_path.lift_names,
        [
            "bluevault shoulder recovery updraft",
            "plateau west rim recovery updraft",
            "great sky plateau updraft"
        ]
    );

    for detour in detours {
        assert!(!detour.label.is_empty());
        assert!(!detour.kind.label().is_empty());
        assert!(!detour.risk.label().is_empty());
        assert!(!detour.landmark_anchor.is_empty());
        assert_route_beat_text_is_not_debug_only(detour.label);
        assert_route_beat_text_is_not_debug_only(detour.landmark_anchor);
        assert_eq!(
            detour.route_positions.len(),
            detour.island_names.len() + detour.lift_names.len()
        );
    }
}

#[test]
fn first_expedition_optional_detours_reconnect_without_becoming_required_objectives() {
    let route = SkyRoute::default();
    let detours = route.first_expedition_optional_detours();
    let baseline_objectives = route.route_objectives(None);
    let plateau_objectives = route.route_objectives(Some("great sky plateau"));
    let upper_crown_objectives = route.route_objectives(Some("upper crown"));
    let non_gating_detour_lifts = [
        "low reef updraft",
        "western catch updraft",
        "bluevault shoulder recovery updraft",
        "plateau west rim recovery updraft",
    ];

    assert_eq!(
        baseline_objectives
            .iter()
            .map(|objective| objective.label)
            .collect::<Vec<_>>(),
        vec!["launch terrace updraft", "landing garden"]
    );
    assert_eq!(plateau_objectives.len(), 10);
    assert_eq!(
        plateau_objectives.last().expect("plateau objective").label,
        "great sky plateau"
    );
    assert_eq!(upper_crown_objectives.len(), 11);
    for lift_name in non_gating_detour_lifts {
        assert!(
            plateau_objectives
                .iter()
                .all(|objective| objective.label != lift_name),
            "{lift_name} should remain optional for the plateau route"
        );
        assert!(
            upper_crown_objectives
                .iter()
                .all(|objective| objective.label != lift_name),
            "{lift_name} should remain optional for the upper route"
        );
    }

    for detour in detours {
        assert!(detour.lift_names.contains(&detour.reconnect_lift_name));
        for &island_name in detour.island_names {
            let island = route
                .island_named(island_name)
                .expect("detour island exists");
            assert_eq!(
                route.ground_at(island.center).island_name,
                Some(island_name)
            );
            assert!(
                plateau_objectives
                    .iter()
                    .all(|objective| objective.label != island_name),
                "{island_name} should not become a required plateau objective"
            );
        }
        for &lift_name in detour.lift_names {
            let node = GAMEPLAY_LIFT_ROUTE
                .iter()
                .find(|node| node.name == lift_name)
                .expect("detour lift exists");
            assert!(node.lift_field().contains(node.center));
            assert!(node.visual_field().contains(node.center));
            assert!(
                node.max_upward_speed <= 28.0,
                "{lift_name} should be readable lift support, not a teleport-like boost"
            );
            assert!(
                node.lift_accel <= 24.0,
                "{lift_name} should stay within established traversal lift acceleration"
            );
        }
    }
}

#[test]
fn first_expedition_navigation_landmarks_cover_required_beats_and_optional_detours() {
    let route = SkyRoute::default();
    let beats = route.first_expedition_route_beats();
    let detours = route.first_expedition_optional_detours();
    let landmarks = route.first_expedition_navigation_landmarks();

    assert_eq!(landmarks.len(), beats.len() + detours.len());
    assert_eq!(
        landmarks
            .iter()
            .filter(|landmark| matches!(
                landmark.context,
                FirstExpeditionNavigationContext::RequiredBeat(_)
            ))
            .count(),
        beats.len()
    );
    assert_eq!(
        landmarks
            .iter()
            .filter(|landmark| matches!(
                landmark.context,
                FirstExpeditionNavigationContext::OptionalDetour(_)
            ))
            .count(),
        detours.len()
    );

    for beat in beats {
        let matching = landmarks
            .iter()
            .filter(|landmark| {
                landmark.context == FirstExpeditionNavigationContext::RequiredBeat(beat.kind)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            matching.len(),
            1,
            "{:?} should have one landmark",
            beat.kind
        );
        let landmark = matching[0];
        assert!(landmark.required_route);
        assert_eq!(landmark.altitude_band, beat.altitude_band);
        assert!(
            landmark.position.distance(beat.position) <= landmark.readable_radius_m,
            "{} should remain within its readable radius of {}",
            landmark.label,
            beat.label
        );
    }

    for detour in detours {
        let matching = landmarks
            .iter()
            .filter(|landmark| {
                landmark.context == FirstExpeditionNavigationContext::OptionalDetour(detour.kind)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            matching.len(),
            1,
            "{:?} should have one non-gating landmark",
            detour.kind
        );
        let landmark = matching[0];
        assert!(!landmark.required_route);
        assert_eq!(landmark.altitude_band, detour.altitude_band);
        assert!(detour.island_names.contains(&landmark.island_name));
        assert!(
            detour
                .route_positions
                .iter()
                .any(|position| position.distance(landmark.position) <= landmark.readable_radius_m),
            "{} should sit on or near its optional detour route",
            landmark.label
        );
    }
}

#[test]
fn first_expedition_navigation_landmarks_are_world_readable_not_debug_text() {
    let route = SkyRoute::default();
    let landmarks = route.first_expedition_navigation_landmarks();
    let mut required_beat_mask = 0_u32;
    let mut optional_detour_mask = 0_u32;

    for landmark in landmarks {
        let island = route
            .island_named(landmark.island_name)
            .expect("landmark island should exist");

        assert!(!landmark.label.is_empty());
        assert!(!landmark.kind.label().is_empty());
        assert!(!landmark.context.label().is_empty());
        assert!(!landmark.visual_anchor.is_empty());
        assert_route_beat_text_is_not_debug_only(landmark.label);
        assert_route_beat_text_is_not_debug_only(landmark.visual_anchor);
        assert!(landmark.position.x.is_finite());
        assert!(landmark.position.y.is_finite());
        assert!(landmark.position.z.is_finite());
        assert!(landmark.readable_radius_m >= 80.0);
        assert!(
            landmark.position.distance(island.center)
                <= island.longest_span_m() + landmark.readable_radius_m,
            "{} should remain visually associated with {}",
            landmark.label,
            island.name
        );

        match landmark.context {
            FirstExpeditionNavigationContext::RequiredBeat(kind) => {
                required_beat_mask |= 1_u32 << kind as u32;
                assert!(landmark.required_route);
            }
            FirstExpeditionNavigationContext::OptionalDetour(kind) => {
                optional_detour_mask |= 1_u32 << kind as u32;
                assert!(!landmark.required_route);
            }
        }
    }

    assert_eq!(
        required_beat_mask.count_ones(),
        route.first_expedition_route_beats().len() as u32
    );
    assert_eq!(
        optional_detour_mask.count_ones(),
        route.first_expedition_optional_detours().len() as u32
    );
}

fn assert_route_beat_text_is_not_debug_only(text: &str) {
    let lower = text.to_ascii_lowercase();

    assert!(
        !lower.contains("debug"),
        "{text} should not depend on debug UI"
    );
    assert!(
        !lower.contains("vector"),
        "{text} should not depend on movement vectors"
    );
    assert!(
        !lower.contains("eval"),
        "{text} should not depend on eval overlays"
    );
}

#[test]
fn route_objective_completion_tracks_flythrough_and_landing() {
    let route = SkyRoute::default();
    let objectives = route.route_objectives(None);
    let target = route.target_island().expect("target island exists");

    assert!(objectives[0].is_complete(&route, GAMEPLAY_LIFT_ROUTE[0].center, FlightMode::Gliding));
    assert!(!objectives[0].is_complete(
        &route,
        GAMEPLAY_LIFT_ROUTE[0].center,
        FlightMode::Airborne
    ));
    assert!(!objectives[0].is_complete(
        &route,
        GAMEPLAY_LIFT_ROUTE[0].center - Vec3::Y * (GAMEPLAY_LIFT_ROUTE[0].half_extents.y + 1.0),
        FlightMode::Gliding
    ));
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

    assert_eq!(route.islands().len(), SKY_ROUTE_ISLAND_COUNT);
    assert!(farthest_z < -3100.0);
}

#[test]
fn route_has_wide_vertical_and_scale_variation() {
    let route = SkyRoute::default();
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut smallest_base_area = f32::INFINITY;
    let mut largest_base_area = 0.0_f32;
    let mut low_island_count = 0;
    let mut high_island_count = 0;

    for island in route.islands() {
        min_y = min_y.min(island.center.y);
        max_y = max_y.max(island.center.y);
        let base_area = island.half_extents.x * island.half_extents.y;
        smallest_base_area = smallest_base_area.min(base_area);
        largest_base_area = largest_base_area.max(base_area);
        low_island_count += usize::from(island.center.y <= 24.0);
        high_island_count += usize::from(island.center.y >= 140.0);
    }

    assert!(max_y - min_y >= 1000.0);
    assert!(low_island_count >= 3);
    assert!(high_island_count >= 14);
    assert!(largest_base_area / smallest_base_area >= 80.0);
}

#[test]
fn route_has_large_traversible_anchor_islands() {
    let route = SkyRoute::default();
    let mut total_base_area = 0.0_f32;
    let mut largest_base_area = 0.0_f32;
    let mut large_anchor_count = 0;

    for island in route.islands() {
        let base_area = island.half_extents.x * island.half_extents.y;
        total_base_area += base_area;
        largest_base_area = largest_base_area.max(base_area);
        large_anchor_count += usize::from(base_area >= 1500.0);
    }

    assert!(total_base_area >= 76_000.0);
    assert!(largest_base_area >= 32_000.0);
    assert!(large_anchor_count >= 12);
}

#[test]
fn route_has_named_great_sky_plateau_anchor() {
    let route = SkyRoute::default();
    let plateau = route
        .island_named("great sky plateau")
        .expect("route should include a named Great Sky Plateau-scale island");
    let largest_non_plateau = route
        .islands()
        .iter()
        .copied()
        .filter(|island| island.name != plateau.name)
        .map(SkyIsland::base_area_m2)
        .fold(0.0_f32, f32::max);

    assert!(plateau.is_great_plateau_anchor());
    assert_eq!(
        plateau.terrain_archetype,
        IslandTerrainArchetype::SkyPlateau
    );
    assert_eq!(
        plateau.world_tags.scale_class,
        IslandScaleClass::HugePlateau
    );
    assert_eq!(plateau.world_tags.route_role, IslandRouteRole::SkyPlateau);
    assert_eq!(
        plateau.world_tags.vertical_profile,
        IslandVerticalProfile::Plateau
    );
    assert_eq!(
        plateau.world_tags.landmark_role,
        IslandLandmarkRole::CaveMouth
    );
    assert!(plateau.base_area_m2() >= largest_non_plateau * 4.0);
    assert!(plateau.longest_span_m() >= 450.0);
    assert!(plateau.thickness >= 68.0);
}

#[test]
fn great_sky_plateau_defines_playable_regions_and_relief() {
    let route = SkyRoute::default();
    let plateau = route
        .island_named("great sky plateau")
        .expect("plateau island exists");
    let regions = [
        IslandPlateauRegion::MeadowPlateau,
        IslandPlateauRegion::CliffRim,
        IslandPlateauRegion::HighShelf,
        IslandPlateauRegion::LowBasin,
        IslandPlateauRegion::BrokenEdge,
        IslandPlateauRegion::UnderhangEntry,
    ];
    let mut region_mask = 0_u32;

    for region in regions {
        assert!(!region.label().is_empty());
        let offset = region.sample_offset();
        assert_eq!(
            plateau.plateau_region_at_normalized_offset(offset),
            Some(region),
            "{region:?} sample offset should classify back to the same plateau region"
        );
        let position = plateau
            .plateau_region_position(region)
            .expect("region should map to a traversable plateau surface");
        let ground = route.ground_at(position);

        assert_eq!(ground.island_name, Some("great sky plateau"));
        assert_eq!(position.y, ground.floor_y);
        region_mask |= 1_u32 << region as u32;
    }

    let high_shelf = plateau
        .plateau_region_position(IslandPlateauRegion::HighShelf)
        .expect("high shelf exists");
    let low_basin = plateau
        .plateau_region_position(IslandPlateauRegion::LowBasin)
        .expect("low basin exists");
    let broken_edge = plateau
        .plateau_region_position(IslandPlateauRegion::BrokenEdge)
        .expect("broken edge exists");
    let meadow = plateau
        .plateau_region_position(IslandPlateauRegion::MeadowPlateau)
        .expect("meadow plateau exists");

    assert_eq!(
        region_mask.count_ones() as usize,
        IslandPlateauRegion::COUNT
    );
    assert!(high_shelf.y - low_basin.y >= 0.30);
    assert!(meadow.distance(broken_edge) >= 150.0);
    assert!(plateau.has_underworld_route_potential());
}

#[test]
fn great_sky_plateau_stays_within_streaming_lod_and_collision_budget() {
    let route = SkyRoute::default();
    let plateau = route
        .island_named("great sky plateau")
        .expect("plateau island exists");
    let stats = route.streaming_lod_stats(plateau.center);
    let body_proxies = terrain_body_collision_proxies(plateau);
    let rim_proxies = terrain_rim_collision_proxies(plateau);

    assert!(plateau.stream_activation(plateau.center).is_active());
    assert_eq!(plateau.lod_band(plateau.center), LodBand::Near);
    assert!(stats.active_island_count <= 8);
    assert!(stats.near_lod_islands <= 3);
    assert_eq!(
        body_proxies.len(),
        TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND
    );
    assert_eq!(rim_proxies.len(), TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND);
}

#[test]
fn great_sky_plateau_defines_under_island_glide_route() {
    let route = SkyRoute::default();
    let plateau = route
        .island_named("great sky plateau")
        .expect("plateau island exists");
    let underbridge_island = route
        .island_named("underbridge cay")
        .expect("underbridge cay exists");
    let under_routes = route.under_island_route_segments();
    let segment = under_routes
        .iter()
        .copied()
        .find(|segment| segment.island_name == "great sky plateau")
        .expect("plateau should expose an under-island glide route");
    let underbridge = under_routes
        .iter()
        .copied()
        .find(|segment| segment.island_name == "underbridge cay")
        .expect("underbridge cay should expose an under-island glide route");
    let lift = GAMEPLAY_LIFT_ROUTE
        .iter()
        .find(|node| node.name == "great sky plateau updraft")
        .expect("plateau should have a paired lift recovery field")
        .lift_field();
    let underbridge_lift = GAMEPLAY_LIFT_ROUTE
        .iter()
        .find(|node| node.name == "underbridge cay updraft")
        .expect("underbridge cay should have a paired lift recovery field")
        .lift_field();

    assert_eq!(under_routes.len(), 2);
    assert_eq!(segment.entry_region, IslandPlateauRegion::UnderhangEntry);
    assert_eq!(segment.exit_region, IslandPlateauRegion::MeadowPlateau);
    assert!(segment.horizontal_length_m() >= 125.0);
    assert!(segment.clearance_radius_m >= 10.0);
    assert!(lift.contains(segment.recommended_lift_point));
    assert_eq!(
        underbridge.entry_region,
        IslandPlateauRegion::UnderhangEntry
    );
    assert_eq!(underbridge.exit_region, IslandPlateauRegion::MeadowPlateau);
    assert!(underbridge.horizontal_length_m() >= 38.0);
    assert!(underbridge.clearance_radius_m >= 4.8);
    assert!(underbridge_lift.contains(underbridge.recommended_lift_point));
    assert!(
        underbridge
            .sample_points()
            .iter()
            .any(|point| underbridge_island.contains_horizontal(*point)),
        "underbridge route should pass beneath the island footprint"
    );

    for point in segment.sample_points() {
        assert!(
            plateau.contains_horizontal(point),
            "under-route point should stay within the plateau footprint"
        );
        assert!(
            point.y < plateau.mesh_top_y() - plateau.thickness * 0.24,
            "under-route point should sit below the top surface"
        );
        assert!(
            point.y > plateau.mesh_top_y() - plateau.thickness * 0.86,
            "under-route point should stay above the tapered underside tip"
        );
    }

    for point in underbridge.sample_points() {
        assert!(
            point.y < underbridge_island.mesh_top_y() - underbridge_island.thickness * 0.32,
            "underbridge route should sit below the top surface"
        );
        assert!(
            point.y > underbridge_island.mesh_top_y() - underbridge_island.thickness * 0.92,
            "underbridge route should stay above the tapered underside tip"
        );
    }
}

#[test]
fn under_route_clearance_does_not_snap_player_to_island_top() {
    let route = SkyRoute::default();
    let underbridge = route
        .island_named("underbridge cay")
        .expect("underbridge cay exists");
    let segment = underbridge
        .under_route_segment()
        .expect("underbridge should expose an under-island route");

    for point in segment.sample_points() {
        let ground = route.ground_at(point);

        assert_ne!(ground.island_name, Some("underbridge cay"));
        assert!(ground.floor_y < point.y - 1.0);
        assert!(!route.is_grounded_at(point));
    }

    let top_surface_position = Vec3::new(
        segment.midpoint.x,
        underbridge.floor_y(),
        segment.midpoint.z,
    );
    let top_ground = route.ground_at(top_surface_position);

    assert_eq!(top_ground.island_name, Some("underbridge cay"));
    assert!(route.is_grounded_at(top_surface_position));
}

#[test]
fn under_route_ground_resolution_preserves_low_glide() {
    let route = SkyRoute::default();
    let underbridge = route
        .island_named("underbridge cay")
        .expect("underbridge cay exists");
    let segment = underbridge
        .under_route_segment()
        .expect("underbridge should expose an under-island route");
    let state = FlightState::new(
        segment.midpoint,
        Vec3::new(0.0, -2.0, -10.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..Default::default()
        },
    );

    let resolved = route.resolve_ground_contact(state);

    assert_eq!(resolved.position, state.position);
    assert_eq!(resolved.velocity, state.velocity);
    assert_eq!(resolved.controller.mode, FlightMode::Gliding);
}

#[test]
fn grounded_under_route_walk_does_not_capture_high_island_top() {
    let route = SkyRoute::default();
    let low_route_position = Vec3::new(-121.1852, PLAYER_STANDING_OFFSET, -182.4107);
    let state = FlightState::new(
        low_route_position,
        Vec3::new(9.0, 0.0, 6.2),
        FlightController::default(),
    );

    let ground = route.ground_at(low_route_position);
    let resolved = route.resolve_ground_contact_after_step(state, true);

    assert_eq!(ground.island_name, None);
    assert_eq!(ground.floor_y, PLAYER_STANDING_OFFSET);
    assert_eq!(resolved.position.y, low_route_position.y);
    assert_eq!(resolved.controller.mode, FlightMode::Grounded);
}

#[test]
fn route_preserves_core_path_and_appends_satellite_islands() {
    let route = SkyRoute::default();
    let core_route_names = [
        "launch mesa",
        "midpoint shelf",
        "landing garden",
        "distant crown",
        "wind overlook",
        "copper stair",
        "sunlit terrace",
        "western refuge",
        "storm porch",
        "high orchard",
        "far needle",
        "sapphire basin",
        "broken stair",
        "mist arch",
        "cloud gate",
    ];
    let satellite_names = [
        "launch spur",
        "garden apron",
        "storm shard",
        "orchard spur",
        "mist stepping stone",
        "underbridge cay",
        "low reef",
        "quiet lower garden",
        "lowwind shelf",
        "upper thermal ring",
        "needle crownlet",
        "skyhook basin",
        "stratos shelf",
        "cloudfall meadow",
        "highgate stair",
        "thin air roost",
        "summit anvil",
        "upper sky shelf",
        "east windchain",
        "bluevault basin",
        "outer switchback",
        "sunspire garden",
        "cloudbreak stair",
        "great sky plateau",
        "far horizon perch",
        "upper crown",
    ];

    for (index, expected_name) in core_route_names.into_iter().enumerate() {
        assert_eq!(route.islands()[index].name, expected_name);
    }
    for expected_name in satellite_names {
        assert!(route.island_named(expected_name).is_some());
    }
}

#[test]
fn playtest_reset_position_targets_known_central_island() {
    let route = SkyRoute::default();
    let reset = route.playtest_reset_position();
    let reset_island = route
        .island_named(PLAYTEST_RESET_ISLAND_NAME)
        .expect("reset island exists");

    assert_eq!(reset.x, reset_island.center.x);
    assert_eq!(reset.z, reset_island.center.z);
    assert_eq!(reset.y, route.ground_at(reset).floor_y);
    assert_eq!(
        route.ground_at(reset).island_name,
        Some(PLAYTEST_RESET_ISLAND_NAME)
    );
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
fn route_defines_explicit_world_taxonomy_for_every_island() {
    let route = SkyRoute::default();
    let mut scale_mask = 0_u32;
    let mut biome_mask = 0_u32;
    let mut water_mask = 0_u32;
    let mut vertical_mask = 0_u32;
    let mut route_role_mask = 0_u32;
    let mut landmark_mask = 0_u32;

    for island in route.islands() {
        let tags = island.world_tags;
        assert!(
            tags.labels().iter().all(|label| !label.is_empty()),
            "{} has an empty taxonomy label",
            island.name
        );
        assert_eq!(
            tags.scale_class,
            IslandScaleClass::from_footprint_area(island.base_area_m2()),
            "{} scale class should match its measured footprint",
            island.name
        );

        scale_mask |= 1_u32 << tags.scale_class as u32;
        biome_mask |= 1_u32 << tags.biome as u32;
        water_mask |= 1_u32 << tags.water_feature as u32;
        vertical_mask |= 1_u32 << tags.vertical_profile as u32;
        route_role_mask |= 1_u32 << tags.route_role as u32;
        landmark_mask |= 1_u32 << tags.landmark_role as u32;
    }

    assert_eq!(scale_mask.count_ones() as usize, IslandScaleClass::COUNT);
    assert!(biome_mask.count_ones() as usize >= IslandBiome::COUNT - 1);
    assert_eq!(water_mask.count_ones() as usize, IslandWaterFeature::COUNT);
    assert!(vertical_mask.count_ones() as usize >= IslandVerticalProfile::COUNT - 1);
    assert!(route_role_mask.count_ones() as usize >= IslandRouteRole::COUNT - 1);
    assert!(landmark_mask.count_ones() as usize >= IslandLandmarkRole::COUNT - 1);
}

#[test]
fn route_defines_authored_composition_for_every_island() {
    let route = SkyRoute::default();
    let compositions = route.island_compositions();
    let mut seen = BTreeSet::new();

    assert_eq!(compositions.len(), route.islands().len());

    for composition in &compositions {
        assert!(
            seen.insert(composition.island_name),
            "{} should have one authored composition record",
            composition.island_name
        );
        assert!(
            route.island_named(composition.island_name).is_some(),
            "{} composition should reference a real island",
            composition.island_name
        );
        assert!(
            composition.labels().iter().all(|label| !label.is_empty()),
            "{} has an empty composition label",
            composition.island_name
        );
        assert!(
            composition
                .player_facing_identity
                .split_whitespace()
                .count()
                >= 3,
            "{} should have a useful player-facing identity",
            composition.island_name
        );
        assert_route_beat_text_is_not_debug_only(composition.player_facing_identity);
        assert!(
            composition.landmark_dependency.is_some(),
            "{} should explicitly name the visual dependency that sells its route purpose",
            composition.island_name
        );
        if let Some(landmark_dependency) = composition.landmark_dependency {
            assert!(!landmark_dependency.is_empty());
            assert_route_beat_text_is_not_debug_only(landmark_dependency);
        }
        assert!(
            !composition.neighbor_islands.is_empty(),
            "{} should participate in the authored neighbor graph",
            composition.island_name
        );
        for neighbor in composition.neighbor_islands {
            assert_ne!(
                *neighbor, composition.island_name,
                "{} should not neighbor itself",
                composition.island_name
            );
            assert!(
                route.island_named(neighbor).is_some(),
                "{} neighbor {} should be a real island",
                composition.island_name,
                neighbor
            );
        }
        if let Some(sightline_target) = composition.sightline_target {
            assert_ne!(sightline_target, composition.island_name);
            assert!(
                route.island_named(sightline_target).is_some(),
                "{} sightline target {} should be a real island",
                composition.island_name,
                sightline_target
            );
        }
    }

    for island in route.islands() {
        assert!(
            seen.contains(island.name),
            "{} should be covered by authored composition",
            island.name
        );
        assert!(route.island_composition(island.name).is_some());
    }
}

#[test]
fn authored_archipelago_composition_graph_is_connected_and_diverse() {
    let route = SkyRoute::default();
    let compositions = route.island_compositions();
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from(["launch mesa"]);
    let mut family_mask = 0_u32;
    let mut motif_mask = 0_u32;
    let mut purpose_mask = 0_u32;
    let mut recovery_loop_count = 0;
    let mut optional_challenge_count = 0;
    let mut high_or_plateau_count = 0;

    while let Some(island_name) = queue.pop_front() {
        if !visited.insert(island_name) {
            continue;
        }
        let composition = route
            .island_composition(island_name)
            .expect("neighbor graph should only contain composed islands");
        for neighbor in composition.neighbor_islands {
            queue.push_back(neighbor);
        }
    }

    for composition in &compositions {
        family_mask |= 1_u32 << composition.family as u32;
        motif_mask |= 1_u32 << composition.visual_motif as u32;
        purpose_mask |= 1_u32 << composition.traversal_purpose as u32;
        recovery_loop_count +=
            usize::from(composition.traversal_purpose == IslandTraversalPurpose::RecoveryLoop);
        optional_challenge_count +=
            usize::from(composition.traversal_purpose == IslandTraversalPurpose::OptionalChallenge);
        high_or_plateau_count +=
            usize::from(composition.altitude_band >= FirstExpeditionAltitudeBand::High);

        for neighbor in composition.neighbor_islands {
            let neighbor_composition = route
                .island_composition(neighbor)
                .expect("neighbor should be composed");
            assert!(
                neighbor_composition
                    .neighbor_islands
                    .contains(&composition.island_name),
                "{} and {} should have a symmetric authored neighbor link",
                composition.island_name,
                neighbor
            );
        }
    }

    assert_eq!(visited.len(), compositions.len());
    assert_eq!(
        family_mask.count_ones() as usize,
        IslandCompositionFamily::COUNT
    );
    assert_eq!(motif_mask.count_ones() as usize, IslandVisualMotif::COUNT);
    assert_eq!(
        purpose_mask.count_ones() as usize,
        IslandTraversalPurpose::COUNT
    );
    assert!(recovery_loop_count >= 6);
    assert!(optional_challenge_count >= 3);
    assert!(high_or_plateau_count >= 10);
}

#[test]
fn authored_composition_aligns_route_beats_detours_and_sightlines() {
    let route = SkyRoute::default();

    for beat in route.first_expedition_route_beats() {
        let composition = route
            .island_composition(beat.anchor_island_name)
            .expect("route beat should have authored composition");
        assert_eq!(
            composition.altitude_band, beat.altitude_band,
            "{} composition should match its route beat altitude band",
            beat.anchor_island_name
        );
        assert_eq!(
            composition.landmark_dependency,
            Some(beat.landmark_anchor),
            "{} should depend on the landmark that pulls the route beat",
            beat.anchor_island_name
        );
        assert!(
            composition.sightline_target.is_some(),
            "{} should name the next visual pull",
            beat.anchor_island_name
        );
    }

    for detour in route.first_expedition_optional_detours() {
        assert!(
            detour.island_names.iter().any(|island_name| {
                route
                    .island_composition(island_name)
                    .is_some_and(|composition| {
                        composition.landmark_dependency == Some(detour.landmark_anchor)
                    })
            }),
            "{} should have at least one island carrying its landmark dependency",
            detour.label
        );
        for island_name in detour.island_names {
            let composition = route
                .island_composition(island_name)
                .expect("detour island should have authored composition");
            assert_eq!(composition.altitude_band, detour.altitude_band);
            assert!(
                matches!(
                    composition.traversal_purpose,
                    IslandTraversalPurpose::RecoveryLoop
                        | IslandTraversalPurpose::OptionalChallenge
                        | IslandTraversalPurpose::LakeVista
                        | IslandTraversalPurpose::ReturnDescent
                ),
                "{} should read as optional/recovery route composition",
                island_name
            );
        }
    }
}

#[test]
fn route_includes_plateau_scale_water_cave_spire_and_stepping_stone_roles() {
    let route = SkyRoute::default();
    let mut tiny_count = 0;
    let mut small_count = 0;
    let mut medium_count = 0;
    let mut large_count = 0;
    let mut huge_plateau_count = 0;
    let mut lake_basin_count = 0;
    let mut waterfall_source_count = 0;
    let mut cave_or_underhang_count = 0;
    let mut spire_count = 0;
    let mut stepping_stone_count = 0;
    let mut plateau_role_count = 0;

    for island in route.islands() {
        let tags = island.world_tags;
        tiny_count += usize::from(tags.scale_class == IslandScaleClass::Tiny);
        small_count += usize::from(tags.scale_class == IslandScaleClass::Small);
        medium_count += usize::from(tags.scale_class == IslandScaleClass::Medium);
        large_count += usize::from(tags.scale_class == IslandScaleClass::Large);
        huge_plateau_count += usize::from(tags.scale_class == IslandScaleClass::HugePlateau);
        lake_basin_count += usize::from(tags.water_feature == IslandWaterFeature::LakeBasin);
        waterfall_source_count +=
            usize::from(tags.water_feature == IslandWaterFeature::WaterfallSource);
        cave_or_underhang_count += usize::from(island.has_underworld_route_potential());
        spire_count += usize::from(tags.vertical_profile == IslandVerticalProfile::Spire);
        stepping_stone_count += usize::from(tags.route_role == IslandRouteRole::SteppingStone);
        plateau_role_count += usize::from(tags.route_role == IslandRouteRole::SkyPlateau);
    }

    assert!(tiny_count >= 5);
    assert!(small_count >= 5);
    assert!(medium_count >= 8);
    assert!(large_count >= 10);
    assert!(huge_plateau_count >= 1);
    assert!(lake_basin_count >= 3);
    assert!(waterfall_source_count >= 1);
    assert!(cave_or_underhang_count >= 2);
    assert!(spire_count >= 3);
    assert!(stepping_stone_count >= 8);
    assert!(plateau_role_count >= 2);
}

#[test]
fn route_footprint_profiles_create_lobes_coves_and_large_playable_shelves() {
    let route = SkyRoute::default();
    let mut lobe_islands = 0;
    let mut cove_islands = 0;

    for island in route.islands() {
        let mut min_scale = f32::INFINITY;
        let mut max_scale = f32::NEG_INFINITY;
        for step in 0..128 {
            let angle = step as f32 / 128.0 * std::f32::consts::TAU;
            let scale = island.playable_silhouette_scale(angle);
            min_scale = min_scale.min(scale);
            max_scale = max_scale.max(scale);
        }

        lobe_islands += usize::from(max_scale > 1.03);
        cove_islands += usize::from(min_scale < 0.82);
        assert!(
            max_scale - min_scale > 0.16,
            "{} footprint is too uniform",
            island.name
        );
        assert_eq!(
            island.footprint_contour_samples(false).len(),
            ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT
        );
    }

    assert!(lobe_islands >= 8);
    assert!(cove_islands >= 8);
}

#[test]
fn island_relief_has_midfield_path_and_crag_detail() {
    let route = SkyRoute::default();
    let island = route
        .island_named("sunlit terrace")
        .expect("sunlit terrace route island exists");
    let sample_count = 144;
    let radius = 0.62;
    let mut min_relief = f32::INFINITY;
    let mut max_relief = f32::NEG_INFINITY;
    let mut trough_samples = 0;
    let mut ridge_samples = 0;
    let mut previous_relief = island.terrain_relief_m(radius, 0.0);
    let mut previous_slope = 0.0_f32;
    let mut slope_reversals = 0;

    for step in 1..=sample_count {
        let angle = step as f32 / sample_count as f32 * std::f32::consts::TAU;
        let relief = island.terrain_relief_m(radius, angle);
        min_relief = min_relief.min(relief);
        max_relief = max_relief.max(relief);
        trough_samples += usize::from(relief <= -0.12);
        ridge_samples += usize::from(relief >= 0.16);

        let slope = relief - previous_relief;
        if previous_slope.abs() > 0.003
            && slope.abs() > 0.003
            && previous_slope.signum() != slope.signum()
        {
            slope_reversals += 1;
        }
        previous_slope = slope;
        previous_relief = relief;
    }

    assert!(max_relief - min_relief >= 0.42);
    assert!(trough_samples >= 4);
    assert!(ridge_samples >= 8);
    assert!(slope_reversals >= 18);
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
fn horizontal_correction_realigns_grounded_player_to_higher_relief_without_damping() {
    let route = SkyRoute::default();
    let island = route.islands()[0];
    let corrected_position = Vec3::new(
        island.center.x + island.half_extents.x * 0.28,
        START_FLOOR_Y - 0.1,
        island.center.z - island.half_extents.y * 0.24,
    );
    let ground = route.ground_at(corrected_position);
    let state = FlightState::new(
        corrected_position,
        Vec3::new(8.0, -1.5, -4.0),
        FlightController::default(),
    );

    let resolved = route.resolve_grounded_after_horizontal_correction(state);

    assert_eq!(resolved.position.y, ground.floor_y);
    assert_eq!(resolved.velocity.x, state.velocity.x);
    assert_eq!(resolved.velocity.z, state.velocity.z);
    assert_eq!(resolved.velocity.y, 0.0);
    assert_eq!(resolved.controller.mode, FlightMode::Grounded);
    assert!(resolved.controller.launch_available);
}

#[test]
fn route_landing_clears_stale_airborne_bank() {
    let route = SkyRoute::default();
    let state = FlightState::new(
        START_POSITION,
        Vec3::new(8.0, -4.0, -4.0),
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
    assert!(resolved.controller.landing_recovery_timer > 0.0);
    assert_eq!(resolved.controller.landing_impact_speed_mps, 4.0);
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
