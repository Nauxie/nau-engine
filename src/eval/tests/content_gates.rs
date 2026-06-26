use super::*;

#[test]
fn accumulator_gates_authored_asset_readiness() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut sample = content_metric_sample(scenario, 0, 12, 0, 96);
    sample.ready_visual_asset_slot_count = 0;
    sample.placeholder_visual_asset_slot_count = VISUAL_ASSET_SLOT_COUNT;
    sample.missing_visual_asset_slot_count = VISUAL_ASSET_SLOT_COUNT;
    sample.deferred_visual_asset_scene_count = 1;
    sample.queued_visual_asset_scene_count = 0;
    sample.loaded_visual_asset_scene_count = 0;
    sample.dependency_loaded_visual_asset_scene_count = 0;
    sample.preload_ready_visual_asset_scene_count = 0;
    sample.spawned_visual_asset_scene_count = 0;
    sample.ready_visual_asset_scene_count = 0;
    sample.visible_authored_world_fixture_count = 0;
    sample.always_preload_ready_visual_asset_slot_count = 0;
    sample.streaming_preload_ready_visual_asset_slot_count = 0;
    sample.ready_animation_clip_count = 0;
    sample.animation_player_count = 0;
    sample.animation_graph_count = 0;

    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(sample);
    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    for check_name in [
        "ready_visual_asset_slot_count",
        "missing_visual_asset_slot_count",
        "deferred_visual_asset_scene_count",
        "loaded_visual_asset_scene_count",
        "dependency_loaded_visual_asset_scene_count",
        "preload_ready_visual_asset_scene_count",
        "always_preload_ready_visual_asset_slot_count",
        "streaming_preload_ready_visual_asset_slot_count",
        "spawned_visual_asset_scene_count",
        "ready_visual_asset_scene_count",
        "visible_authored_world_fixture_count",
        "ready_animation_clip_count",
        "animation_player_count",
        "animation_graph_count",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without a loaded authored scene"
        );
    }
}

#[test]
fn accumulator_fails_when_procedural_body_count_disappears_after_startup() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
    accumulator.observe(content_metric_sample(scenario, 10, 8, 0, 96));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let procedural_check = named_check(&summary, "procedural_island_body_count");

    assert!(!procedural_check.passed);
    assert_eq!(procedural_check.value, 8.0);
}

#[test]
fn accumulator_fails_registered_primitive_or_low_silhouette_body_content() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 1, 48));
    accumulator.observe(
        content_metric_sample(scenario, 5, 12, 0, 96)
            .with_content_metrics(12, 2305, 61, 0.8, 9, 12, 0, 96, 96.0, 900, 1633),
    );
    accumulator.observe(content_metric_sample(scenario, 10, 12, 0, 96));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let primitive_check = named_check(&summary, "primitive_island_body_count");
    let silhouette_check = named_check(&summary, "island_body_silhouette_segments");
    let mesh_check = named_check(&summary, "island_body_mesh_vertices");

    assert!(!primitive_check.passed);
    assert_eq!(primitive_check.value, 1.0);
    assert!(!silhouette_check.passed);
    assert_eq!(silhouette_check.value, 48.0);
    assert!(!mesh_check.passed);
    assert_eq!(mesh_check.value, 900.0);
}

#[test]
fn accumulator_fails_low_detail_island_impostors() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
    accumulator.observe(
        content_metric_sample(scenario, 10, 12, 0, 96).with_island_impostor_metrics(42, 4),
    );

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let mesh_check = named_check(&summary, "island_impostor_mesh_vertices");
    let color_check = named_check(&summary, "island_impostor_color_bands");

    assert!(!mesh_check.passed);
    assert_eq!(mesh_check.value, 42.0);
    assert!(!color_check.passed);
    assert_eq!(color_check.value, 4.0);
    assert!(
        summary
            .to_json()
            .contains("\"min_island_impostor_mesh_vertices\"")
    );
}

#[test]
fn accumulator_fails_terrain_detail_regression() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
    accumulator.observe(
        content_metric_sample(scenario, 10, 12, 0, 96)
            .with_content_metrics(10, 1200, 2, 0.2, 3, 12, 0, 96, 96.0, 1633, 1633)
            .with_terrain_material_metrics(4, 2, 2, 16),
    );

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let terrain_count_check = named_check(&summary, "island_terrain_surface_count");
    let terrain_vertex_check = named_check(&summary, "island_terrain_mesh_vertices");
    let terrain_color_check = named_check(&summary, "island_terrain_color_bands");
    let material_band_check = named_check(&summary, "island_terrain_material_weight_bands");
    let material_channel_check = named_check(&summary, "island_terrain_material_channels");
    let material_region_check = named_check(&summary, "island_terrain_material_regions");
    let texture_detail_check = named_check(&summary, "island_terrain_texture_detail_bands");
    let relief_check = named_check(&summary, "island_terrain_relief_range");
    let cliff_color_check = named_check(&summary, "island_cliff_color_bands");

    assert!(!terrain_count_check.passed);
    assert_eq!(terrain_count_check.value, 10.0);
    assert!(!terrain_vertex_check.passed);
    assert_eq!(terrain_vertex_check.value, 1200.0);
    assert!(!terrain_color_check.passed);
    assert_eq!(terrain_color_check.value, 2.0);
    assert!(!material_band_check.passed);
    assert_eq!(material_band_check.value, 4.0);
    assert!(!material_channel_check.passed);
    assert_eq!(material_channel_check.value, 2.0);
    assert!(!material_region_check.passed);
    assert_eq!(material_region_check.value, 2.0);
    assert!(!texture_detail_check.passed);
    assert_eq!(texture_detail_check.value, 16.0);
    assert!(!relief_check.passed);
    assert_eq!(relief_check.value, 0.2);
    assert!(!cliff_color_check.passed);
    assert_eq!(cliff_color_check.value, 3.0);
}

#[test]
fn summary_json_exposes_terrain_detail_thresholds() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let summary_json = summary.to_json();

    assert!(summary_json.contains("\"min_island_terrain_surface_count\": 12"));
    assert!(summary_json.contains("\"min_island_terrain_mesh_vertices\": 2305"));
    assert!(summary_json.contains("\"min_island_terrain_color_bands\": 61"));
    assert!(summary_json.contains("\"min_island_terrain_material_weight_bands\": 36"));
    assert!(summary_json.contains("\"min_island_terrain_material_channels\": 3"));
    assert!(summary_json.contains("\"min_island_terrain_material_regions\": 4"));
    assert!(summary_json.contains("\"min_island_terrain_texture_detail_bands\": 64"));
    assert!(summary_json.contains("\"min_island_terrain_relief_range_m\": 0.8000"));
    assert!(summary_json.contains("\"min_island_cliff_color_bands\": 9"));
    assert!(summary_json.contains("\"min_island_body_mesh_vertices\": 1633"));
    assert!(summary_json.contains("\"min_generated_ground_cover_patch_count\": 528"));
    assert!(summary_json.contains("\"min_ground_cover_blade_count\": 220"));
    assert!(summary_json.contains("\"min_ground_cover_mesh_vertices\": 1100"));
    assert!(summary_json.contains("\"min_tree_canopy_mesh_vertices\": 412"));
    assert!(summary_json.contains("\"min_detail_biome_palette_count\": 5"));
    assert!(summary_json.contains("\"min_generated_rock_count\": 60"));
    assert!(summary_json.contains("\"min_rock_mesh_vertices\": 74"));
    assert!(summary_json.contains("\"min_generated_landmark_count\": 27"));
    assert!(summary_json.contains("\"min_generated_route_cairn_count\": 10"));
    assert!(summary_json.contains("\"min_generated_launch_beacon_count\": 1"));
    assert!(summary_json.contains("\"min_generated_landing_garden_marker_count\": 4"));
    assert!(summary_json.contains("\"min_generated_pond_surface_count\": 12"));
    assert!(summary_json.contains("\"min_landmark_mesh_vertices\": 39"));
    assert!(summary_json.contains("\"min_generated_weather_cloud_bank_count\": 12"));
    assert!(summary_json.contains("\"min_weather_cloud_bank_depth_m\": 4.8000"));
    assert!(summary_json.contains("\"min_weather_cloud_mesh_vertices\": 910"));
    assert!(summary_json.contains("\"min_weather_cloud_filament_ribbon_detail_count\": 14"));
}

#[test]
fn accumulator_fails_generated_visual_shape_regression() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96).with_generated_visual_shape_metrics(
            528, 220, 1100, 12, 12, 62, 316, 5, 48, 74, 27, 10, 1, 4, 12, 39, 12, 12, 4.8, 6, 10,
            910, 14,
        ),
    );
    accumulator.observe(
        content_metric_sample(scenario, 10, 12, 0, 96).with_generated_visual_shape_metrics(
            10, 12, 60, 0, 0, 8, 45, 1, 1, 12, 0, 0, 0, 0, 0, 8, 0, 0, 0.4, 1, 1, 45, 0,
        ),
    );

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let tree_count_check = named_check(&summary, "generated_tree_trunk_count");
    let ground_patch_check = named_check(&summary, "generated_ground_cover_patch_count");
    let ground_blade_check = named_check(&summary, "ground_cover_blade_count");
    let ground_vertex_check = named_check(&summary, "ground_cover_mesh_vertices");
    let canopy_vertex_check = named_check(&summary, "tree_canopy_mesh_vertices");
    let detail_palette_check = named_check(&summary, "detail_biome_palette_count");
    let rock_count_check = named_check(&summary, "generated_rock_count");
    let rock_vertex_check = named_check(&summary, "rock_mesh_vertices");
    let landmark_count_check = named_check(&summary, "generated_landmark_count");
    let route_cairn_count_check = named_check(&summary, "generated_route_cairn_count");
    let launch_beacon_count_check = named_check(&summary, "generated_launch_beacon_count");
    let landing_garden_marker_count_check =
        named_check(&summary, "generated_landing_garden_marker_count");
    let pond_surface_count_check = named_check(&summary, "generated_pond_surface_count");
    let landmark_vertex_check = named_check(&summary, "landmark_mesh_vertices");
    let cloud_lobe_check = named_check(&summary, "weather_cloud_lobe_count");
    let cloud_bank_lobe_check = named_check(&summary, "weather_cloud_bank_lobe_count");
    let cloud_mesh_check = named_check(&summary, "weather_cloud_mesh_vertices");
    let cloud_filament_check = named_check(&summary, "weather_cloud_filament_ribbon_detail_count");
    let cloud_bank_count_check = named_check(&summary, "generated_weather_cloud_bank_count");
    let cloud_bank_depth_check = named_check(&summary, "weather_cloud_bank_depth");

    assert!(!ground_patch_check.passed);
    assert_eq!(ground_patch_check.value, 10.0);
    assert!(!ground_blade_check.passed);
    assert_eq!(ground_blade_check.value, 12.0);
    assert!(!ground_vertex_check.passed);
    assert_eq!(ground_vertex_check.value, 60.0);
    assert!(!tree_count_check.passed);
    assert_eq!(tree_count_check.value, 0.0);
    assert!(!canopy_vertex_check.passed);
    assert_eq!(canopy_vertex_check.value, 45.0);
    assert!(!detail_palette_check.passed);
    assert_eq!(detail_palette_check.value, 1.0);
    assert!(!rock_count_check.passed);
    assert_eq!(rock_count_check.value, 1.0);
    assert!(!rock_vertex_check.passed);
    assert_eq!(rock_vertex_check.value, 12.0);
    assert!(!landmark_count_check.passed);
    assert_eq!(landmark_count_check.value, 0.0);
    assert!(!route_cairn_count_check.passed);
    assert_eq!(route_cairn_count_check.value, 0.0);
    assert!(!launch_beacon_count_check.passed);
    assert_eq!(launch_beacon_count_check.value, 0.0);
    assert!(!landing_garden_marker_count_check.passed);
    assert_eq!(landing_garden_marker_count_check.value, 0.0);
    assert!(!pond_surface_count_check.passed);
    assert_eq!(pond_surface_count_check.value, 0.0);
    assert!(!landmark_vertex_check.passed);
    assert_eq!(landmark_vertex_check.value, 8.0);
    assert!(!cloud_lobe_check.passed);
    assert_eq!(cloud_lobe_check.value, 1.0);
    assert!(!cloud_bank_lobe_check.passed);
    assert_eq!(cloud_bank_lobe_check.value, 1.0);
    assert!(!cloud_mesh_check.passed);
    assert_eq!(cloud_mesh_check.value, 45.0);
    assert!(!cloud_filament_check.passed);
    assert_eq!(cloud_filament_check.value, 0.0);
    assert!(!cloud_bank_count_check.passed);
    assert_eq!(cloud_bank_count_check.value, 0.0);
    assert!(!cloud_bank_depth_check.passed);
    assert_eq!(cloud_bank_depth_check.value, 0.4);
}

#[test]
fn accumulator_marks_current_baseline_shape_as_passing() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    let objective = EvalObjectiveProgress::new(0, 2, "near route updraft", 140.0, false);

    observe_current_content(
        &mut accumulator,
        EvalSample::new(
            0,
            scenario.fixed_dt,
            Vec3::new(0.0, 1.2, 0.0),
            Vec3::ZERO,
            FlightMode::Grounded,
            12.0,
            3.0,
            4.0,
            -20.0,
            0.0,
            0.0,
            0.2,
            2.0,
            0.0,
            0.0,
            0.0,
            0,
            0,
            3,
            0,
            0,
            1,
            140.0,
            false,
            objective,
            12,
            25,
            6,
            2,
            4,
            6,
            24,
            36,
            8,
            4,
            26,
            118,
            16,
            12,
            8,
            0.08,
            160,
            0,
            12,
            12,
            335,
            175,
            0.48,
            0,
            0,
            12,
            12,
            20,
            20,
            130,
            VISUAL_ASSET_SLOT_COUNT,
            GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            MIN_READY_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            STREAMING_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
            MIN_READY_VISUAL_ASSET_SCENE_COUNT,
            ALWAYS_VISUAL_ASSET_SLOT_COUNT,
            STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
            NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            WEATHER_VISUAL_ASSET_SLOT_COUNT,
            MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_VISUAL_ANIMATION_PLAYER_COUNT,
            MIN_VISUAL_ANIMATION_GRAPH_COUNT,
            AERIAL_POWER_UP_ROUTE.len(),
            AERIAL_POWER_UP_ROUTE.len(),
            0,
            0,
            0,
        ),
    );
    observe_current_content(
        &mut accumulator,
        EvalSample::new(
            scenario.frame_count,
            scenario.fixed_dt,
            Vec3::new(0.0, 32.0, 140.0),
            Vec3::new(0.0, -4.0, 30.0),
            FlightMode::Gliding,
            14.0,
            3.0,
            4.0,
            -18.0,
            0.0,
            0.0,
            0.2,
            2.0,
            0.0,
            0.0,
            0.0,
            0,
            0,
            3,
            0,
            0,
            1,
            0.0,
            false,
            objective,
            12,
            25,
            6,
            2,
            4,
            6,
            24,
            36,
            8,
            4,
            26,
            118,
            16,
            12,
            8,
            0.08,
            160,
            0,
            12,
            12,
            335,
            175,
            0.48,
            0,
            0,
            12,
            12,
            20,
            20,
            130,
            VISUAL_ASSET_SLOT_COUNT,
            GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            MIN_READY_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            STREAMING_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
            MIN_READY_VISUAL_ASSET_SCENE_COUNT,
            ALWAYS_VISUAL_ASSET_SLOT_COUNT,
            STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
            NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            WEATHER_VISUAL_ASSET_SLOT_COUNT,
            MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_VISUAL_ANIMATION_PLAYER_COUNT,
            MIN_VISUAL_ANIMATION_GRAPH_COUNT,
            AERIAL_POWER_UP_ROUTE.len(),
            AERIAL_POWER_UP_ROUTE.len(),
            0,
            0,
            0,
        ),
    );
    for frame in 1..=scenario.thresholds.min_gliding_samples {
        observe_current_content(
            &mut accumulator,
            EvalSample::new(
                frame,
                scenario.fixed_dt,
                Vec3::new(0.0, 24.0, frame as f32 * 4.0),
                Vec3::new(0.0, -3.0, 25.0),
                FlightMode::Gliding,
                13.0,
                3.0,
                4.0,
                -18.0,
                0.0,
                0.0,
                0.2,
                2.0,
                0.0,
                0.0,
                0.0,
                0,
                0,
                3,
                0,
                0,
                1,
                140.0 - frame as f32 * 4.0,
                false,
                objective,
                12,
                25,
                6,
                2,
                4,
                6,
                24,
                36,
                8,
                4,
                26,
                118,
                16,
                12,
                8,
                0.08,
                160,
                0,
                12,
                12,
                335,
                175,
                0.48,
                0,
                0,
                12,
                12,
                20,
                20,
                130,
                VISUAL_ASSET_SLOT_COUNT,
                GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                MIN_READY_VISUAL_ASSET_SLOT_COUNT,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                STREAMING_VISUAL_ASSET_SLOT_COUNT,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                0,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
                MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
                0,
                MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
                MIN_READY_VISUAL_ASSET_SCENE_COUNT,
                ALWAYS_VISUAL_ASSET_SLOT_COUNT,
                STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
                NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                WEATHER_VISUAL_ASSET_SLOT_COUNT,
                MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
                MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
                MIN_VISUAL_ANIMATION_PLAYER_COUNT,
                MIN_VISUAL_ANIMATION_GRAPH_COUNT,
                AERIAL_POWER_UP_ROUTE.len(),
                AERIAL_POWER_UP_ROUTE.len(),
                0,
                0,
                0,
            ),
        );
    }

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: vec!["checkpoints/glide_midroute.png".to_string()],
            checkpoint_marker_metadata: vec!["checkpoints/glide_midroute.markers.json".to_string()],
        },
    );

    assert!(summary.passed);
    assert_eq!(summary.metrics.objective_total_count, 2);
    assert_eq!(summary.metrics.max_completed_objective_count, 0);
    assert!(summary.to_json().contains("\"passed\": true"));
    assert!(summary.to_json().contains("\"objective\":"));
    assert!(
        summary
            .to_json()
            .contains("\"checkpoint_screenshots\": [\"checkpoints/glide_midroute.png\"]")
    );
    assert!(
        summary.to_json().contains(
            "\"checkpoint_marker_metadata\": [\"checkpoints/glide_midroute.markers.json\"]"
        )
    );
}

fn observe_current_content(accumulator: &mut EvalAccumulator, sample: EvalSample) {
    accumulator.observe(
        sample
            .with_content_metrics(12, 2305, 61, 0.8, 9, 12, 0, 96, 96.0, 1633, 1633)
            .with_island_impostor_metrics(146, 24)
            .with_terrain_material_metrics(36, 3, 4, 64)
            .with_generated_visual_shape_metrics(
                528, 220, 1100, 37, 37, 196, 412, 5, 60, 74, 27, 10, 1, 4, 12, 39, 30, 12, 4.8, 7,
                14, 910, 14,
            )
            .with_visible_authored_world_fixture_count(MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT),
    );
}
