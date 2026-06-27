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
    accumulator.observe(content_metric_sample(
        scenario,
        0,
        MIN_SKY_ISLAND_COUNT,
        0,
        96,
    ));
    accumulator.observe(content_metric_sample(
        scenario,
        10,
        MIN_SKY_ISLAND_COUNT - 4,
        0,
        96,
    ));

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
    assert_eq!(procedural_check.value, (MIN_SKY_ISLAND_COUNT - 4) as f32);
}

#[test]
fn accumulator_fails_registered_primitive_or_low_silhouette_body_content() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(
        scenario,
        0,
        MIN_SKY_ISLAND_COUNT,
        1,
        48,
    ));
    accumulator.observe(
        content_metric_sample(scenario, 5, MIN_SKY_ISLAND_COUNT, 0, 96).with_content_metrics(
            MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            2305,
            61,
            0.8,
            MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
            9,
            MIN_SKY_ISLAND_COUNT,
            0,
            96,
            96.0,
            900,
            1633,
        ),
    );
    accumulator.observe(content_metric_sample(
        scenario,
        10,
        MIN_SKY_ISLAND_COUNT,
        0,
        96,
    ));

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
            .with_content_metrics(10, 1200, 2, 0.2, 3, 3, 12, 0, 96, 96.0, 1633, 1633)
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
    let archetype_check = named_check(&summary, "island_terrain_archetype_count");
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
    assert!(!archetype_check.passed);
    assert_eq!(archetype_check.value, 3.0);
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

    assert!(summary_json.contains("\"min_island_terrain_surface_count\": 20"));
    assert!(summary_json.contains("\"min_island_terrain_mesh_vertices\": 2305"));
    assert!(summary_json.contains("\"min_island_terrain_color_bands\": 61"));
    assert!(summary_json.contains("\"min_island_terrain_material_weight_bands\": 36"));
    assert!(summary_json.contains("\"min_island_terrain_material_channels\": 3"));
    assert!(summary_json.contains("\"min_island_terrain_material_regions\": 4"));
    assert!(summary_json.contains("\"min_island_terrain_texture_detail_bands\": 64"));
    assert!(summary_json.contains("\"min_island_terrain_relief_range_m\": 0.8000"));
    assert!(summary_json.contains("\"min_island_terrain_archetype_count\": 19"));
    assert!(summary_json.contains("\"min_island_cliff_color_bands\": 9"));
    assert!(summary_json.contains("\"min_island_body_mesh_vertices\": 1633"));
    assert!(summary_json.contains("\"min_generated_ground_cover_patch_count\": 800"));
    assert!(summary_json.contains("\"min_ground_cover_blade_count\": 220"));
    assert!(summary_json.contains("\"min_ground_cover_mesh_vertices\": 1100"));
    assert!(summary_json.contains("\"min_tree_canopy_mesh_vertices\": 412"));
    assert!(summary_json.contains("\"min_detail_biome_palette_count\": 5"));
    assert!(summary_json.contains("\"min_generated_rock_count\": 90"));
    assert!(summary_json.contains("\"min_rock_mesh_vertices\": 74"));
    assert!(summary_json.contains("\"min_generated_landmark_count\": 40"));
    assert!(summary_json.contains("\"min_generated_route_cairn_count\": 16"));
    assert!(summary_json.contains("\"min_generated_launch_beacon_count\": 1"));
    assert!(summary_json.contains("\"min_generated_landing_garden_marker_count\": 4"));
    assert!(summary_json.contains("\"min_generated_pond_surface_count\": 20"));
    assert!(summary_json.contains("\"min_landmark_mesh_vertices\": 39"));
    assert!(summary_json.contains("\"min_generated_weather_cloud_bank_count\": 20"));
    assert!(summary_json.contains("\"min_weather_cloud_bank_depth_m\": 6.2000"));
    assert!(summary_json.contains("\"min_weather_cloud_mesh_vertices\": 1458"));
    assert!(summary_json.contains("\"min_weather_cloud_filament_ribbon_detail_count\": 27"));
    assert!(summary_json.contains("\"max_updraft_guide_visual_count\": 126"));
    assert!(summary_json.contains("\"max_updraft_ribbon_visual_count\": 12"));
    assert!(summary_json.contains("\"max_crosswind_guide_visual_count\": 120"));
    assert!(summary_json.contains("\"max_crosswind_ribbon_visual_count\": 14"));
    assert!(summary_json.contains("\"max_updraft_visual_rise_m\": 0.4500"));
    assert!(summary_json.contains("\"max_updraft_visual_swirl_displacement_m\": 0.3500"));
    assert!(summary_json.contains("\"max_updraft_visual_depth_span_m\": 48.0000"));
    assert!(summary_json.contains("\"max_updraft_visual_scale_pulse\": 0.0600"));
    assert!(summary_json.contains("\"max_crosswind_visual_motion_m\": 0.6500"));
    assert!(summary_json.contains("\"max_crosswind_guide_flow_displacement_m\": 0.6500"));
    assert!(summary_json.contains("\"max_crosswind_ribbon_flow_displacement_m\": 0.6500"));
    assert!(summary_json.contains("\"max_crosswind_visual_lane_depth_span_m\": 30.0000"));
    assert!(summary_json.contains("\"max_crosswind_visual_scale_pulse\": 0.1000"));
    assert!(summary_json.contains("\"max_updraft_flow_coherent_visual_count\": 108"));
    assert!(summary_json.contains("\"max_crosswind_flow_coherent_visual_count\": 100"));
    assert!(summary_json.contains("\"max_updraft_visual_flow_alignment\": 0.5500"));
    assert!(summary_json.contains("\"max_crosswind_visual_flow_alignment\": 0.5500"));
    assert!(summary_json.contains("\"wind_force_samples\": 1"));
    assert!(summary_json.contains("\"meaningful_wind_force_samples\": 1"));
    assert!(summary_json.contains("\"max_active_wind_force_fields\": 1"));
    assert!(summary_json.contains("\"max_crosswind_force_fields\": 1"));
    assert!(summary_json.contains("\"max_updraft_swirl_force_fields\": 1"));
    assert!(summary_json.contains("\"max_wind_force_delta_mps\": 0.0400"));
    assert!(summary_json.contains("\"max_crosswind_force_delta_mps\": 0.0400"));
    assert!(summary_json.contains("\"max_updraft_swirl_force_delta_mps\": 0.0300"));
    assert!(summary_json.contains("\"max_wind_force_flow_speed_mps\": 6.0000"));
    assert!(summary_json.contains("\"max_wind_force_variation\": 0.1200"));
    assert!(summary_json.contains("\"max_world_collision_proxy_count\": 24"));
    assert!(summary_json.contains("\"max_terrain_rim_collision_proxy_count\": 4"));
    assert!(summary_json.contains("\"max_solid_world_collision_proxy_count\": 60"));
    assert!(summary_json.contains("\"max_tree_world_collision_proxy_count\": 10"));
    assert!(summary_json.contains("\"max_rock_world_collision_proxy_count\": 16"));
    assert!(summary_json.contains("\"max_landmark_world_collision_proxy_count\": 40"));
}

#[test]
fn accumulator_fails_when_world_collision_proxies_are_missing() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96).with_world_collision_metrics(0, 0, 0.0),
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
    let collision_check = named_check(&summary, "world_collision_proxy_count");

    assert!(!collision_check.passed);
    assert_eq!(collision_check.value, 0.0);
}

#[test]
fn accumulator_fails_when_terrain_rim_collision_proxies_are_missing() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96).with_terrain_rim_collision_metrics(0, 0, 0.0),
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
    let collision_check = named_check(&summary, "terrain_rim_collision_proxy_count");

    assert!(!collision_check.passed);
    assert_eq!(collision_check.value, 0.0);
}

#[test]
fn accumulator_fails_when_solid_world_collision_proxy_kinds_are_missing() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96)
            .with_world_collision_metrics(MIN_WORLD_COLLISION_PROXY_COUNT, 0, 0.0)
            .with_terrain_rim_collision_metrics(MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT, 0, 0.0)
            .with_world_collision_kind_metrics(0, 0, 0, 0),
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

    for check_name in [
        "solid_world_collision_proxy_count",
        "tree_world_collision_proxy_count",
        "rock_world_collision_proxy_count",
        "landmark_world_collision_proxy_count",
    ] {
        assert!(!named_check(&summary, check_name).passed);
    }
}

#[test]
fn collision_contact_eval_fails_without_asset_contact_resolution() {
    let scenario = scenario_named(WORLD_COLLISION_CONTACT).expect("collision route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96).with_world_collision_metrics(
            MIN_WORLD_COLLISION_PROXY_COUNT,
            0,
            0.0,
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
    let contact_check = named_check(&summary, "world_collision_contact_samples");
    let push_check = named_check(&summary, "world_collision_push");

    assert!(!contact_check.passed);
    assert_eq!(contact_check.value, 0.0);
    assert!(!push_check.passed);
    assert_eq!(push_check.value, 0.0);
}

#[test]
fn collision_contact_eval_fails_when_contact_samples_are_only_skin_depth() {
    let scenario = scenario_named(WORLD_COLLISION_CONTACT).expect("collision route exists");
    let mut accumulator = EvalAccumulator::default();
    for frame in 0..MIN_WORLD_COLLISION_CONTACT_SAMPLES {
        accumulator.observe(
            content_metric_sample(scenario, frame, 12, 0, 96).with_world_collision_metrics(
                MIN_WORLD_COLLISION_PROXY_COUNT,
                1,
                MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M * 0.5,
            ),
        );
    }
    accumulator.observe(
        content_metric_sample(scenario, 99, 12, 0, 96).with_world_collision_metrics(
            MIN_WORLD_COLLISION_PROXY_COUNT,
            0,
            MIN_WORLD_COLLISION_CONTACT_PUSH_M,
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
    let contact_check = named_check(&summary, "world_collision_contact_samples");
    let push_check = named_check(&summary, "world_collision_push");

    assert!(!contact_check.passed);
    assert_eq!(contact_check.value, 0.0);
    assert!(push_check.passed);
}

#[test]
fn collision_contact_eval_passes_when_asset_contact_resolves_with_meaningful_push() {
    let scenario = scenario_named(WORLD_COLLISION_CONTACT).expect("collision route exists");
    let mut accumulator = EvalAccumulator::default();
    for frame in 0..MIN_WORLD_COLLISION_CONTACT_SAMPLES {
        let push_m = if frame == 0 {
            MIN_WORLD_COLLISION_CONTACT_PUSH_M
        } else {
            MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M
        };
        accumulator.observe(
            content_metric_sample(scenario, frame, 12, 0, 96).with_world_collision_metrics(
                MIN_WORLD_COLLISION_PROXY_COUNT,
                1,
                push_m,
            ),
        );
    }

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
    let contact_check = named_check(&summary, "world_collision_contact_samples");
    let push_check = named_check(&summary, "world_collision_push");

    assert!(contact_check.passed);
    assert_eq!(
        contact_check.value,
        MIN_WORLD_COLLISION_CONTACT_SAMPLES as f32
    );
    assert!(push_check.passed);
    assert_eq!(push_check.value, MIN_WORLD_COLLISION_CONTACT_PUSH_M);
}

#[test]
fn terrain_rim_contact_eval_fails_without_rim_contact_resolution() {
    let scenario = scenario_named(TERRAIN_RIM_COLLISION_CONTACT).expect("rim route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96).with_terrain_rim_collision_metrics(
            MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT,
            0,
            0.0,
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
    let contact_check = named_check(&summary, "terrain_rim_collision_contact_samples");
    let push_check = named_check(&summary, "terrain_rim_collision_push");

    assert!(!contact_check.passed);
    assert_eq!(contact_check.value, 0.0);
    assert!(!push_check.passed);
    assert_eq!(push_check.value, 0.0);
}

#[test]
fn terrain_rim_contact_eval_fails_when_contact_samples_are_only_skin_depth() {
    let scenario = scenario_named(TERRAIN_RIM_COLLISION_CONTACT).expect("rim route exists");
    let mut accumulator = EvalAccumulator::default();
    for frame in 0..MIN_WORLD_COLLISION_CONTACT_SAMPLES {
        accumulator.observe(
            content_metric_sample(scenario, frame, 12, 0, 96).with_terrain_rim_collision_metrics(
                MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT,
                1,
                MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M * 0.5,
            ),
        );
    }
    accumulator.observe(
        content_metric_sample(scenario, 99, 12, 0, 96).with_terrain_rim_collision_metrics(
            MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT,
            0,
            MIN_WORLD_COLLISION_CONTACT_PUSH_M,
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
    let contact_check = named_check(&summary, "terrain_rim_collision_contact_samples");
    let push_check = named_check(&summary, "terrain_rim_collision_push");

    assert!(!contact_check.passed);
    assert_eq!(contact_check.value, 0.0);
    assert!(push_check.passed);
}

#[test]
fn terrain_rim_contact_eval_passes_when_rim_contact_resolves_with_meaningful_push() {
    let scenario = scenario_named(TERRAIN_RIM_COLLISION_CONTACT).expect("rim route exists");
    let mut accumulator = EvalAccumulator::default();
    for frame in 0..MIN_WORLD_COLLISION_CONTACT_SAMPLES {
        let push_m = if frame == 0 {
            MIN_WORLD_COLLISION_CONTACT_PUSH_M
        } else {
            MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M
        };
        accumulator.observe(
            content_metric_sample(scenario, frame, 12, 0, 96).with_terrain_rim_collision_metrics(
                MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT,
                1,
                push_m,
            ),
        );
    }

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
    let contact_check = named_check(&summary, "terrain_rim_collision_contact_samples");
    let push_check = named_check(&summary, "terrain_rim_collision_push");

    assert!(contact_check.passed);
    assert_eq!(
        contact_check.value,
        MIN_WORLD_COLLISION_CONTACT_SAMPLES as f32
    );
    assert!(push_check.passed);
    assert_eq!(push_check.value, MIN_WORLD_COLLISION_CONTACT_PUSH_M);
}

#[test]
fn ground_taxi_control_fails_when_terrain_rim_contacts_are_recorded() {
    let scenario = scenario_named(GROUND_TAXI_CONTROL).expect("ground taxi route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96).with_terrain_rim_collision_metrics(
            MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT,
            1,
            MIN_WORLD_COLLISION_CONTACT_PUSH_M,
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
    let contact_check = named_check(&summary, "ground_taxi_terrain_rim_contact_samples");

    assert!(!contact_check.passed);
    assert_eq!(contact_check.value, 1.0);
}

#[test]
fn sample_json_emits_wind_guide_visual_metrics() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let sample_json = content_metric_sample(scenario, 0, 12, 0, 96).to_json();

    assert!(sample_json.contains("\"updraft_guide_visual_count\":126"));
    assert!(sample_json.contains("\"updraft_ribbon_visual_count\":12"));
    assert!(sample_json.contains("\"crosswind_guide_visual_count\":120"));
    assert!(sample_json.contains("\"crosswind_ribbon_visual_count\":14"));
    assert!(sample_json.contains("\"max_updraft_visual_motion_m\":0.4500"));
    assert!(sample_json.contains("\"max_updraft_visual_rise_m\":0.4500"));
    assert!(sample_json.contains("\"max_updraft_visual_swirl_displacement_m\":0.3500"));
    assert!(sample_json.contains("\"max_updraft_visual_depth_span_m\":48.0000"));
    assert!(sample_json.contains("\"max_updraft_visual_scale_pulse\":0.0600"));
    assert!(sample_json.contains("\"max_crosswind_visual_motion_m\":0.6500"));
    assert!(sample_json.contains("\"max_crosswind_guide_flow_displacement_m\":0.6500"));
    assert!(sample_json.contains("\"max_crosswind_ribbon_flow_displacement_m\":0.6500"));
    assert!(sample_json.contains("\"max_crosswind_visual_lane_depth_span_m\":30.0000"));
    assert!(sample_json.contains("\"max_crosswind_visual_scale_pulse\":0.1000"));
    assert!(sample_json.contains("\"updraft_flow_coherent_visual_count\":108"));
    assert!(sample_json.contains("\"crosswind_flow_coherent_visual_count\":100"));
    assert!(sample_json.contains("\"max_updraft_visual_flow_alignment\":0.5500"));
    assert!(sample_json.contains("\"max_crosswind_visual_flow_alignment\":0.5500"));
    assert!(sample_json.contains("\"active_wind_force_fields\":1"));
    assert!(sample_json.contains("\"crosswind_force_fields\":1"));
    assert!(sample_json.contains("\"updraft_swirl_force_fields\":1"));
    assert!(sample_json.contains("\"max_wind_force_delta_mps\":0.0400"));
    assert!(sample_json.contains("\"max_crosswind_force_delta_mps\":0.0400"));
    assert!(sample_json.contains("\"max_updraft_swirl_force_delta_mps\":0.0300"));
    assert!(sample_json.contains("\"max_wind_force_flow_speed_mps\":6.0000"));
    assert!(sample_json.contains("\"max_wind_force_variation\":0.1200"));
    assert!(sample_json.contains("\"terrain_rim_collision_proxy_count\":4"));
    assert!(sample_json.contains("\"solid_world_collision_proxy_count\":60"));
    assert!(sample_json.contains("\"tree_world_collision_proxy_count\":10"));
    assert!(sample_json.contains("\"rock_world_collision_proxy_count\":16"));
    assert!(sample_json.contains("\"landmark_world_collision_proxy_count\":40"));
    assert!(sample_json.contains("\"terrain_rim_collision_resolved_count\":0"));
    assert!(sample_json.contains("\"max_terrain_rim_collision_push_m\":0.0000"));
    assert!(sample_json.contains("\"island_terrain_archetype_count\":19"));
}

#[test]
fn accumulator_fails_when_wind_guide_visuals_are_missing_or_static() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96)
            .with_wind_guide_visual_metrics(0, 0, 0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
            .with_wind_guide_depth_metrics(0.0, 0.0, 0.0, 0.0),
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

    for check_name in [
        "updraft_guide_visual_count",
        "updraft_ribbon_visual_count",
        "crosswind_guide_visual_count",
        "crosswind_ribbon_visual_count",
        "updraft_visual_motion",
        "updraft_visual_rise",
        "updraft_visual_swirl_displacement",
        "updraft_visual_depth_span",
        "updraft_visual_scale_pulse",
        "crosswind_visual_motion",
        "crosswind_guide_flow_displacement",
        "crosswind_ribbon_flow_displacement",
        "crosswind_visual_lane_depth_span",
        "crosswind_visual_scale_pulse",
    ] {
        let check = named_check(&summary, check_name);
        assert!(!check.passed, "{check_name} should fail");
        assert_eq!(check.value, 0.0);
    }
}

#[test]
fn accumulator_fails_generated_visual_shape_regression() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96).with_generated_visual_shape_metrics(
            528, 220, 1100, 12, 12, 62, 316, 5, 48, 74, 27, 10, 1, 4, 12, 39, 12, 12, 6.2, 9, 18,
            1458, 27,
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
            "grounded_idle",
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
            0.0,
            0.0,
            0,
            0,
            1,
            140.0,
            false,
            objective,
            MIN_SKY_ISLAND_COUNT,
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
            "gliding",
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
            0.0,
            0.0,
            0,
            0,
            1,
            0.0,
            false,
            objective,
            MIN_SKY_ISLAND_COUNT,
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
                "gliding",
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
                0.0,
                0.0,
                0,
                0,
                1,
                140.0 - frame as f32 * 4.0,
                false,
                objective,
                MIN_SKY_ISLAND_COUNT,
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
            )
            .with_wind_guide_visual_metrics(
                MIN_UPDRAFT_GUIDE_VISUAL_COUNT,
                MIN_UPDRAFT_RIBBON_VISUAL_COUNT,
                MIN_CROSSWIND_GUIDE_VISUAL_COUNT,
                MIN_CROSSWIND_RIBBON_VISUAL_COUNT,
                MIN_UPDRAFT_VISUAL_MOTION_M,
                MIN_UPDRAFT_VISUAL_RISE_M,
                MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M,
                MIN_CROSSWIND_VISUAL_MOTION_M,
                MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M,
                MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M,
            )
            .with_wind_guide_depth_metrics(
                MIN_UPDRAFT_VISUAL_DEPTH_SPAN_M,
                MIN_UPDRAFT_VISUAL_SCALE_PULSE,
                MIN_CROSSWIND_VISUAL_LANE_DEPTH_SPAN_M,
                MIN_CROSSWIND_VISUAL_SCALE_PULSE,
            )
            .with_wind_guide_flow_coherence_metrics(
                MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT,
                MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT,
                MIN_WIND_VISUAL_FLOW_ALIGNMENT,
                MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            )
            .with_wind_force_metrics(
                1,
                1,
                1,
                MIN_WIND_FORCE_DELTA_MPS,
                MIN_CROSSWIND_FORCE_DELTA_MPS,
                MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
                MIN_WIND_FORCE_FLOW_SPEED_MPS,
                MIN_WIND_FORCE_VARIATION,
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
            .with_content_metrics(
                MIN_ISLAND_TERRAIN_SURFACE_COUNT,
                2305,
                61,
                0.8,
                MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
                9,
                MIN_SKY_ISLAND_COUNT,
                0,
                96,
                96.0,
                1633,
                1633,
            )
            .with_island_impostor_metrics(146, 24)
            .with_terrain_material_metrics(36, 3, 4, 64)
            .with_generated_visual_shape_metrics(
                MIN_GENERATED_GROUND_COVER_PATCH_COUNT,
                220,
                1100,
                MIN_GENERATED_TREE_TRUNK_COUNT,
                MIN_GENERATED_TREE_CANOPY_COUNT,
                196,
                412,
                MIN_DETAIL_BIOME_PALETTE_COUNT,
                MIN_GENERATED_ROCK_COUNT,
                74,
                MIN_GENERATED_LANDMARK_COUNT,
                MIN_GENERATED_ROUTE_CAIRN_COUNT,
                MIN_GENERATED_LAUNCH_BEACON_COUNT,
                MIN_GENERATED_LANDING_GARDEN_MARKER_COUNT,
                MIN_GENERATED_POND_SURFACE_COUNT,
                39,
                MIN_GENERATED_WEATHER_CLOUD_COUNT,
                MIN_GENERATED_WEATHER_CLOUD_BANK_COUNT,
                6.2,
                9,
                18,
                1458,
                27,
            )
            .with_world_collision_metrics(MIN_WORLD_COLLISION_PROXY_COUNT, 0, 0.0)
            .with_terrain_rim_collision_metrics(MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT, 0, 0.0)
            .with_world_collision_kind_metrics(
                MIN_SOLID_WORLD_COLLISION_PROXY_COUNT,
                MIN_TREE_WORLD_COLLISION_PROXY_COUNT,
                MIN_ROCK_WORLD_COLLISION_PROXY_COUNT,
                MIN_LANDMARK_WORLD_COLLISION_PROXY_COUNT,
            )
            .with_visible_authored_world_fixture_count(MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT),
    );
}
