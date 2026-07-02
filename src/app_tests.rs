use super::*;
use bevy::mesh::{Indices, VertexAttributeValues};
use nau_engine::animation::PlayerPoseIntent;
use nau_engine::movement::FlightInput;
use nau_engine::world::{
    IslandLandmarkRole, IslandPlateauRegion, IslandScaleClass, IslandTerrainArchetype,
    IslandWaterFeature,
};

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

fn normalized_visual_radius(island: SkyIsland, position: [f32; 3]) -> f32 {
    let normalized = Vec2::new(
        (position[0] - island.center.x) / island.half_extents.x,
        (position[2] - island.center.z) / island.half_extents.y,
    );
    normalized.length() / island.visual_silhouette_scale(normalized.y.atan2(normalized.x))
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
        AuthoredPlayerClip::Walk
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Grounded, 7.0),
        AuthoredPlayerClip::Run
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
        AuthoredPlayerClip::Fall
    );
    assert_eq!(
        authored_player_clip_for_state(FlightMode::Airborne, 4.0),
        AuthoredPlayerClip::Fall
    );
}

#[test]
fn authored_player_clip_selection_tracks_pose_intent() {
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedIdle, 0.2),
        AuthoredPlayerClip::Idle
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedWalk, 4.0),
        AuthoredPlayerClip::Walk
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedStride, 4.0),
        AuthoredPlayerClip::Run
    );
    assert_eq!(
        authored_player_clip_for_pose_intent(PlayerPoseIntent::GroundedRun, 4.0),
        AuthoredPlayerClip::Run
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
        authored_player_clip_for_pose_intent_with_input(
            PlayerPoseIntent::AirTurn,
            28.0,
            FlightInput {
                left: true,
                ..default()
            },
        ),
        AuthoredPlayerClip::BankLeft
    );
    assert_eq!(
        authored_player_clip_for_pose_intent_with_input(
            PlayerPoseIntent::AirTurn,
            28.0,
            FlightInput {
                right: true,
                ..default()
            },
        ),
        AuthoredPlayerClip::BankRight
    );
    assert_eq!(
        authored_player_clip_for_pose_intent_with_input(
            PlayerPoseIntent::Diving,
            34.0,
            FlightInput {
                right: true,
                dive: true,
                ..default()
            },
        ),
        AuthoredPlayerClip::Dive
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
    assert_eq!(AuthoredPlayerClip::Walk.index(), 1);
    assert_eq!(AuthoredPlayerClip::Run.index(), 2);
    assert_eq!(AuthoredPlayerClip::Launch.index(), 3);
    assert_eq!(AuthoredPlayerClip::Fall.index(), 4);
    assert_eq!(AuthoredPlayerClip::Glide.index(), 5);
    assert_eq!(AuthoredPlayerClip::BankLeft.index(), 6);
    assert_eq!(AuthoredPlayerClip::BankRight.index(), 7);
    assert_eq!(AuthoredPlayerClip::Dive.index(), 8);
    assert_eq!(AuthoredPlayerClip::AirBrake.index(), 9);
    assert_eq!(AuthoredPlayerClip::Land.index(), 10);
}

#[test]
fn named_animation_clip_resolution_reports_missing_clips() {
    let mut named_animations = HashMap::new();
    named_animations.insert("Idle_Loop".to_string(), Handle::<AnimationClip>::default());
    named_animations.insert("Glide_Loop".to_string(), Handle::<AnimationClip>::default());

    let resolution = resolve_named_animation_clip_handles(
        &["Idle_Loop", "Walk_Fwd_Loop", "Glide_Loop"],
        &named_animations,
    );

    assert_eq!(resolution.ready_clip_count(), 2);
    assert_eq!(resolution.expected_clip_count, 3);
    assert_eq!(resolution.missing_clip_names, vec!["Walk_Fwd_Loop"]);
    assert!(!resolution.is_complete());
}

#[test]
fn parse_cli_args_defaults_to_debug_run_mode() {
    let action = parse_cli_args(std::iter::empty::<String>())
        .expect("empty args should run the debug sandbox");

    match action {
        CliAction::Run { eval, mode } => {
            assert!(eval.is_none());
            assert_eq!(mode, RunMode::Debug);
        }
        _ => panic!("expected run action"),
    }
}

#[test]
fn parse_cli_args_accepts_play_mode() {
    let action =
        parse_cli_args(["--play"].into_iter().map(str::to_string)).expect("play args should parse");

    match action {
        CliAction::Run { eval, mode } => {
            assert!(eval.is_none());
            assert_eq!(mode, RunMode::Play);
            assert!(!mode.debug_readout_enabled());
            assert!(!mode.debug_visuals_enabled());
        }
        _ => panic!("expected play run action"),
    }
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
fn parse_cli_args_accepts_wind_visual_export() {
    let action = parse_cli_args(
        ["--export-wind-visuals", "target/wind_visual_export"]
            .into_iter()
            .map(str::to_string),
    )
    .expect("wind visual export args should parse");

    match action {
        CliAction::ExportWindVisuals { output_dir } => {
            assert_eq!(output_dir, PathBuf::from("target/wind_visual_export"));
        }
        _ => panic!("expected wind visual export action"),
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
fn parse_cli_args_rejects_eval_and_wind_visual_export_together() {
    let error = parse_cli_args(
        [
            "--eval",
            "baseline_route",
            "--export-wind-visuals",
            "target/wind_visual_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("eval and wind visual export should be mutually exclusive");

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
            "--export-wind-visuals",
            "target/wind_visual_export",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .expect_err("export paths should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_play_and_eval_together() {
    let error = parse_cli_args(
        ["--play", "--eval", "baseline_route"]
            .into_iter()
            .map(str::to_string),
    )
    .expect_err("play and eval should be mutually exclusive");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_cli_args_rejects_play_and_export_together() {
    let error = parse_cli_args(
        ["--play", "--export-terrain", "target/terrain_export"]
            .into_iter()
            .map(str::to_string),
    )
    .expect_err("play and export should be mutually exclusive");

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
    assert!(manifest.contains(&format!(
        "\"impostor_color_bands\": {}",
        report.min_impostor_color_bands
    )));
    assert!(manifest.contains("\"impostor\": {\"obj\": \"islands/00_launch_mesa_impostor.obj\""));

    remove_existing_dir(&output_dir).expect("terrain export test dir should be removable");
}

#[test]
fn terrain_export_includes_great_sky_plateau_scale_and_region_evidence() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-terrain-export-plateau-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    remove_existing_dir(&output_dir).expect("stale terrain export dir should be removable");

    let report = export_terrain_inspection(&output_dir).expect("terrain export should succeed");
    let manifest = fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
    let plateau = report
        .islands
        .iter()
        .find(|island| island.island.name == "great sky plateau")
        .expect("terrain export should include the Great Sky Plateau island");
    let terrain_obj = output_dir.join(&plateau.terrain.obj_path);
    let regions = [
        IslandPlateauRegion::MeadowPlateau,
        IslandPlateauRegion::CliffRim,
        IslandPlateauRegion::HighShelf,
        IslandPlateauRegion::LowBasin,
        IslandPlateauRegion::BrokenEdge,
        IslandPlateauRegion::UnderhangEntry,
    ];

    assert_eq!(
        plateau.island.terrain_archetype,
        IslandTerrainArchetype::SkyPlateau
    );
    assert!(plateau.island.is_great_plateau_anchor());
    assert!(plateau.island.base_area_m2() >= 32_000.0);
    assert!(plateau.island.longest_span_m() >= 450.0);
    assert!(plateau.terrain.relief_range_m >= 0.8);
    assert!(plateau.terrain.height_bands >= ISLAND_TERRAIN_HEIGHT_BANDS);
    assert!(plateau.terrain.normal_slope_bands >= ISLAND_TERRAIN_NORMAL_SLOPE_BANDS);
    assert!(plateau.slug.contains("great_sky_plateau"));
    assert!(terrain_obj.exists());
    assert!(manifest.contains("\"name\": \"great sky plateau\""));
    assert!(manifest.contains("\"terrain_archetype\": \"sky_plateau\""));
    assert!(manifest.contains("great_sky_plateau_terrain.obj"));

    for region in regions {
        assert!(
            plateau.island.plateau_region_position(region).is_some(),
            "{region:?} should export from a playable plateau surface"
        );
    }

    remove_existing_dir(&output_dir).expect("terrain export test dir should be removable");
}

#[test]
fn great_sky_plateau_water_specs_span_basin_cliffs_and_falls() {
    let route = SkyRoute::default();
    let (plateau_index, plateau) = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, island)| island.is_great_plateau_anchor())
        .expect("route should include great sky plateau");
    let water_features = island_water_visual_specs(plateau_index, plateau);

    assert_eq!(
        water_features
            .iter()
            .filter(|feature| feature.kind == IslandWaterVisualKind::PondSurface)
            .count(),
        1
    );
    assert_eq!(
        water_features
            .iter()
            .filter(|feature| feature.kind == IslandWaterVisualKind::PlateauLakeSurface)
            .count(),
        2
    );
    assert_eq!(
        water_features
            .iter()
            .filter(|feature| feature.kind == IslandWaterVisualKind::PlateauWaterfallRibbon)
            .count(),
        2
    );
    assert_eq!(
        water_features
            .iter()
            .filter(|feature| feature.kind == IslandWaterVisualKind::PlateauWaterfallMist)
            .count(),
        2
    );

    let low_basin = plateau
        .plateau_region_position(IslandPlateauRegion::LowBasin)
        .expect("low basin should be playable");
    let high_shelf = plateau
        .plateau_region_position(IslandPlateauRegion::HighShelf)
        .expect("high shelf should be playable");
    let lake = water_features
        .iter()
        .find(|feature| feature.label == "low basin lake")
        .expect("plateau should place a low basin lake");
    let high_pool = water_features
        .iter()
        .find(|feature| feature.label == "high shelf pool")
        .expect("plateau should place a high shelf pool");
    assert!(lake.translation.distance(low_basin) < 0.2);
    assert!(high_pool.translation.distance(high_shelf) < 0.2);
    assert!(high_pool.translation.y - lake.translation.y >= 0.30);

    let rim = plateau
        .plateau_region_position(IslandPlateauRegion::CliffRim)
        .expect("rim should be playable");
    let waterfall = water_features
        .iter()
        .find(|feature| feature.label == "north rim waterfall")
        .expect("plateau should place a rim waterfall");
    let mist = water_features
        .iter()
        .find(|feature| feature.label == "north rim waterfall mist")
        .expect("plateau should place waterfall mist below the rim");
    assert!(rim.y - waterfall.translation.y >= 25.0);
    assert!(waterfall.translation.y - mist.translation.y >= 25.0);

    let lake_mesh = lake.build_mesh();
    let waterfall_mesh = waterfall.build_mesh();
    let mist_mesh = mist.build_mesh();
    assert!(lake_mesh.count_vertices() > LAKE_SURFACE_SEGMENTS * 3);
    assert!(mesh_y_range(&lake_mesh) >= 0.05);
    assert!(mesh_y_range(&waterfall_mesh) >= 58.0);
    assert!(waterfall_mesh.count_vertices() >= WATERFALL_RIBBON_COLUMNS * WATERFALL_RIBBON_ROWS);
    assert!(mist_mesh.count_vertices() >= WATERFALL_MIST_LOBES * 40);
}

#[test]
fn great_sky_plateau_under_route_visual_specs_mark_cave_and_shelf() {
    let route = SkyRoute::default();
    let (plateau_index, plateau) = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, island)| island.is_great_plateau_anchor())
        .expect("route should include great sky plateau");
    let under_route = plateau
        .under_route_segment()
        .expect("plateau should define an under-route");
    let cave_features = island_under_route_visual_specs(plateau_index, plateau);

    assert_eq!(cave_features.len(), 4);
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::CaveMouthArch)
            .count(),
        2
    );
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::UnderhangShelf)
            .count(),
        1
    );
    assert_eq!(
        cave_features
            .iter()
            .filter(|feature| feature.kind == IslandUnderRouteVisualKind::HangingRoots)
            .count(),
        1
    );

    let entry_arch = cave_features
        .iter()
        .find(|feature| feature.label == "underhang entry arch")
        .expect("entry arch should be generated");
    let shelf = cave_features
        .iter()
        .find(|feature| feature.label == "underside glide shelf")
        .expect("glide shelf should be generated");
    let roots = cave_features
        .iter()
        .find(|feature| feature.label == "hanging root curtain")
        .expect("hanging roots should be generated");
    let exit_arch = cave_features
        .iter()
        .find(|feature| feature.label == "updraft skylight exit arch")
        .expect("exit arch should be generated");

    assert!(entry_arch.translation.distance(under_route.entry) < 0.1);
    assert!(exit_arch.translation.distance(under_route.exit) < 0.1);
    assert!(shelf.translation.y < under_route.midpoint.y);
    assert!(roots.translation.y > under_route.midpoint.y);
    assert!(entry_arch.camera_half_extents.x >= under_route.clearance_radius_m);
    assert!(shelf.camera_half_extents.x > entry_arch.camera_half_extents.x);
    assert!(roots.camera_half_extents.y >= under_route.clearance_radius_m * 0.4);

    let arch_mesh = entry_arch.build_mesh();
    let shelf_mesh = shelf.build_mesh();
    let roots_mesh = roots.build_mesh();
    assert!(arch_mesh.count_vertices() >= CAVE_MOUTH_ARCH_STONES * 40);
    assert!(mesh_y_range(&arch_mesh) >= under_route.clearance_radius_m);
    assert_eq!(shelf_mesh.count_vertices(), UNDERHANG_SHELF_SEGMENTS * 2);
    assert!(mesh_y_range(&shelf_mesh) > 3.0);
    assert_eq!(
        roots_mesh.count_vertices(),
        HANGING_ROOT_STRANDS * (HANGING_ROOT_SEGMENTS + 1) * 4
    );
    assert!(mesh_y_range(&roots_mesh) >= under_route.clearance_radius_m * 0.7);
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
    let launch_spire = output_dir.join("visuals/00_launch_mesa_obstruction_spire.obj");
    let landing_marker = output_dir.join("visuals/02_landing_garden_landing_garden_marker_0.obj");
    let broken_stair_ruin = output_dir.join("visuals/12_broken_stair_ruin_arch.obj");
    let plateau_roots = output_dir.join("visuals/38_great_sky_plateau_hanging_root_curtain.obj");
    let route = SkyRoute::default();
    let island_count = route.islands().len();
    let small_island_count = route
        .islands()
        .iter()
        .filter(|island| {
            matches!(
                island.world_tags.scale_class,
                IslandScaleClass::Tiny | IslandScaleClass::Small
            )
        })
        .count();
    let ruin_arch_count = route
        .islands()
        .iter()
        .filter(|island| island.world_tags.landmark_role == IslandLandmarkRole::RuinArch)
        .count();
    let cliff_teeth_count = route
        .islands()
        .iter()
        .filter(|island| {
            matches!(
                island.terrain_archetype,
                IslandTerrainArchetype::StormRavine | IslandTerrainArchetype::StormShard
            )
        })
        .count();
    let garden_ring_count = route
        .islands()
        .iter()
        .filter(|island| {
            matches!(
                island.terrain_archetype,
                IslandTerrainArchetype::GardenBasin
                    | IslandTerrainArchetype::GardenApron
                    | IslandTerrainArchetype::OrchardBasin
                    | IslandTerrainArchetype::OrchardSpur
            )
        })
        .count();
    let lake_basin_count: usize = route
        .islands()
        .iter()
        .enumerate()
        .map(|(index, island)| island_lake_basin_visual_specs(index, *island).len())
        .sum();
    let route_lake_surface_count = route
        .islands()
        .iter()
        .filter(|island| island.world_tags.water_feature == IslandWaterFeature::LakeBasin)
        .count();
    let route_waterfall_source_count = route
        .islands()
        .iter()
        .filter(|island| island.world_tags.water_feature == IslandWaterFeature::WaterfallSource)
        .count();
    let route_waterfall_visual_count = route_waterfall_source_count * 2;
    let generated_tree_count = island_count * 3;
    let weather_veil_count = island_count.div_ceil(2) * 3;
    let route_cairn_count = island_count - 2;
    let plateau_extra_water_count = 6;
    let plateau_extra_cave_count = 4;
    let landmark_count = island_count * 2
        + route_cairn_count
        + 1
        + 4
        + plateau_extra_water_count
        + plateau_extra_cave_count
        + ruin_arch_count
        + cliff_teeth_count
        + garden_ring_count
        + lake_basin_count
        + route_lake_surface_count
        + route_waterfall_visual_count;

    assert_eq!(report.ground_cover_count, island_count);
    assert_eq!(
        report.ground_cover_patch_total,
        island_count * GROUND_COVER_PATCHES
    );
    assert_eq!(
        report.ground_cover_blade_total,
        island_count * GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH
    );
    assert_eq!(report.tree_trunk_count, generated_tree_count);
    assert_eq!(report.tree_canopy_count, generated_tree_count);
    assert_eq!(
        report.weather_cloud_count,
        island_count + weather_veil_count
    );
    assert_eq!(report.weather_cloud_bank_count, island_count);
    assert_eq!(report.weather_cloud_veil_count, weather_veil_count);
    assert_eq!(report.landmark_count, landmark_count);
    assert!(report.landmark_kind_count >= 18);
    assert_eq!(report.small_island_count, small_island_count);
    assert!(report.small_island_count >= 10);
    assert_eq!(report.plateau_landmark_count, 15);
    assert_eq!(report.plateau_waterfall_ribbon_count, 2);
    assert_eq!(report.plateau_waterfall_mist_count, 2);
    assert_eq!(
        report.route_waterfall_ribbon_count,
        route_waterfall_source_count
    );
    assert_eq!(
        report.route_waterfall_mist_count,
        route_waterfall_source_count
    );
    assert_eq!(report.route_lake_surface_count, route_lake_surface_count);
    assert_eq!(report.under_route_visual_count, plateau_extra_cave_count);
    assert_eq!(report.under_route_cave_mouth_count, 2);
    assert_eq!(report.ruin_arch_count, ruin_arch_count);
    assert_eq!(report.route_cairn_count, route_cairn_count);
    assert_eq!(report.launch_beacon_count, 1);
    assert_eq!(report.landing_garden_marker_count, 4);
    assert_eq!(report.pond_surface_count, island_count);
    assert_eq!(report.obstruction_spire_count, island_count);
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_lake_surface")
            .count(),
        2
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_waterfall_ribbon")
            .count(),
        2
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "plateau_waterfall_mist")
            .count(),
        2
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "route_waterfall_ribbon")
            .count(),
        route_waterfall_source_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "route_waterfall_mist")
            .count(),
        route_waterfall_source_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "route_lake_surface")
            .count(),
        route_lake_surface_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "under_route_cave_mouth")
            .count(),
        2
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "under_route_hanging_shelf")
            .count(),
        1
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "under_route_hanging_roots")
            .count(),
        1
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "ruin_arch")
            .count(),
        ruin_arch_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "cliff_teeth")
            .count(),
        cliff_teeth_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "garden_ring")
            .count(),
        garden_ring_count
    );
    assert_eq!(
        report
            .landmarks
            .iter()
            .filter(|summary| summary.kind == "lake_basin")
            .count(),
        lake_basin_count
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
    assert!(report.min_ground_cover_mesh_vertices >= 1320);
    assert!(report.min_ground_cover_blade_count >= 220);
    assert!(report.min_ground_cover_blade_height_range_m >= 0.7);
    assert!(report.min_tree_trunk_mesh_vertices >= 190);
    assert!(report.min_tree_trunk_taper_ratio >= 1.35);
    assert!(report.min_tree_branch_reach_ratio >= 1.8);
    assert!(report.min_tree_branch_count >= 4);
    assert!(report.min_tree_root_flare_count >= 5);
    assert!(report.min_tree_trunk_ring_count >= 5);
    assert!(report.tree_trunk_height_range_m >= 1.5);
    assert!(report.min_tree_canopy_mesh_vertices >= 450);
    assert!(report.min_tree_canopy_lobe_count >= 6);
    assert!(report.min_tree_canopy_detail_card_count >= 18);
    assert!(report.min_tree_canopy_vertical_to_horizontal_ratio >= 0.45);
    assert!(report.tree_canopy_radius_range_m >= 0.35);
    assert!(report.min_weather_cloud_mesh_vertices >= 2500);
    assert!(report.min_weather_cloud_lobe_count >= 12);
    assert!(report.min_weather_cloud_wisp_card_count >= 60);
    assert!(report.min_weather_cloud_filament_ribbon_detail_count >= 48);
    assert!(report.min_weather_cloud_bank_depth_m >= 6.4);
    assert!(report.min_weather_cloud_bank_lobe_count >= 22);
    assert!(report.min_weather_cloud_scaled_depth_span_m >= 14.0);
    assert!(report.min_route_cairn_mesh_vertices >= 240);
    assert!(report.min_route_cairn_vertical_span_m >= 3.0);
    assert!(report.min_launch_beacon_mesh_vertices >= 300);
    assert!(report.min_launch_beacon_vertical_span_m >= 2.8);
    assert!(report.min_landing_garden_marker_mesh_vertices >= 39);
    assert!(report.min_landing_garden_marker_vertical_span_m >= 0.12);
    assert!(report.min_pond_surface_mesh_vertices >= 65);
    assert!(report.min_pond_surface_vertical_span_m >= 0.015);
    assert!(report.plateau_landmark_vertex_total >= 2_500);
    assert!(report.max_plateau_landmark_mesh_vertices >= 600);
    assert!(report.min_plateau_waterfall_vertical_span_m >= 58.0);
    assert!(report.min_route_waterfall_vertical_span_m >= 24.0);
    assert!(report.min_route_lake_surface_horizontal_span_m >= 18.0);
    assert!(report.min_under_route_visual_vertical_span_m >= 4.0);
    assert!(report.min_ruin_arch_mesh_vertices >= 500);
    assert!(report.min_ruin_arch_vertical_span_m >= 4.5);
    assert!(report.min_ruin_arch_radius_band_count >= 8);
    assert!(report.min_ruin_arch_normal_slope_band_count >= 5);
    assert!(report.min_obstruction_spire_mesh_vertices >= 300);
    assert!(report.min_obstruction_spire_triangle_count >= 500);
    assert!(report.min_obstruction_spire_vertical_span_m >= 3.0);
    assert!(report.min_obstruction_spire_height_band_count >= 6);
    assert!(report.min_obstruction_spire_radius_band_count >= 5);
    assert!(report.min_obstruction_spire_normal_slope_band_count >= 5);
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
    assert!(launch_spire.exists());
    assert!(landing_marker.exists());
    assert!(broken_stair_ruin.exists());
    assert!(plateau_roots.exists());
    let low_basin_lake = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "low basin lake"
        })
        .expect("great sky plateau should export a low basin lake");
    let waterfall = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.kind == "plateau_waterfall_ribbon"
        })
        .expect("great sky plateau should export waterfall ribbons");
    let mist = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.kind == "plateau_waterfall_mist"
        })
        .expect("great sky plateau should export waterfall mist");
    let route_waterfall = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "cloudfall meadow" && summary.kind == "route_waterfall_ribbon"
        })
        .expect("waterfall-source islands should export route waterfall ribbons");
    let route_mist = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "cloudfall meadow" && summary.kind == "route_waterfall_mist"
        })
        .expect("waterfall-source islands should export route waterfall mist");
    let route_lake = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "sapphire basin" && summary.kind == "route_lake_surface"
        })
        .expect("lake-basin islands should export route lake surfaces");
    let bluevault_basin = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "bluevault basin" && summary.label == "route lake basin"
        })
        .expect("bluevault basin should export a terrain lake basin rim");
    let cave_arch = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "underhang entry arch"
        })
        .expect("great sky plateau should export an underhang entry arch");
    let underhang_shelf = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "underside glide shelf"
        })
        .expect("great sky plateau should export an underside glide shelf");
    let hanging_roots = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "hanging root curtain"
        })
        .expect("great sky plateau should export hanging roots");
    let ruin_arch = report
        .landmarks
        .iter()
        .find(|summary| summary.kind == "ruin_arch")
        .expect("ruin-tagged islands should export stacked stone arches");
    let cliff_teeth = report
        .landmarks
        .iter()
        .find(|summary| summary.kind == "cliff_teeth")
        .expect("storm islands should export jagged cliff teeth");
    let garden_ring = report
        .landmarks
        .iter()
        .find(|summary| summary.kind == "garden_ring")
        .expect("garden and orchard islands should export organic garden rings");
    let lake_basin = report
        .landmarks
        .iter()
        .find(|summary| {
            summary.island_name == "great sky plateau" && summary.label == "low basin lake basin"
        })
        .expect("great sky plateau should export a terrain lake basin rim");

    assert!(low_basin_lake.mesh.horizontal_span_m >= 100.0);
    assert!(low_basin_lake.mesh.depth_span_m >= 45.0);
    assert!(waterfall.mesh.vertical_span_m >= 58.0);
    assert!(waterfall.normal_slope_band_count >= 4);
    assert!(mist.mesh.horizontal_span_m >= 20.0);
    assert!(route_waterfall.mesh.vertical_span_m >= 24.0);
    assert!(route_waterfall.normal_slope_band_count >= 4);
    assert!(route_mist.mesh.vertical_span_m >= 1.8);
    assert!(route_lake.mesh.horizontal_span_m >= 18.0);
    assert!(route_lake.mesh.depth_span_m >= 9.0);
    assert!(bluevault_basin.mesh.horizontal_span_m >= 32.0);
    assert!(bluevault_basin.normal_slope_band_count >= 4);
    assert!(cave_arch.mesh.horizontal_span_m >= 20.0);
    assert!(cave_arch.mesh.vertical_span_m >= 14.0);
    assert!(underhang_shelf.mesh.horizontal_span_m >= 45.0);
    assert!(underhang_shelf.mesh.depth_span_m >= 24.0);
    assert!(hanging_roots.mesh.vertex_count >= 350);
    assert!(hanging_roots.mesh.vertical_span_m >= 8.0);
    assert!(hanging_roots.mesh.horizontal_span_m >= 20.0);
    assert!(ruin_arch.mesh.vertex_count >= 500);
    assert!(ruin_arch.mesh.vertical_span_m >= 4.5);
    assert!(ruin_arch.radius_band_count >= 8);
    assert!(
        cliff_teeth.mesh.vertex_count >= CLIFF_TOOTH_COUNT * CLIFF_TOOTH_TRIANGLES_PER_TOOTH * 3
    );
    assert!(cliff_teeth.mesh.vertical_span_m >= 4.0);
    assert!(cliff_teeth.mesh.horizontal_span_m >= 10.0);
    assert!(cliff_teeth.normal_slope_band_count >= 4);
    assert!(garden_ring.mesh.vertex_count >= (GARDEN_RING_SEGMENTS + 1) * GARDEN_RING_BANDS);
    assert!(garden_ring.mesh.horizontal_span_m >= 5.0);
    assert!(garden_ring.mesh.depth_span_m >= 5.0);
    assert!(garden_ring.mesh.vertical_span_m >= 0.16);
    assert!(garden_ring.normal_slope_band_count >= 3);
    assert!(lake_basin.mesh.vertex_count >= (LAKE_BASIN_RIM_SEGMENTS + 1) * LAKE_BASIN_RIM_BANDS);
    assert!(lake_basin.mesh.horizontal_span_m >= 120.0);
    assert!(lake_basin.mesh.depth_span_m >= 65.0);
    assert!(lake_basin.mesh.vertical_span_m >= 1.0);
    assert!(lake_basin.normal_slope_band_count >= 4);
    assert!(output_dir.join(&low_basin_lake.mesh.obj_path).exists());
    assert!(output_dir.join(&waterfall.mesh.obj_path).exists());
    assert!(output_dir.join(&mist.mesh.obj_path).exists());
    assert!(output_dir.join(&route_waterfall.mesh.obj_path).exists());
    assert!(output_dir.join(&route_mist.mesh.obj_path).exists());
    assert!(output_dir.join(&route_lake.mesh.obj_path).exists());
    assert!(output_dir.join(&bluevault_basin.mesh.obj_path).exists());
    assert!(output_dir.join(&cave_arch.mesh.obj_path).exists());
    assert!(output_dir.join(&underhang_shelf.mesh.obj_path).exists());
    assert!(output_dir.join(&hanging_roots.mesh.obj_path).exists());
    assert!(output_dir.join(&ruin_arch.mesh.obj_path).exists());
    assert!(output_dir.join(&cliff_teeth.mesh.obj_path).exists());
    assert!(output_dir.join(&garden_ring.mesh.obj_path).exists());
    assert!(output_dir.join(&lake_basin.mesh.obj_path).exists());
    assert!(manifest.contains("\"schema\": \"nau_visual_content_export.v1\""));
    assert!(manifest.contains("\"ground_cover_blade_height_range_m\""));
    assert!(manifest.contains("\"tree_branch_reach_ratio\""));
    assert!(manifest.contains("\"tree_root_flare_count\": 5"));
    assert!(manifest.contains("\"tree_trunk_ring_count\": 5"));
    assert!(manifest.contains("\"tree_trunk_height_range_m\""));
    assert!(manifest.contains("\"tree_canopy_radius_range_m\""));
    assert!(manifest.contains(&format!(
        "\"weather_cloud_veil_count\": {weather_veil_count}"
    )));
    assert!(manifest.contains("\"weather_cloud_scaled_depth_span_m\""));
    assert!(manifest.contains("\"weather_cloud_wisp_card_count\""));
    assert!(manifest.contains("\"weather_cloud_filament_ribbon_detail_count\""));
    assert!(manifest.contains(&format!("\"landmark_count\": {landmark_count}")));
    assert!(manifest.contains("\"landmark_kind_count\""));
    assert!(manifest.contains(&format!("\"small_island_count\": {small_island_count}")));
    assert!(manifest.contains("\"plateau_landmark_count\": 15"));
    assert!(manifest.contains("\"plateau_waterfall_ribbon_count\": 2"));
    assert!(manifest.contains("\"plateau_waterfall_mist_count\": 2"));
    assert!(manifest.contains(&format!(
        "\"route_waterfall_ribbon_count\": {route_waterfall_source_count}"
    )));
    assert!(manifest.contains(&format!(
        "\"route_waterfall_mist_count\": {route_waterfall_source_count}"
    )));
    assert!(manifest.contains(&format!(
        "\"route_lake_surface_count\": {route_lake_surface_count}"
    )));
    assert!(manifest.contains("\"under_route_visual_count\": 4"));
    assert!(manifest.contains("\"under_route_cave_mouth_count\": 2"));
    assert!(manifest.contains(&format!("\"ruin_arch_count\": {ruin_arch_count}")));
    assert!(manifest.contains(&format!("\"route_cairn_count\": {route_cairn_count}")));
    assert!(manifest.contains("\"launch_beacon_count\": 1"));
    assert!(manifest.contains("\"landing_garden_marker_count\": 4"));
    assert!(manifest.contains(&format!("\"pond_surface_count\": {island_count}")));
    assert!(manifest.contains(&format!("\"obstruction_spire_count\": {island_count}")));
    assert!(manifest.contains("\"route_cairn_vertical_span_m\""));
    assert!(manifest.contains("\"launch_beacon_vertical_span_m\""));
    assert!(manifest.contains("\"landing_garden_marker_vertical_span_m\""));
    assert!(manifest.contains("\"pond_surface_vertical_span_m\""));
    assert!(manifest.contains("\"plateau_landmark_vertex_total\""));
    assert!(manifest.contains("\"max_plateau_landmark_mesh_vertices\""));
    assert!(manifest.contains("\"plateau_waterfall_vertical_span_m\""));
    assert!(manifest.contains("\"route_waterfall_vertical_span_m\""));
    assert!(manifest.contains("\"route_lake_surface_horizontal_span_m\""));
    assert!(manifest.contains("\"under_route_visual_vertical_span_m\""));
    assert!(manifest.contains("\"ruin_arch_mesh_vertices\""));
    assert!(manifest.contains("\"ruin_arch_vertical_span_m\""));
    assert!(manifest.contains("\"ruin_arch_radius_band_count\""));
    assert!(manifest.contains("\"ruin_arch_normal_slope_band_count\""));
    assert!(manifest.contains("\"kind\": \"plateau_lake_surface\""));
    assert!(manifest.contains("\"kind\": \"plateau_waterfall_ribbon\""));
    assert!(manifest.contains("\"kind\": \"plateau_waterfall_mist\""));
    assert!(manifest.contains("\"kind\": \"route_waterfall_ribbon\""));
    assert!(manifest.contains("\"kind\": \"route_waterfall_mist\""));
    assert!(manifest.contains("\"kind\": \"route_lake_surface\""));
    assert!(manifest.contains("\"kind\": \"under_route_cave_mouth\""));
    assert!(manifest.contains("\"kind\": \"under_route_hanging_shelf\""));
    assert!(manifest.contains("\"kind\": \"under_route_hanging_roots\""));
    assert!(manifest.contains("\"kind\": \"ruin_arch\""));
    assert!(manifest.contains("\"kind\": \"cliff_teeth\""));
    assert!(manifest.contains("\"kind\": \"garden_ring\""));
    assert!(manifest.contains("\"kind\": \"lake_basin\""));
    assert!(manifest.contains("great_sky_plateau_low_basin_lake.obj"));
    assert!(manifest.contains("great_sky_plateau_north_rim_waterfall.obj"));
    assert!(manifest.contains("cloudfall_meadow_route_edge_waterfall.obj"));
    assert!(manifest.contains("cloudfall_meadow_route_edge_mist.obj"));
    assert!(manifest.contains("sapphire_basin_route_lake_surface.obj"));
    assert!(manifest.contains("bluevault_basin_route_lake_basin.obj"));
    assert!(manifest.contains("great_sky_plateau_underhang_entry_arch.obj"));
    assert!(manifest.contains("great_sky_plateau_underside_glide_shelf.obj"));
    assert!(manifest.contains("great_sky_plateau_hanging_root_curtain.obj"));
    assert!(manifest.contains("broken_stair_ruin_arch.obj"));
    assert!(manifest.contains("storm_porch_cliff_teeth.obj"));
    assert!(manifest.contains("landing_garden_garden_ring.obj"));
    assert!(manifest.contains("great_sky_plateau_low_basin_lake_basin.obj"));
    assert!(manifest.contains("\"obstruction_spire_height_band_count\""));
    assert!(manifest.contains("\"obstruction_spire_radius_band_count\""));
    assert!(manifest.contains("\"obstruction_spire_normal_slope_band_count\""));
    assert!(manifest.contains("\"terrain_biome_palette_count\": 5"));

    remove_existing_dir(&output_dir).expect("visual content export test dir should be removable");
}

#[test]
fn wind_visual_export_writes_motion_tracks_and_manifest() {
    let output_dir = std::env::temp_dir().join(format!(
        "nau-wind-visual-export-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    remove_existing_dir(&output_dir).expect("stale wind visual export dir should be removable");

    let report =
        export_wind_visual_inspection(&output_dir).expect("wind visual export should succeed");
    let manifest = fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
    let manifest_json: serde_json::Value =
        serde_json::from_str(&manifest).expect("manifest should be valid json");
    let track_obj = output_dir.join("wind_tracks/wind_visual_tracks.obj");
    let track_ndjson = output_dir.join("wind_tracks/wind_visual_tracks.ndjson");
    let track_lines = fs::read_to_string(&track_ndjson).expect("track ndjson should be readable");
    let updraft_field_count = nau_engine::environment::GAMEPLAY_LIFT_ROUTE.len();
    let crosswind_field_count = nau_engine::environment::VISUAL_CROSSWIND_FIELD_COUNT;
    let updraft_guide_count =
        updraft_field_count * UPDRAFT_GUIDE_RING_LEVELS.len() * UPDRAFT_GUIDES_PER_RING;
    let updraft_ribbon_count = updraft_field_count * UPDRAFT_RIBBONS_PER_FIELD;
    let crosswind_guide_count = crosswind_field_count * CROSSWIND_GUIDES_PER_FIELD;
    let crosswind_ribbon_count = crosswind_field_count * CROSSWIND_RIBBONS_PER_FIELD;
    let sample_window_count = manifest_json["sample_windows_secs"]
        .as_array()
        .expect("sample windows should be present")
        .len();
    let expected_tracks = (updraft_guide_count
        + updraft_ribbon_count * 4
        + crosswind_guide_count
        + crosswind_ribbon_count * 3)
        * sample_window_count;

    assert_eq!(report.track_count, expected_tracks);
    assert!(track_obj.exists());
    assert!(track_ndjson.exists());
    assert_eq!(track_lines.lines().count(), expected_tracks);
    assert!(track_lines.contains("\"family\":\"updraft_guide\""));
    assert!(track_lines.contains("\"family\":\"updraft_ribbon\""));
    assert!(track_lines.contains("\"family\":\"crosswind_guide\""));
    assert!(track_lines.contains("\"family\":\"crosswind_ribbon\""));
    assert!(manifest.contains("\"schema\": \"nau_wind_visual_export.v1\""));
    assert_eq!(
        manifest_json["counts"]["updraft_field_count"].as_u64(),
        Some(updraft_field_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["crosswind_field_count"].as_u64(),
        Some(crosswind_field_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["updraft_guide_count"].as_u64(),
        Some(updraft_guide_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["updraft_ribbon_count"].as_u64(),
        Some(updraft_ribbon_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["crosswind_guide_count"].as_u64(),
        Some(crosswind_guide_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["crosswind_ribbon_count"].as_u64(),
        Some(crosswind_ribbon_count as u64)
    );
    assert_eq!(
        manifest_json["counts"]["track_count"].as_u64(),
        Some(expected_tracks as u64)
    );
    let total_motion = &manifest_json["motion"]["total"];
    let static_track_count = total_motion["static_track_count"]
        .as_u64()
        .expect("static track count should be present");
    let off_field_track_count = total_motion["off_field_track_count"]
        .as_u64()
        .expect("off-field track count should be present");
    let low_alignment_track_count = total_motion["low_alignment_track_count"]
        .as_u64()
        .expect("low-alignment track count should be present");
    assert!(static_track_count * 50 <= expected_tracks as u64);
    assert!(off_field_track_count * 1000 <= expected_tracks as u64);
    assert!(low_alignment_track_count * 100 <= expected_tracks as u64 * 9);
    assert!(
        total_motion["coherent_track_count"]
            .as_u64()
            .expect("coherent track count should be present")
            >= expected_tracks as u64 * 17 / 20
    );

    remove_existing_dir(&output_dir).expect("wind visual export test dir should be removable");
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

    let coverage = audit_island_collision_coverage(&catalog, &route);
    assert!(
        coverage.passed,
        "island visual collision coverage should pass:\n{}",
        coverage.failures.join("\n")
    );
    assert!(coverage.checked_visual_count > 0);
    assert!(coverage.solid_visual_count >= 60);
    assert_eq!(
        coverage.terrain_rim_proxy_count,
        route.islands().len() * nau_engine::world::TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND
    );
    assert_eq!(
        coverage.terrain_body_proxy_count,
        route.islands().len() * nau_engine::world::TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND
    );
    assert!(coverage.camera_only_allowance_count >= route.islands().len());
    assert_eq!(
        catalog.named_obstacle_count("great sky plateau", "plateau cave mouth arch"),
        2
    );
    assert_eq!(
        catalog.named_obstacle_count("great sky plateau", "plateau underhang shelf"),
        1
    );
    assert_eq!(catalog.deferred_mesh_count(), route.islands().len() * 4);
    assert!(catalog.prebuilt_mesh_count() > catalog.deferred_mesh_count());

    let mut world = World::new();
    let stream_state;
    {
        let mut commands = world.commands();
        stream_state = spawn_initial_island_visuals(
            &mut commands,
            &mut meshes,
            &catalog,
            nau_engine::world::START_POSITION,
        );
    }
    world.flush();
    assert!(stream_state.loaded_mesh_count() > 0);
    assert!(stream_state.loaded_mesh_count() < catalog.deferred_mesh_count());

    let mut query = world.query::<&WorldCollisionProxy>();
    let proxies = query.iter(&world).copied().collect::<Vec<_>>();

    let terrain_rim_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::TerrainRim)
        .count();
    let terrain_body_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::TerrainBody)
        .count();
    let expected_spawned_near_island_count = route
        .islands()
        .iter()
        .filter(|island| {
            island
                .stream_activation(nau_engine::world::START_POSITION)
                .is_active()
                && island.lod_band(nau_engine::world::START_POSITION)
                    == nau_engine::world::LodBand::Near
        })
        .count();
    let expected_spawned_terrain_rim_proxy_count = expected_spawned_near_island_count
        * nau_engine::world::TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND;
    let expected_spawned_terrain_body_proxy_count = catalog.resident_collision_proxy_count(
        nau_engine::world::START_POSITION,
        WorldCollisionProxyKind::TerrainBody,
    );
    let expected_spawned_landmark_proxy_count = catalog.resident_collision_proxy_count(
        nau_engine::world::START_POSITION,
        WorldCollisionProxyKind::Landmark,
    );
    let tree_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::Tree)
        .count();
    let rock_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::Rock)
        .count();
    let landmark_proxy_count = proxies
        .iter()
        .filter(|proxy| proxy.kind == WorldCollisionProxyKind::Landmark)
        .count();
    let solid_proxy_count = tree_proxy_count + rock_proxy_count + landmark_proxy_count;

    assert!(proxies.len() >= 24);
    assert!(solid_proxy_count >= 60);
    assert!(tree_proxy_count >= 10);
    assert!(rock_proxy_count >= 12);
    assert!(landmark_proxy_count >= 24);
    assert_eq!(landmark_proxy_count, expected_spawned_landmark_proxy_count);
    assert_eq!(
        terrain_rim_proxy_count,
        expected_spawned_terrain_rim_proxy_count
    );
    assert_eq!(
        terrain_body_proxy_count,
        expected_spawned_terrain_body_proxy_count
    );
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
    assert!(
        proxies
            .iter()
            .filter(|proxy| {
                proxy.kind == WorldCollisionProxyKind::Landmark
                    && (proxy.half_extents.x > 3.0 || proxy.half_extents.z > 3.0)
            })
            .count()
            >= 4
    );
    assert!(
        proxies
            .iter()
            .filter(|proxy| {
                proxy.kind == WorldCollisionProxyKind::Landmark
                    && proxy.half_extents.y <= 0.4
                    && proxy.half_extents.x > 2.0
                    && proxy.half_extents.z > 1.0
            })
            .count()
            >= expected_spawned_near_island_count
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
    let secondary_lobe_vertices = 5 * ((4 + 1) * (8 + 1));
    let lobe_vertices = single_lobe_vertices + secondary_lobe_vertices;
    let expected_card_vertices = TREE_CANOPY_CARD_COUNT * DETAIL_CARD_VERTICES;
    let skirt_card_vertices = 6 * DETAIL_CARD_VERTICES;
    let skirt_start = lobe_vertices + expected_card_vertices - skirt_card_vertices;
    let skirt_positions = &positions[skirt_start..skirt_start + skirt_card_vertices];
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
    let avg_skirt_y = skirt_positions
        .iter()
        .map(|position| position[1])
        .sum::<f32>()
        / skirt_positions.len() as f32;
    let skirt_horizontal_span = skirt_positions
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);

    assert!(mesh.count_vertices() > single_lobe_vertices * 3);
    assert!(mesh.count_vertices() >= single_lobe_vertices + expected_card_vertices);
    assert!(mesh.count_vertices() >= 460);
    assert!(max_y - min_y > 1.9);
    assert!(horizontal_span > 1.45);
    assert!(avg_skirt_y < -0.18);
    assert!(skirt_horizontal_span > 1.0);
}

#[test]
fn cloud_cluster_mesh_uses_multiple_lobes_for_depth() {
    let mesh = cloud_cluster_mesh(99, CLOUD_BANK_LOBES);
    let positions = positions(&mesh);
    let colors = colors(&mesh);
    let lobe_vertices = (5 + 1) * (10 + 1);
    let card_vertices = CLOUD_WISP_CARDS_PER_LOBE * DETAIL_CARD_VERTICES;
    let filament_vertices = CLOUD_FILAMENT_RIBBONS_PER_LOBE * CLOUD_FILAMENT_RIBBON_VERTICES;
    let per_lobe_vertices = lobe_vertices + card_vertices + filament_vertices;
    let mut lower_depth_wisp_count = 0usize;
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

    assert_eq!(mesh.count_vertices(), CLOUD_BANK_LOBES * per_lobe_vertices);
    assert_eq!(colors.len(), positions.len());
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
    for lobe in 0..CLOUD_BANK_LOBES {
        let lobe_start = lobe * per_lobe_vertices + lobe_vertices;
        let lower_depth_start = lobe_start + (CLOUD_WISP_CARDS_PER_LOBE - 1) * DETAIL_CARD_VERTICES;
        let lower_depth_wisp =
            &positions[lower_depth_start..lower_depth_start + DETAIL_CARD_VERTICES];
        let upper_wisp = &positions[lobe_start..lower_depth_start];
        let lower_avg_y = lower_depth_wisp
            .iter()
            .map(|position| position[1])
            .sum::<f32>()
            / lower_depth_wisp.len() as f32;
        let upper_avg_y =
            upper_wisp.iter().map(|position| position[1]).sum::<f32>() / upper_wisp.len() as f32;

        if lower_avg_y + 0.05 < upper_avg_y {
            lower_depth_wisp_count += 1;
        }
    }
    assert!(
        lower_depth_wisp_count > CLOUD_BANK_LOBES * 3 / 4,
        "cloud clusters should add lower depth wisps under most lobes"
    );
}

#[test]
fn cloud_cluster_mesh_has_painterly_warm_and_cool_color_wash() {
    let mesh = cloud_cluster_mesh(123, CLOUD_BANK_LOBES);
    let colors = colors(&mesh);
    let min_alpha = colors
        .iter()
        .map(|color| color[3])
        .fold(f32::INFINITY, f32::min);
    let max_alpha = colors
        .iter()
        .map(|color| color[3])
        .fold(f32::NEG_INFINITY, f32::max);
    let warm_vertices = colors
        .iter()
        .filter(|color| color[0] > color[2] && color[1] > 0.72)
        .count();
    let cool_shadow_vertices = colors
        .iter()
        .filter(|color| color[2] >= color[0] && color[1] < 0.84)
        .count();

    assert!(
        mesh_vertex_color_band_count(&mesh) >= 8,
        "clouds should carry watercolor-like color variation instead of one flat material color"
    );
    assert!(
        max_alpha - min_alpha > 0.12,
        "cloud core and wisp geometry should have layered opacity"
    );
    assert!(
        warm_vertices > colors.len() / 5,
        "clouds should include warm sunlit wash vertices"
    );
    assert!(
        cool_shadow_vertices > colors.len() / 8,
        "clouds should include cooler shadowed vapor vertices"
    );
}

#[test]
fn ground_cover_mesh_uses_dense_curved_blades() {
    let mesh = island_ground_cover_mesh(2, test_island());
    let positions = positions(&mesh);
    let indices = u32_indices(&mesh);
    let blade_count = GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH;
    let mut side_leaf_count = 0usize;
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
    for (blade_index, blade) in positions
        .chunks_exact(VERTICES_PER_GROUND_BLADE)
        .enumerate()
    {
        let base_center = (Vec3::from_array(blade[0]) + Vec3::from_array(blade[1])) * 0.5;
        let mid_center = (Vec3::from_array(blade[2]) + Vec3::from_array(blade[3])) * 0.5;
        let tip = Vec3::from_array(blade[4]);
        let leaflet = Vec3::from_array(blade[5]);
        let main_axis = Vec2::new(tip.x - base_center.x, tip.z - base_center.z).normalize();
        let side_axis = Vec2::new(-main_axis.y, main_axis.x);
        let side_offset = Vec2::new(leaflet.x - mid_center.x, leaflet.z - mid_center.z)
            .dot(side_axis)
            .abs();

        if side_offset > 0.08 && leaflet.y > mid_center.y && leaflet.y < tip.y {
            side_leaf_count += 1;
        }

        let blade_vertex_start = blade_index * VERTICES_PER_GROUND_BLADE;
        let blade_vertex_end = blade_vertex_start + VERTICES_PER_GROUND_BLADE;
        let blade_index_start = blade_index * INDICES_PER_GROUND_BLADE;
        for index in &indices[blade_index_start..blade_index_start + INDICES_PER_GROUND_BLADE] {
            let index = *index as usize;
            assert!(
                index >= blade_vertex_start && index < blade_vertex_end,
                "ground cover indices should stay inside their blade chunk"
            );
        }
    }
    assert!(
        side_leaf_count > blade_count * 9 / 10,
        "ground cover should add side leaflets to most blades"
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

    let ruin_arch = ruin_arch_mesh(9.0, 6.2, 2.2, 15_789);
    let ruin_positions = positions(&ruin_arch);
    assert!(
        ruin_arch.count_vertices() >= RUIN_ARCH_STONE_COUNT * 50,
        "ruin arches should be made from separate stacked stones"
    );
    assert!(
        mesh_y_range(&ruin_arch) > 6.0,
        "ruin arches should form a readable vertical opening"
    );
    assert!(
        radial_range(ruin_positions) > 3.0,
        "ruin arches should have a broad broken-stone silhouette"
    );
    assert!(
        mesh_normal_slope_band_count(&ruin_arch) >= 5,
        "ruin arches should not collapse into one flat slab"
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

    let lake = lake_surface_mesh(18.0, 9.0, 31_789);
    let lake_positions = positions(&lake);
    let lake_indices = u32_indices(&lake);
    assert_eq!(lake.count_vertices(), 1 + LAKE_SURFACE_SEGMENTS * 3);
    assert_eq!(lake_indices.len(), LAKE_SURFACE_SEGMENTS * 15);
    assert!(
        radial_range(lake_positions) > 14.0,
        "large lakes should read as broad irregular basins"
    );
    assert!(
        mesh_y_range(&lake) > 0.05,
        "large lakes should carry stronger watercolor ripple relief than small ponds"
    );

    let waterfall = waterfall_ribbon_mesh(16.0, 60.0, 1.4, 33_789);
    let waterfall_indices = u32_indices(&waterfall);
    assert_eq!(
        waterfall.count_vertices(),
        WATERFALL_RIBBON_COLUMNS * WATERFALL_RIBBON_ROWS
    );
    assert_eq!(
        waterfall_indices.len(),
        (WATERFALL_RIBBON_COLUMNS - 1) * (WATERFALL_RIBBON_ROWS - 1) * 6
    );
    assert!(
        mesh_y_range(&waterfall) > 59.0,
        "waterfalls should be vertical traversal-scale ribbons"
    );
    assert!(
        mesh_normal_slope_band_count(&waterfall) >= 4,
        "waterfall ribbons should not be a single flat quad"
    );

    let mist = waterfall_mist_mesh(7.0, 5.0, 34_789);
    assert!(
        mist.count_vertices() > WATERFALL_MIST_LOBES * 40,
        "waterfall mist should be built from multiple soft lobes"
    );
    assert!(
        mesh_y_range(&mist) > 4.0,
        "mist should have visible depth instead of a flat decal"
    );

    let cave_arch = cave_mouth_arch_mesh(24.0, 18.0, 6.0, 41_789);
    assert!(
        cave_arch.count_vertices() >= CAVE_MOUTH_ARCH_STONES * 40,
        "cave mouths should be built from stacked arch stones, not one flat decal"
    );
    assert!(
        mesh_y_range(&cave_arch) > 14.0,
        "cave mouth arches should frame a readable flight opening"
    );

    let underhang_shelf = underhang_shelf_mesh(54.0, 30.0, 4.0, 42_789);
    let shelf_indices = u32_indices(&underhang_shelf);
    assert_eq!(
        underhang_shelf.count_vertices(),
        UNDERHANG_SHELF_SEGMENTS * 2
    );
    assert_eq!(
        shelf_indices.len(),
        (UNDERHANG_SHELF_SEGMENTS - 2) * 6 + UNDERHANG_SHELF_SEGMENTS * 6
    );
    assert!(
        radial_range(positions(&underhang_shelf)) > 10.0,
        "underhang shelves should have broad irregular ledge silhouettes"
    );

    let hanging_roots = hanging_root_curtain_mesh(32.0, 12.0, 8.0, 44_789);
    let hanging_root_indices = u32_indices(&hanging_roots);
    assert_eq!(
        hanging_roots.count_vertices(),
        HANGING_ROOT_STRANDS * (HANGING_ROOT_SEGMENTS + 1) * 4
    );
    assert_eq!(
        hanging_root_indices.len(),
        HANGING_ROOT_STRANDS * HANGING_ROOT_SEGMENTS * 24
    );
    assert!(
        mesh_y_range(&hanging_roots) > 9.0,
        "hanging roots should visibly descend from the cave ceiling"
    );
    assert!(
        radial_range(positions(&hanging_roots)) > 12.0,
        "hanging roots should form a broad organic curtain, not one strand"
    );

    let cliff_teeth = cliff_tooth_ridge_mesh(24.0, 8.0, 4.0, 45_789);
    let cliff_teeth_indices = u32_indices(&cliff_teeth);
    assert_eq!(
        cliff_teeth.count_vertices(),
        CLIFF_TOOTH_COUNT * CLIFF_TOOTH_TRIANGLES_PER_TOOTH * 3
    );
    assert_eq!(
        cliff_teeth_indices.len(),
        CLIFF_TOOTH_COUNT * CLIFF_TOOTH_TRIANGLES_PER_TOOTH * 3
    );
    assert!(
        mesh_y_range(&cliff_teeth) > 7.0,
        "cliff teeth should create sharp vertical silhouette peaks"
    );
    assert!(
        radial_range(positions(&cliff_teeth)) > 10.0,
        "cliff teeth should form a broad broken ridge, not one spike"
    );
    assert!(
        mesh_normal_slope_band_count(&cliff_teeth) >= 4,
        "cliff teeth should keep faceted fracture planes"
    );

    let garden_ring = garden_ring_mesh(6.0, 1.2, 0.6, 46_789);
    let garden_ring_positions = positions(&garden_ring);
    let garden_ring_indices = u32_indices(&garden_ring);
    let garden_min_x = garden_ring_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let garden_max_x = garden_ring_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        garden_ring.count_vertices(),
        (GARDEN_RING_SEGMENTS + 1) * GARDEN_RING_BANDS
    );
    assert_eq!(
        garden_ring_indices.len(),
        GARDEN_RING_SEGMENTS * (GARDEN_RING_BANDS - 1) * 6
    );
    assert!(
        garden_max_x - garden_min_x > 11.0,
        "garden rings should be broad circular landmarks, not short strips"
    );
    assert!(
        radial_range(garden_ring_positions) > 0.7,
        "garden rings should have measurable annular width"
    );
    assert!(
        mesh_y_range(&garden_ring) > 0.45,
        "garden rings should have low organic mound relief instead of a flat decal"
    );
    assert!(
        mesh_normal_slope_band_count(&garden_ring) >= 3,
        "garden rings should vary their soft ridge slopes"
    );

    let lake_basin = lake_basin_rim_mesh(24.0, 14.0, 2.4, 1.2, 47_789);
    let lake_basin_positions = positions(&lake_basin);
    let lake_basin_indices = u32_indices(&lake_basin);
    let lake_basin_min_x = lake_basin_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let lake_basin_max_x = lake_basin_positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let lake_basin_min_z = lake_basin_positions
        .iter()
        .map(|position| position[2])
        .fold(f32::INFINITY, f32::min);
    let lake_basin_max_z = lake_basin_positions
        .iter()
        .map(|position| position[2])
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        lake_basin.count_vertices(),
        (LAKE_BASIN_RIM_SEGMENTS + 1) * LAKE_BASIN_RIM_BANDS
    );
    assert_eq!(
        lake_basin_indices.len(),
        LAKE_BASIN_RIM_SEGMENTS * (LAKE_BASIN_RIM_BANDS - 1) * 6
    );
    assert!(
        lake_basin_max_x - lake_basin_min_x > 45.0,
        "lake basin rims should read as broad terrain-scale shorelines"
    );
    assert!(
        lake_basin_max_z - lake_basin_min_z > 25.0,
        "lake basin rims should preserve elliptical basin depth"
    );
    assert!(
        mesh_y_range(&lake_basin) > 0.9,
        "lake basin rims should have terraced shoreline relief"
    );
    assert!(
        mesh_normal_slope_band_count(&lake_basin) >= 4,
        "lake basin rims should include varied inner and outer slopes"
    );

    let spire = obstruction_spire_mesh(1.0, 5.2, 18_123);
    let spire_positions = positions(&spire);
    let spire_indices = u32_indices(&spire);
    assert_eq!(
        spire.count_vertices(),
        OBSTRUCTION_SPIRE_RINGS * OBSTRUCTION_SPIRE_SEGMENTS + 2 + OBSTRUCTION_SPIRE_RIB_COUNT * 8
    );
    assert_eq!(
        spire_indices.len(),
        OBSTRUCTION_SPIRE_SEGMENTS * 6
            + (OBSTRUCTION_SPIRE_RINGS - 1) * OBSTRUCTION_SPIRE_SEGMENTS * 6
            + OBSTRUCTION_SPIRE_RIB_COUNT * 12
    );
    assert!(
        mesh_y_range(&spire) > 5.0,
        "obstruction spires should read as tall terrain-integrated blockers"
    );
    assert!(
        radial_range(spire_positions) > 1.0,
        "obstruction spires should have roots/ribs instead of a box footprint"
    );
    assert!(mesh_vertical_band_count(&spire) >= 6);
    assert!(mesh_normal_slope_band_count(&spire) >= 5);
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
    let min_visual_radius = outer_ring
        .iter()
        .map(|position| normalized_visual_radius(island, *position))
        .fold(f32::INFINITY, f32::min);
    let max_visual_radius = outer_ring
        .iter()
        .map(|position| normalized_visual_radius(island, *position))
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(
        mesh.count_vertices(),
        1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS
    );
    assert!(
        min_visual_radius >= 0.999 && max_visual_radius <= 1.001,
        "terrain top ring must meet the visual cliff contour without a see-through rim gap"
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
fn terrain_and_cliff_top_rings_share_a_closed_visual_seam() {
    let island = test_island();
    let terrain_mesh = island_terrain_mesh(2, island);
    let cliff_mesh = island_cliff_mesh(2, island);
    let terrain_positions = positions(&terrain_mesh);
    let cliff_positions = positions(&cliff_mesh);
    let terrain_outer_start = 1 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS;

    for segment in 0..ISLAND_BODY_SEGMENTS {
        let terrain = Vec3::from_array(terrain_positions[terrain_outer_start + segment]);
        let cliff = Vec3::from_array(cliff_positions[segment]);
        assert!(
            terrain.distance(cliff) < 0.001,
            "terrain and cliff top rings should meet without a hollow rim seam"
        );
    }
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
