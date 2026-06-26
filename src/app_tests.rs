use super::*;
use bevy::mesh::{Indices, VertexAttributeValues};
use nau_engine::animation::PlayerPoseIntent;

fn test_island() -> SkyIsland {
    SkyIsland::new(
        "test island",
        Vec3::new(12.0, 40.0, -8.0),
        Vec2::new(22.0, 15.0),
        12.0,
        false,
    )
}

fn positions(mesh: &Mesh) -> &[[f32; 3]] {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => panic!("mesh should expose Float32x3 positions"),
    }
}

fn u32_indices(mesh: &Mesh) -> &[u32] {
    match mesh.indices() {
        Some(Indices::U32(values)) => values,
        _ => panic!("mesh should expose U32 indices"),
    }
}

fn colors(mesh: &Mesh) -> &[[f32; 4]] {
    match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(values)) => values,
        _ => panic!("mesh should expose Float32x4 vertex colors"),
    }
}

fn triangle_normal_y(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let a = Vec3::from_array(positions[indices[0] as usize]);
    let b = Vec3::from_array(positions[indices[1] as usize]);
    let c = Vec3::from_array(positions[indices[2] as usize]);
    (b - a).cross(c - a).y
}

fn normalized_radius(island: SkyIsland, position: [f32; 3]) -> f32 {
    Vec2::new(
        (position[0] - island.center.x) / island.half_extents.x,
        (position[2] - island.center.z) / island.half_extents.y,
    )
    .length()
}

fn radial_range(positions: &[[f32; 3]]) -> f32 {
    let mut min_radius = f32::INFINITY;
    let mut max_radius = f32::NEG_INFINITY;
    for position in positions {
        let radius = Vec2::new(position[0], position[2]).length();
        min_radius = min_radius.min(radius);
        max_radius = max_radius.max(radius);
    }
    max_radius - min_radius
}

#[test]
fn marker_occlusion_detects_island_between_camera_and_marker() {
    let island = SkyIsland::new(
        "blocking island",
        Vec3::new(0.0, 40.0, -40.0),
        Vec2::new(22.0, 16.0),
        14.0,
        false,
    );
    let occlusion = marker_occlusion_between(
        Vec3::new(0.0, 40.0, 0.0),
        Vec3::new(0.0, 40.0, -96.0),
        &[island],
    )
    .expect("island should block the marker ray");

    assert_eq!(occlusion.island_name, "blocking island");
    assert!(occlusion.distance_m > 20.0);
    assert!(occlusion.distance_m < 70.0);
}

#[test]
fn marker_occlusion_ignores_clear_high_ray() {
    let island = SkyIsland::new(
        "low island",
        Vec3::new(0.0, 40.0, -40.0),
        Vec2::new(22.0, 16.0),
        14.0,
        false,
    );

    assert!(
        marker_occlusion_between(
            Vec3::new(0.0, 72.0, 0.0),
            Vec3::new(0.0, 72.0, -96.0),
            &[island],
        )
        .is_none()
    );
}

#[test]
fn authored_player_clip_selection_tracks_flight_state() {
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Grounded, 0.2),
        AuthoredPlayerClip::Idle
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Grounded, 4.0),
        AuthoredPlayerClip::Jog
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Launching, 18.0),
        AuthoredPlayerClip::Launch
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Gliding, 40.0),
        AuthoredPlayerClip::Glide
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Airborne, 16.0),
        AuthoredPlayerClip::AirBrake
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Airborne, 4.0),
        AuthoredPlayerClip::Land
    );
}

#[test]
fn authored_player_clip_selection_tracks_pose_intent() {
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedIdle, 0.2),
        AuthoredPlayerClip::Idle
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedStride, 4.0),
        AuthoredPlayerClip::Jog
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::Diving, 34.0),
        AuthoredPlayerClip::Dive
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::AirTurn, 28.0),
        AuthoredPlayerClip::Glide
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::AirBrake, 28.0),
        AuthoredPlayerClip::AirBrake
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::LandingAnticipation, 12.0),
        AuthoredPlayerClip::Land
    );
}

#[test]
fn authored_player_clip_indices_match_declared_gltf_order() {
    assert_eq!(AuthoredPlayerClip::Idle.index(), 0);
    assert_eq!(AuthoredPlayerClip::Jog.index(), 1);
    assert_eq!(AuthoredPlayerClip::Launch.index(), 2);
    assert_eq!(AuthoredPlayerClip::Glide.index(), 3);
    assert_eq!(AuthoredPlayerClip::Dive.index(), 4);
    assert_eq!(AuthoredPlayerClip::AirBrake.index(), 5);
    assert_eq!(AuthoredPlayerClip::Land.index(), 6);
}

#[test]
fn named_animation_clip_resolution_reports_missing_clips() {
    let mut named_animations = HashMap::new();
    named_animations.insert("Idle_Loop".to_string(), Handle::<AnimationClip>::default());
    named_animations.insert("Glide_Loop".to_string(), Handle::<AnimationClip>::default());

    let resolution = resolve_named_animation_clip_handles(
        &["Idle_Loop", "Jog_Fwd_Loop", "Glide_Loop"],
        &named_animations,
    );

    assert_eq!(resolution.ready_clip_count(), 2);
    assert_eq!(resolution.expected_clip_count, 3);
    assert_eq!(resolution.missing_clip_names, vec!["Jog_Fwd_Loop"]);
    assert!(!resolution.is_complete());
}

#[test]
fn parse_cli_args_accepts_terrain_export() {
    let action = parse_cli_args(
        ["--export-terrain", "target/terrain_export"]
            .into_iter()
            .map(str::to_string),
    )
    .expect("terrain export args should parse");

    match action {
        CliAction::ExportTerrain { output_dir } => {
            assert_eq!(output_dir, PathBuf::from("target/terrain_export"));
        }
        _ => panic!("expected terrain export action"),
    }
}

#[test]
fn parse_cli_args_accepts_visual_content_export() {
    let action = parse_cli_args(
        ["--export-visual-content", "target/visual_content_export"]
            .into_iter()
            .map(str::to_string),
    )
    .expect("visual content export args should parse");

    match action {
        CliAction::ExportVisualContent { output_dir } => {
            assert_eq!(output_dir, PathBuf::from("target/visual_content_export"));
        }
        _ => panic!("expected visual content export action"),
    }
}

#[test]
fn parse_cli_args_rejects_eval_and_terrain_export_together() {
    let error = parse_cli_args(
        [
            "--eval",
            "baseline_route",
            "--export-terrain",
            "target/terrain_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("eval and terrain export should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_eval_and_visual_content_export_together() {
    let error = parse_cli_args(
        [
            "--eval",
            "baseline_route",
            "--export-visual-content",
            "target/visual_content_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("eval and visual content export should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_both_export_paths_together() {
    let error = parse_cli_args(
        [
            "--export-terrain",
            "target/terrain_export",
            "--export-visual-content",
            "target/visual_content_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("export paths should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn metric_only_eval_window_is_hidden_and_unfocused() {
    let scenario = scenario_named("baseline_route").expect("baseline scenario should exist");
    let options = EvalOptions {
        scenario,
        output_dir: PathBuf::from("target/eval/test_hidden_window"),
        capture_screenshot: false,
    };

    let window = primary_window(Some(&options));

    assert!(!window.visible);
    assert!(!window.focused);
    assert!(!window.transparent);
    assert_eq!(window.composite_alpha_mode, CompositeAlphaMode::Opaque);
}

#[test]
fn screenshot_eval_window_remains_visible_for_capture() {
    let scenario = scenario_named("baseline_route").expect("baseline scenario should exist");
    let options = EvalOptions {
        scenario,
        output_dir: PathBuf::from("target/eval/test_visible_window"),
        capture_screenshot: true,
    };

    let window = primary_window(Some(&options));

    assert!(window.visible);
    assert!(window.focused);
    assert!(!window.transparent);
    assert_eq!(window.composite_alpha_mode, CompositeAlphaMode::Opaque);
}

#[test]
fn terrain_export_writes_manifest_meshes_and_weight_sidecars() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-terrain-export-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    remove_existing_dir(&output_dir).expect("stale terrain export dir should be removable");

    let report = export_terrain_inspection(&output_dir).expect("terrain export should succeed");
    let manifest = fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
    let launch_terrain = output_dir.join("islands/00_launch_mesa_terrain.obj");
    let launch_impostor = output_dir.join("islands/00_launch_mesa_impostor.obj");
    let launch_weights = output_dir.join("islands/00_launch_mesa_terrain_material_weights.csv");
    let weights =
        fs::read_to_string(&launch_weights).expect("material weights csv should be readable");

    assert_eq!(report.island_count, SkyRoute::default().islands().len());
    assert_eq!(report.mesh_count, report.island_count * 4);
    assert!(report.total_vertex_count > report.island_count * (2305 + 140));
    assert!(report.total_triangle_count > report.island_count * 4000);
    assert!(report.min_terrain_mesh_vertices >= 2305);
    assert!(report.min_terrain_color_bands >= ISLAND_TERRAIN_COLOR_BANDS);
    assert!(report.min_terrain_material_weight_bands >= ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS);
    assert!(report.min_terrain_material_channels >= ISLAND_TERRAIN_MATERIAL_CHANNELS);
    assert!(report.min_terrain_material_regions >= ISLAND_TERRAIN_MATERIAL_REGIONS);
    assert!(report.min_terrain_height_bands >= ISLAND_TERRAIN_HEIGHT_BANDS);
    assert!(report.min_terrain_normal_slope_bands >= ISLAND_TERRAIN_NORMAL_SLOPE_BANDS);
    assert!(report.min_terrain_texture_detail_bands >= ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS);
    assert!(report.min_terrain_texture_edge_promille >= ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE);
    assert!(report.min_terrain_relief_range_m >= 0.8);
    assert!(report.min_cliff_color_bands >= ISLAND_CLIFF_STRATA_BANDS / 2);
    assert!(report.min_impostor_mesh_vertices >= 2 + ISLAND_IMPOSTOR_SEGMENTS * 3);
    assert!(report.min_impostor_color_bands >= ISLAND_IMPOSTOR_COLOR_BANDS);
    assert!(launch_terrain.exists());
    assert!(launch_impostor.exists());
    assert!(launch_weights.exists());
    assert!(weights.starts_with("vertex,lush_highland,exposed_edge\n"));
    assert!(weights.lines().count() > 2000);
    assert!(manifest.contains("\"schema\": \"nau_terrain_export.v1\""));
    assert!(manifest.contains(
        "\"material_weights_csv\": \"islands/00_launch_mesa_terrain_material_weights.csv\""
    ));
    assert!(manifest.contains(&format!(
        "\"terrain_material_weight_bands\": {}",
        report.min_terrain_material_weight_bands
    )));
    assert!(manifest.contains(&format!(
        "\"terrain_material_regions\": {}",
        report.min_terrain_material_regions
    )));
    assert!(manifest.contains("\"terrain_height_bands\""));
    assert!(manifest.contains("\"terrain_normal_slope_bands\""));
    assert!(manifest.contains(&format!(
        "\"terrain_texture_detail_bands\": {}",
        report.min_terrain_texture_detail_bands
    )));
    assert!(manifest.contains(&format!(
        "\"terrain_texture_edge_promille\": {}",
        report.min_terrain_texture_edge_promille
    )));
    assert!(manifest.contains("\"impostor_mesh_vertices\": 146"));
    assert!(manifest.contains("\"impostor_color_bands\": 21"));
    assert!(manifest.contains("\"impostor\": {\"obj\": \"islands/00_launch_mesa_impostor.obj\""));

    remove_existing_dir(&output_dir).expect("terrain export test dir should be removable");
}

#[test]
fn visual_content_export_writes_manifest_meshes_and_shape_metrics() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-visual-content-export-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    remove_existing_dir(&output_dir).expect("stale visual content export dir should be removable");

    let report = export_visual_content_inspection(&output_dir)
        .expect("visual content export should succeed");
    let manifest = fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
    let launch_ground_cover = output_dir.join("visuals/00_launch_mesa_ground_cover.obj");
    let launch_tree_trunk = output_dir.join("visuals/00_launch_mesa_launch_tree_trunk.obj");
    let launch_cloud = output_dir.join("visuals/00_launch_mesa_bank_0.obj");
    let launch_beacon = output_dir.join("visuals/00_launch_mesa_launch_beacon.obj");
    let midpoint_cairn = output_dir.join("visuals/01_midpoint_shelf_route_cairn.obj");
    let launch_pond = output_dir.join("visuals/00_launch_mesa_pond_surface.obj");
    let landing_marker = output_dir.join("visuals/02_landing_garden_landing_garden_marker_0.obj");

    assert_eq!(
        report.ground_cover_count,
        SkyRoute::default().islands().len()
    );
    assert_eq!(
        report.ground_cover_patch_total,
        SkyRoute::default().islands().len() * GROUND_COVER_PATCHES
    );
    assert_eq!(
        report.ground_cover_blade_total,
        SkyRoute::default().islands().len() * GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH
    );
    assert_eq!(report.tree_trunk_count, 36);
    assert_eq!(report.tree_canopy_count, 36);
    assert_eq!(report.weather_cloud_count, 30);
    assert_eq!(
        report.weather_cloud_bank_count,
        SkyRoute::default().islands().len()
    );
    assert_eq!(report.weather_cloud_veil_count, 18);
    assert_eq!(report.landmark_count, 27);
    assert_eq!(report.route_cairn_count, 10);
    assert_eq!(report.launch_beacon_count, 1);
    assert_eq!(report.landing_garden_marker_count, 4);
    assert_eq!(
        report.pond_surface_count,
        SkyRoute::default().islands().len()
    );
    assert_eq!(
        report.mesh_count,
        report.ground_cover_count
            + report.tree_trunk_count * 2
            + report.weather_cloud_count
            + report.landmark_count
    );
    assert!(report.total_vertex_count > 70_000);
    assert!(report.total_triangle_count > 75_000);
    assert!(report.min_ground_cover_mesh_vertices >= 1100);
    assert!(report.min_ground_cover_blade_count >= 220);
    assert!(report.min_ground_cover_blade_height_range_m >= 0.7);
    assert!(report.min_tree_trunk_mesh_vertices >= 190);
    assert!(report.min_tree_trunk_taper_ratio >= 1.35);
    assert!(report.min_tree_branch_reach_ratio >= 1.8);
    assert!(report.min_tree_branch_count >= 4);
    assert!(report.min_tree_root_flare_count >= 5);
    assert!(report.min_tree_trunk_ring_count >= 5);
    assert!(report.tree_trunk_height_range_m >= 1.5);
    assert!(report.min_tree_canopy_mesh_vertices >= 400);
    assert!(report.min_tree_canopy_lobe_count >= 6);
    assert!(report.min_tree_canopy_detail_card_count >= 12);
    assert!(report.min_tree_canopy_vertical_to_horizontal_ratio >= 0.45);
    assert!(report.tree_canopy_radius_range_m >= 0.35);
    assert!(report.min_weather_cloud_mesh_vertices >= 1458);
    assert!(report.min_weather_cloud_lobe_count >= 9);
    assert!(report.min_weather_cloud_wisp_card_count >= 27);
    assert!(report.min_weather_cloud_filament_ribbon_detail_count >= 27);
    assert!(report.min_weather_cloud_bank_depth_m >= 5.8);
    assert!(report.min_weather_cloud_bank_lobe_count >= 18);
    assert!(report.min_weather_cloud_scaled_depth_span_m >= 12.0);
    assert!(report.min_route_cairn_mesh_vertices >= 240);
    assert!(report.min_route_cairn_vertical_span_m >= 3.0);
    assert!(report.min_launch_beacon_mesh_vertices >= 300);
    assert!(report.min_launch_beacon_vertical_span_m >= 2.8);
    assert!(report.min_landing_garden_marker_mesh_vertices >= 39);
    assert!(report.min_landing_garden_marker_vertical_span_m >= 0.12);
    assert!(report.min_pond_surface_mesh_vertices >= 65);
    assert!(report.min_pond_surface_vertical_span_m >= 0.015);
    assert_eq!(
        report.terrain_biome_palette_count,
        TERRAIN_BIOME_PALETTE_COUNT
    );
    assert_eq!(report.foliage_palette_count, TERRAIN_BIOME_PALETTE_COUNT);
    assert!(report.stone_palette_count >= TERRAIN_BIOME_PALETTE_COUNT - 1);
    assert!(launch_ground_cover.exists());
    assert!(launch_tree_trunk.exists());
    assert!(launch_cloud.exists());
    assert!(launch_beacon.exists());
    assert!(midpoint_cairn.exists());
    assert!(launch_pond.exists());
    assert!(landing_marker.exists());
    assert!(manifest.contains("\"schema\": \"nau_visual_content_export.v1\""));
    assert!(manifest.contains("\"ground_cover_blade_height_range_m\""));
    assert!(manifest.contains("\"tree_branch_reach_ratio\""));
    assert!(manifest.contains("\"tree_root_flare_count\": 5"));
    assert!(manifest.contains("\"tree_trunk_ring_count\": 5"));
    assert!(manifest.contains("\"tree_trunk_height_range_m\""));
    assert!(manifest.contains("\"tree_canopy_radius_range_m\""));
    assert!(manifest.contains("\"weather_cloud_veil_count\": 18"));
    assert!(manifest.contains("\"weather_cloud_scaled_depth_span_m\""));
    assert!(manifest.contains("\"weather_cloud_wisp_card_count\""));
    assert!(manifest.contains("\"weather_cloud_filament_ribbon_detail_count\""));
    assert!(manifest.contains("\"landmark_count\": 27"));
    assert!(manifest.contains("\"route_cairn_count\": 10"));
    assert!(manifest.contains("\"launch_beacon_count\": 1"));
    assert!(manifest.contains("\"landing_garden_marker_count\": 4"));
    assert!(manifest.contains("\"pond_surface_count\": 12"));
    assert!(manifest.contains("\"route_cairn_vertical_span_m\""));
    assert!(manifest.contains("\"launch_beacon_vertical_span_m\""));
    assert!(manifest.contains("\"landing_garden_marker_vertical_span_m\""));
    assert!(manifest.contains("\"pond_surface_vertical_span_m\""));
    assert!(manifest.contains("\"terrain_biome_palette_count\": 5"));

    remove_existing_dir(&output_dir).expect("visual content export test dir should be removable");
}

#[test]
fn spawned_island_visuals_attach_world_collision_proxies() {
    let route = SkyRoute::default();
    let mut catalog = IslandVisualCatalog::default();
    let mut diagnostics = content_diagnostics::IslandContentDiagnostics::default();
    let mut meshes = Assets::<Mesh>::default();
    let material = Handle::<StandardMaterial>::default();
    let detail_materials = IslandDetailMaterials {
        trunk: material.clone(),
        foliage: material.clone(),
        ground_cover: material.clone(),
        stone: material.clone(),
    };

    for (index, island) in route.islands().iter().copied().enumerate() {
        queue_sky_island(
            &mut catalog,
            &mut diagnostics,
            &mut meshes,
            material.clone(),
            material.clone(),
            material.clone(),
            material.clone(),
            material.clone(),
            detail_materials.clone(),
            material.clone(),
            material.clone(),
            index,
            island,
        );
    }

    let mut world = World::new();
    {
        let mut commands = world.commands();
        spawn_initial_island_visuals(&mut commands, &catalog, nau_engine::world::START_POSITION);
    }
    world.flush();

    let mut query = world.query::<&WorldCollisionProxy>();
    let proxies = query.iter(&world).copied().collect::<Vec<_>>();

    assert!(proxies.len() >= 24);
    assert!(
        proxies
            .iter()
            .any(|proxy| proxy.kind == WorldCollisionProxyKind::Tree)
    );
    assert!(
        proxies
            .iter()
            .any(|proxy| proxy.kind == WorldCollisionProxyKind::Rock)
    );
    assert!(
        proxies
            .iter()
            .any(|proxy| proxy.kind == WorldCollisionProxyKind::Landmark)
    );
}

#[test]
fn tree_trunk_mesh_is_tapered_instead_of_a_straight_cylinder() {
    let mesh = tree_trunk_mesh(0.3, 4.0, 123);
    let positions = positions(&mesh);
    let bottom_ring = &positions[..TREE_TRUNK_SEGMENTS];
    let top_ring_start = TREE_TRUNK_SEGMENTS * (TREE_TRUNK_RING_COUNT - 1);
    let top_ring = &positions[top_ring_start..top_ring_start + TREE_TRUNK_SEGMENTS];
    let branch_vertices_start = TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT + 2;
    let root_vertices_start = branch_vertices_start + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2;
    let top_center = top_ring
        .iter()
        .map(|position| Vec2::new(position[0], position[2]))
        .sum::<Vec2>()
        / TREE_TRUNK_SEGMENTS as f32;
    let average_bottom_radius = bottom_ring
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .sum::<f32>()
        / TREE_TRUNK_SEGMENTS as f32;
    let average_top_radius = top_ring
        .iter()
        .map(|position| (Vec2::new(position[0], position[2]) - top_center).length())
        .sum::<f32>()
        / TREE_TRUNK_SEGMENTS as f32;
    let max_branch_reach = positions[branch_vertices_start..root_vertices_start]
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);
    let max_root_reach = positions[root_vertices_start..]
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT
            + 2
            + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2
            + TREE_ROOT_FLARE_COUNT * TREE_ROOT_FLARE_SEGMENTS * 2
    );
    assert!(
        average_bottom_radius > average_top_radius * 1.45,
        "tree trunks should taper enough to stop reading as plain cylinders"
    );
    assert!(
        max_branch_reach > average_bottom_radius * 1.8,
        "tree trunks should include visible branch mass instead of only a tapered stick"
    );
    assert!(
        max_root_reach > average_bottom_radius * 1.35,
        "tree trunks should include root flares that break the pole silhouette near the ground"
    );
}

#[test]
fn tree_trunk_cap_winding_matches_declared_normals() {
    let mesh = tree_trunk_mesh(0.3, 4.0, 123);
    let positions = positions(&mesh);
    let indices = u32_indices(&mesh);
    let cap_start = TREE_TRUNK_SEGMENTS * 6 * (TREE_TRUNK_RING_COUNT - 1);

    for segment in 0..TREE_TRUNK_SEGMENTS {
        let bottom = &indices[cap_start + segment * 6..cap_start + segment * 6 + 3];
        let top = &indices[cap_start + segment * 6 + 3..cap_start + segment * 6 + 6];

        assert!(
            triangle_normal_y(positions, bottom) < 0.0,
            "bottom cap triangles should face downward"
        );
        assert!(
            triangle_normal_y(positions, top) > 0.0,
            "top cap triangles should face upward"
        );
    }
}

#[test]
fn tree_canopy_mesh_uses_overlapping_lobes_instead_of_one_sphere() {
    let mesh = tree_canopy_mesh(1.4, 42);
    let positions = positions(&mesh);
    let single_lobe_vertices =
        (TREE_CANOPY_LATITUDE_SEGMENTS + 1) * (TREE_CANOPY_LONGITUDE_SEGMENTS + 1);
    let expected_card_vertices = TREE_CANOPY_CARD_COUNT * DETAIL_CARD_VERTICES;
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let horizontal_span = positions
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);

    assert!(mesh.count_vertices() > single_lobe_vertices * 3);
    assert!(mesh.count_vertices() >= single_lobe_vertices + expected_card_vertices);
    assert!(max_y - min_y > 1.9);
    assert!(horizontal_span > 1.45);
}

#[test]
fn cloud_cluster_mesh_uses_multiple_lobes_for_depth() {
    let mesh = cloud_cluster_mesh(99, CLOUD_BANK_LOBES);
    let positions = positions(&mesh);
    let lobe_vertices = (5 + 1) * (10 + 1);
    let card_vertices = CLOUD_WISP_CARDS_PER_LOBE * DETAIL_CARD_VERTICES;
    let filament_vertices = CLOUD_FILAMENT_RIBBONS_PER_LOBE * CLOUD_FILAMENT_RIBBON_VERTICES;
    let min_x = positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_z = positions
        .iter()
        .map(|position| position[2])
        .fold(f32::INFINITY, f32::min);
    let max_z = positions
        .iter()
        .map(|position| position[2])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        CLOUD_BANK_LOBES * (lobe_vertices + card_vertices + filament_vertices)
    );
    assert_eq!(
        cloud_filament_ribbon_detail_count(CLOUD_BANK_LOBES),
        CLOUD_BANK_LOBES * CLOUD_FILAMENT_RIBBONS_PER_LOBE
    );
    assert!(
        max_x - min_x > 1.2,
        "cloud clusters should have lateral lobe structure"
    );
    assert!(
        max_y - min_y > 1.0,
        "cloud clusters should stack lobes vertically instead of staying wafer-flat"
    );
    assert!(
        max_z - min_z > 0.8,
        "cloud clusters should have visible depth, not one flat blob"
    );
}

#[test]
fn ground_cover_mesh_uses_dense_curved_blades() {
    let mesh = island_ground_cover_mesh(2, test_island());
    let positions = positions(&mesh);
    let indices = u32_indices(&mesh);
    let blade_count = GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH;
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        blade_count * VERTICES_PER_GROUND_BLADE
    );
    assert_eq!(indices.len(), blade_count * INDICES_PER_GROUND_BLADE);
    assert!(
        max_y - min_y > 1.0,
        "ground cover should have enough varied height to read as dense vegetation"
    );
}

#[test]
fn island_detail_surface_offsets_clamp_to_playable_silhouette() {
    let island = SkyIsland::new(
        "storm porch",
        Vec3::ZERO,
        Vec2::new(42.0, 28.0),
        15.0,
        false,
    );
    let mut narrow_angle = 0.0;
    let mut narrow_scale = f32::INFINITY;
    for step in 0..128 {
        let angle = step as f32 / 128.0 * std::f32::consts::TAU;
        let scale = island.playable_silhouette_scale(angle);
        if scale < narrow_scale {
            narrow_angle = angle;
            narrow_scale = scale;
        }
    }

    let outside_offset = Vec2::new(narrow_angle.cos(), narrow_angle.sin()) * 0.92;
    let clamped_offset = island_playable_normalized_offset(island, outside_offset);
    let surface = island_visual_surface_position(island, outside_offset);

    assert!(clamped_offset.length() < outside_offset.length());
    assert!(clamped_offset.length() <= narrow_scale * 0.941);
    assert!(island.contains_horizontal(surface));
}

#[test]
fn rock_scatter_mesh_has_flattened_irregular_silhouette() {
    let mesh = rock_scatter_mesh(0.7, 1234);
    let positions = positions(&mesh);
    let indices = u32_indices(&mesh);
    let radial_lengths: Vec<f32> = positions[1..positions.len() - 1]
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .collect();
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_radius = radial_lengths.iter().copied().fold(f32::INFINITY, f32::min);
    let max_radius = radial_lengths
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let top_cap_start = ROCK_MESH_SEGMENTS * 3 + (ROCK_MESH_RINGS - 1) * ROCK_MESH_SEGMENTS * 6;

    assert_eq!(
        mesh.count_vertices(),
        ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2
    );
    assert!(
        max_radius - min_radius > 0.5,
        "rock scatter should have a jagged profile instead of one repeated radius"
    );
    assert!(
        max_radius > (max_y - min_y) * 0.95,
        "rock scatter should be squat and grounded rather than a sphere"
    );
    assert!(
        triangle_normal_y(positions, &indices[0..3]) < 0.0,
        "rock bottom cap should face downward"
    );
    assert!(
        triangle_normal_y(positions, &indices[top_cap_start..top_cap_start + 3]) > 0.0,
        "rock top cap should face upward"
    );
}

#[test]
fn landmark_meshes_replace_basic_cylinders_and_boxes() {
    let cairn = route_cairn_mesh(0.44, 4.2, 12_345);
    let cairn_positions = positions(&cairn);
    let cairn_y_range = mesh_y_range(&cairn);
    let cairn_radius_range = radial_range(cairn_positions);

    assert!(
        cairn.count_vertices() > ROUTE_CAIRN_STONE_COUNT * 40,
        "route cairns should be stacked stone meshes, not one cylinder"
    );
    assert!(cairn_y_range > 3.0);
    assert!(
        cairn_radius_range > 0.18,
        "route cairn stones should vary their silhouette radius"
    );

    let launch_beacon = launch_beacon_mesh(0.78, 3.2, 14_321);
    let launch_positions = positions(&launch_beacon);
    assert!(
        launch_beacon.count_vertices() > cairn.count_vertices() + LAUNCH_BEACON_CRYSTAL_COUNT * 10,
        "launch beacon should add shard geometry on top of its stone base"
    );
    assert!(
        mesh_y_range(&launch_beacon) > 2.8,
        "launch beacon should read as a vertical landmark"
    );
    assert!(
        launch_positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max)
            > 2.0
    );

    let marker = landing_garden_marker_mesh(8.0, 0.62, 13_579);
    let marker_positions = positions(&marker);
    let marker_indices = u32_indices(&marker);
    let min_x = marker_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = marker_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        marker.count_vertices(),
        (LANDING_GARDEN_MARKER_SEGMENTS + 1) * 3
    );
    assert_eq!(marker_indices.len(), LANDING_GARDEN_MARKER_SEGMENTS * 12);
    assert!(max_x - min_x > 7.8);
    assert!(
        mesh_y_range(&marker) > 0.12,
        "landing garden markers should be low organic mounds, not flat boxes"
    );

    let pond = pond_surface_mesh(3.2, 1.4, 11_789);
    let pond_positions = positions(&pond);
    let pond_indices = u32_indices(&pond);
    let pond_radius_range = radial_range(pond_positions);

    assert_eq!(pond.count_vertices(), 1 + POND_SURFACE_SEGMENTS * 2);
    assert_eq!(pond_indices.len(), POND_SURFACE_SEGMENTS * 9);
    assert!(
        pond_radius_range > 2.1,
        "pond perimeter should be an irregular surface mesh, not a scaled cylinder"
    );
    assert!(
        mesh_y_range(&pond) > 0.015,
        "pond surface should carry subtle ripple variation"
    );
}

#[test]
fn terrain_mesh_uses_high_resolution_irregular_silhouette() {
    let island = test_island();
    let mesh = island_terrain_mesh(2, island);
    let positions = positions(&mesh);
    let colors = colors(&mesh);
    let outer_ring_start = 1 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS;
    let outer_ring = &positions[outer_ring_start..outer_ring_start + ISLAND_BODY_SEGMENTS];
    let min_radius = outer_ring
        .iter()
        .map(|position| normalized_radius(island, *position))
        .fold(f32::INFINITY, f32::min);
    let max_radius = outer_ring
        .iter()
        .map(|position| normalized_radius(island, *position))
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS
    );
    assert!(
        max_radius <= 1.001,
        "playable terrain must stay inside the route collision footprint"
    );
    assert!(
        max_radius - min_radius > 0.10,
        "outer ring should not read as a perfect cylinder"
    );
    assert_eq!(colors.len(), positions.len());
    assert!(
        mesh_vertex_color_band_count(&mesh) >= ISLAND_TERRAIN_COLOR_BANDS,
        "terrain mesh should carry vertex-color biome/detail variation"
    );
    assert!(
        mesh_terrain_material_weight_band_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS,
        "terrain mesh should carry material-weight variation for future PBR blends"
    );
    assert!(
        mesh_terrain_material_channel_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_CHANNELS,
        "terrain mesh should expose base, lush, and edge material channels"
    );
    assert!(
        mesh_terrain_material_region_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_REGIONS,
        "terrain mesh should expose distinct meadow, transition, highland, and edge regions"
    );
    assert!(
        mesh_vertical_band_count(&mesh) >= ISLAND_TERRAIN_HEIGHT_BANDS,
        "terrain mesh should expose enough vertical bands for ravines and terrace relief"
    );
    assert!(
        mesh_normal_slope_band_count(&mesh) >= ISLAND_TERRAIN_NORMAL_SLOPE_BANDS,
        "terrain mesh normals should expose slope variety rather than one smooth plate"
    );
    let uvs = mesh_uv0(&mesh).expect("terrain mesh should expose material uvs");
    let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
    let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
    let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_u - min_u >= 3.0 && max_v - min_v >= 2.0,
        "terrain albedo should tile across large islands instead of stretching one texture over the whole surface"
    );
    assert!(
        mesh_y_range(&mesh) >= 0.8,
        "terrain mesh should have enough relief range to avoid flat plateaus"
    );
}

#[test]
fn island_impostor_mesh_uses_layered_color_and_silhouette() {
    let island = test_island();
    let mesh = island_impostor_mesh(4, island);
    let positions = positions(&mesh);
    let colors = colors(&mesh);
    let top_ring = &positions[1..1 + ISLAND_IMPOSTOR_SEGMENTS];
    let min_radius = top_ring
        .iter()
        .map(|position| normalized_radius(island, *position))
        .fold(f32::INFINITY, f32::min);
    let max_radius = top_ring
        .iter()
        .map(|position| normalized_radius(island, *position))
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(mesh.count_vertices(), 2 + ISLAND_IMPOSTOR_SEGMENTS * 3);
    assert_eq!(colors.len(), positions.len());
    assert!(
        max_radius - min_radius > 0.08,
        "distant impostor should keep an irregular island silhouette"
    );
    assert!(
        mesh_y_range(&mesh) >= island.thickness * 0.85,
        "distant impostor should include a readable underside mass"
    );
    assert!(
        mesh_vertex_color_band_count(&mesh) >= ISLAND_IMPOSTOR_COLOR_BANDS,
        "distant impostor should carry terrain, cliff, and underside color variation"
    );
}

#[test]
fn cliff_and_underside_meshes_replace_cylinder_body_resolution() {
    let island = test_island();
    let cliff_mesh = island_cliff_mesh(3, island);
    let underside_mesh = island_underside_mesh(3, island);
    let underside_positions = positions(&underside_mesh);
    let underside_top_radius = normalized_radius(island, underside_positions[0]);
    let underside_tip = *underside_positions.last().expect("bottom tip exists");

    assert_eq!(
        cliff_mesh.count_vertices(),
        (ISLAND_CLIFF_RINGS + 1) * ISLAND_BODY_SEGMENTS
    );
    assert_eq!(
        underside_mesh.count_vertices(),
        (ISLAND_UNDERSIDE_RINGS + 1) * ISLAND_BODY_SEGMENTS + 1
    );
    assert!(underside_top_radius > 0.55);
    assert!(normalized_radius(island, underside_tip) < 0.01);
    assert!(underside_tip[1] < island.mesh_top_y() - island.thickness * 1.5);
    assert!(
        mesh_vertex_color_band_count(&cliff_mesh) >= ISLAND_CLIFF_STRATA_BANDS,
        "cliff mesh should carry visible strata color bands"
    );
    assert!(
        mesh_vertex_color_band_count(&underside_mesh) >= ISLAND_CLIFF_STRATA_BANDS / 2,
        "underside mesh should not be one flat rock color"
    );
}

#[test]
fn cliff_and_underside_share_their_transition_ring() {
    let island = test_island();
    let cliff_mesh = island_cliff_mesh(3, island);
    let underside_mesh = island_underside_mesh(3, island);
    let cliff_positions = positions(&cliff_mesh);
    let underside_positions = positions(&underside_mesh);
    let cliff_bottom_start = ISLAND_CLIFF_RINGS * ISLAND_BODY_SEGMENTS;

    for segment in 0..ISLAND_BODY_SEGMENTS {
        let cliff = Vec3::from_array(cliff_positions[cliff_bottom_start + segment]);
        let underside = Vec3::from_array(underside_positions[segment]);
        assert!(
            cliff.distance(underside) < 0.001,
            "cliff and underside should not leave a visible body seam"
        );
    }
}

#[test]
fn terrain_surface_texture_has_sharp_material_detail() {
    let data = procedural_terrain_surface_texture_data(
        [54, 128, 70, 255],
        [28, 92, 48, 255],
        [128, 174, 78, 255],
        17,
        TERRAIN_TEXTURE_SIZE,
    );

    assert_eq!(
        data.len(),
        (TERRAIN_TEXTURE_SIZE * TERRAIN_TEXTURE_SIZE * 4) as usize
    );
    assert!(
        texture_detail_band_count(&data) >= ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS,
        "terrain texture should carry enough high-frequency color bins to avoid blurry flat fills"
    );
    assert!(
        texture_edge_promille(&data, TERRAIN_TEXTURE_SIZE) >= ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE,
        "terrain texture should carry enough local edge contrast to avoid smeared low-frequency fills"
    );
}

#[test]
fn terrain_biome_palettes_vary_base_hues() {
    let palette_keys = (0..5)
        .map(|index| {
            let grass = terrain_biome_palette(index).grass;
            [
                (grass.x * 31.0).round() as u8,
                (grass.y * 31.0).round() as u8,
                (grass.z * 31.0).round() as u8,
            ]
        })
        .collect::<HashSet<_>>();

    assert_eq!(
        palette_keys.len(),
        5,
        "terrain palettes should give repeated island materials distinct base hues"
    );
}

#[test]
fn terrain_vertex_colors_use_biome_palette_variation() {
    let color_keys = (0..5)
        .map(|index| {
            let color = island_terrain_vertex_color(index, 0.56, 1.2, 0.24);
            [
                (color[0] * 31.0).round() as u8,
                (color[1] * 31.0).round() as u8,
                (color[2] * 31.0).round() as u8,
            ]
        })
        .collect::<HashSet<_>>();

    assert!(
        color_keys.len() >= 4,
        "same-region terrain samples should not collapse into one shared island palette"
    );
}

#[test]
fn biome_detail_color_sets_vary_vegetation_and_stone_hues() {
    let foliage_keys = (0..TERRAIN_BIOME_PALETTE_COUNT)
        .map(|index| biome_detail_color_set(index).foliage_primary)
        .collect::<HashSet<_>>();
    let stone_keys = (0..TERRAIN_BIOME_PALETTE_COUNT)
        .map(|index| biome_detail_color_set(index).stone_primary)
        .collect::<HashSet<_>>();

    assert_eq!(
        foliage_keys.len(),
        TERRAIN_BIOME_PALETTE_COUNT,
        "generated tree canopies should inherit per-island biome identity"
    );
    assert!(
        stone_keys.len() >= TERRAIN_BIOME_PALETTE_COUNT - 1,
        "stone scatter should vary with the island biome instead of sharing one material"
    );
}
