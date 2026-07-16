use crate::{
    checkpoint::{audit_checkpoint_path, audit_scene_sample, terrain_material_variant_for_label},
    materials::{material_matches, sample_pixel_hits, sample_pixel_hits_with_variant},
    report::{json_number, json_string, report_checks, report_json},
    thresholds::{
        MIN_DISTANT_ISLAND_PIXEL_COVERAGE, MIN_FOLIAGE_PIXEL_COVERAGE,
        MIN_PASSED_TERRAIN_MATERIAL_VARIANTS, MIN_SAMPLE_PIXEL_HITS,
        MIN_TERRAIN_MATERIAL_VARIANT_PIXEL_COVERAGE, MIN_TERRAIN_PIXEL_COVERAGE,
        MIN_VISIBLE_TERRAIN_MATERIAL_VARIANTS, MIN_WIND_PIXEL_COVERAGE_PER_VISIBLE_SAMPLE,
    },
    types::{CheckpointAudit, SceneSampleAudit},
};
use image::{Rgb, RgbImage};
use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

fn sample(expected_material: &str, x: f64, y: f64) -> Value {
    serde_json::json!({
        "kind": "test_sample",
        "label": expected_material,
        "expected_material": expected_material,
        "in_viewport": true,
        "screen": {"x": x, "y": y}
    })
}

#[test]
fn sample_pixel_hits_classifies_expected_scene_materials() {
    let mut image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    image.put_pixel(16, 16, Rgb([104, 82, 48]));
    image.put_pixel(17, 16, Rgb([92, 74, 46]));
    image.put_pixel(18, 16, Rgb([74, 68, 62]));
    image.put_pixel(16, 32, Rgb([44, 126, 32]));
    image.put_pixel(17, 32, Rgb([48, 132, 36]));
    image.put_pixel(18, 32, Rgb([52, 138, 34]));
    image.put_pixel(16, 48, Rgb([158, 166, 174]));
    image.put_pixel(17, 48, Rgb([166, 174, 184]));
    image.put_pixel(18, 48, Rgb([148, 158, 168]));

    assert!(sample_pixel_hits(&image, 17.0, 16.0, "terrain", (1.0, 1.0)) >= MIN_SAMPLE_PIXEL_HITS);
    assert!(sample_pixel_hits(&image, 17.0, 32.0, "foliage", (1.0, 1.0)) >= MIN_SAMPLE_PIXEL_HITS);
    assert!(sample_pixel_hits(&image, 17.0, 48.0, "cloud", (1.0, 1.0)) >= MIN_SAMPLE_PIXEL_HITS);
    assert!(
        sample_pixel_hits(&image, 17.0, 16.0, "distant_island", (1.0, 1.0))
            >= MIN_SAMPLE_PIXEL_HITS
    );
}

#[test]
fn sample_pixel_hits_classifies_terrain_material_variants() {
    let mut image = RgbImage::from_pixel(320, 32, Rgb([130, 170, 220]));
    for (x, color) in [
        (24, Rgb([54, 128, 70])),
        (25, Rgb([34, 100, 62])),
        (26, Rgb([70, 150, 94])),
        (84, Rgb([96, 138, 70])),
        (85, Rgb([128, 154, 78])),
        (86, Rgb([166, 172, 90])),
        (144, Rgb([126, 104, 76])),
        (145, Rgb([106, 82, 62])),
        (146, Rgb([162, 138, 96])),
        (204, Rgb([52, 110, 118])),
        (205, Rgb([76, 130, 132])),
        (206, Rgb([142, 176, 164])),
        (264, Rgb([132, 132, 92])),
        (265, Rgb([112, 122, 82])),
        (266, Rgb([178, 166, 112])),
    ] {
        image.put_pixel(x, 16, color);
    }

    assert!(
        sample_pixel_hits_with_variant(
            &image,
            25.0,
            16.0,
            "terrain",
            "terrain_lush_meadow",
            (1.0, 1.0)
        ) >= MIN_SAMPLE_PIXEL_HITS
    );
    assert_eq!(
        sample_pixel_hits_with_variant(
            &image,
            25.0,
            16.0,
            "terrain",
            "terrain_copper_clay",
            (1.0, 1.0)
        ),
        0
    );
    assert!(
        sample_pixel_hits_with_variant(
            &image,
            145.0,
            16.0,
            "terrain",
            "terrain_copper_clay",
            (1.0, 1.0)
        ) >= MIN_SAMPLE_PIXEL_HITS
    );
    assert_eq!(
        sample_pixel_hits_with_variant(
            &image,
            145.0,
            16.0,
            "terrain",
            "terrain_alpine_mist",
            (1.0, 1.0)
        ),
        0
    );
    assert!(
        sample_pixel_hits_with_variant(
            &image,
            205.0,
            16.0,
            "terrain",
            "terrain_alpine_mist",
            (1.0, 1.0)
        ) >= MIN_SAMPLE_PIXEL_HITS
    );
    assert!(
        sample_pixel_hits_with_variant(
            &image,
            265.0,
            16.0,
            "terrain",
            "terrain_highland_grass",
            (1.0, 1.0)
        ) >= MIN_SAMPLE_PIXEL_HITS
    );
    assert!(
        sample_pixel_hits_with_variant(
            &image,
            85.0,
            16.0,
            "terrain",
            "terrain_gold_meadow",
            (1.0, 1.0)
        ) >= MIN_SAMPLE_PIXEL_HITS
    );
}

#[test]
fn sample_pixel_hits_classifies_wind_pixels() {
    let mut image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    image.put_pixel(32, 32, Rgb([62, 198, 244]));
    image.put_pixel(33, 32, Rgb([112, 235, 255]));
    image.put_pixel(34, 32, Rgb([20, 118, 184]));

    assert!(sample_pixel_hits(&image, 33.0, 32.0, "wind", (1.0, 1.0)) >= MIN_SAMPLE_PIXEL_HITS);
}

#[test]
fn wind_classifier_rejects_sky_and_cloud_colors() {
    assert!(!material_matches("wind", 130.0, 170.0, 220.0));
    assert!(!material_matches("wind", 158.0, 166.0, 174.0));
    assert!(!material_matches("wind", 219.0, 232.0, 245.0));

    let image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    assert_eq!(sample_pixel_hits(&image, 32.0, 32.0, "wind", (1.0, 1.0)), 0);
}

#[test]
fn checkpoint_requires_projected_scene_sample_hits() {
    let image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    let audit = audit_scene_sample(&sample("terrain", 32.0, 32.0), &image, (1.0, 1.0))
        .expect("sample should parse");

    assert!(audit.in_viewport);
    assert_eq!(audit.material_variant, "terrain_unknown");
    assert!(!audit.passed);
}

#[test]
fn audit_scene_sample_parses_explicit_material_variant() {
    let mut image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    image.put_pixel(32, 32, Rgb([54, 128, 70]));
    image.put_pixel(33, 32, Rgb([34, 100, 62]));
    image.put_pixel(34, 32, Rgb([70, 150, 94]));
    let sample = serde_json::json!({
        "kind": "terrain_surface",
        "label": "launch mesa",
        "expected_material": "terrain",
        "material_variant": "terrain_lush_meadow",
        "in_viewport": true,
        "visibility": "visible",
        "screen": {"x": 32.0, "y": 32.0}
    });

    let audit = audit_scene_sample(&sample, &image, (1.0, 1.0)).expect("sample should parse");

    assert_eq!(audit.material_variant, "terrain_lush_meadow");
    assert!(audit.passed);
}

#[test]
fn audit_scene_sample_requires_terrain_pixels_to_match_material_variant() {
    let mut image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    image.put_pixel(32, 32, Rgb([54, 128, 70]));
    image.put_pixel(33, 32, Rgb([34, 100, 62]));
    image.put_pixel(34, 32, Rgb([70, 150, 94]));
    let sample = serde_json::json!({
        "kind": "terrain_surface",
        "label": "landing garden",
        "expected_material": "terrain",
        "material_variant": "terrain_copper_clay",
        "in_viewport": true,
        "visibility": "visible",
        "screen": {"x": 32.0, "y": 32.0}
    });

    let audit = audit_scene_sample(&sample, &image, (1.0, 1.0)).expect("sample should parse");

    assert_eq!(audit.material_variant, "terrain_copper_clay");
    assert_eq!(audit.semantic_pixel_hits, 0);
    assert!(!audit.passed);
}

#[test]
fn audit_scene_sample_derives_legacy_terrain_variant_from_label() {
    let mut image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    image.put_pixel(32, 32, Rgb([52, 110, 118]));
    image.put_pixel(33, 32, Rgb([76, 130, 132]));
    image.put_pixel(34, 32, Rgb([142, 176, 164]));
    let sample = serde_json::json!({
        "kind": "terrain_surface",
        "label": "storm porch",
        "expected_material": "terrain",
        "in_viewport": true,
        "visibility": "visible",
        "screen": {"x": 32.0, "y": 32.0}
    });

    let audit = audit_scene_sample(&sample, &image, (1.0, 1.0)).expect("sample should parse");

    assert_eq!(
        terrain_material_variant_for_label("storm porch"),
        Some("terrain_alpine_mist")
    );
    assert_eq!(audit.material_variant, "terrain_alpine_mist");
    assert!(audit.passed);
}

#[test]
fn occluded_projected_scene_samples_do_not_count_as_hits() {
    let mut image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    image.put_pixel(32, 32, Rgb([104, 82, 48]));
    image.put_pixel(33, 32, Rgb([92, 74, 46]));
    image.put_pixel(34, 32, Rgb([74, 68, 62]));
    let occluded_sample = serde_json::json!({
        "kind": "terrain_surface",
        "label": "blocked terrain",
        "expected_material": "terrain",
        "in_viewport": true,
        "visibility": "occluded",
        "screen": {"x": 32.0, "y": 32.0}
    });

    let audit =
        audit_scene_sample(&occluded_sample, &image, (1.0, 1.0)).expect("sample should parse");

    assert!(audit.in_viewport);
    assert!(!audit.is_visible());
    assert_eq!(audit.semantic_pixel_hits, 0);
    assert!(!audit.passed);
}

#[test]
fn report_checks_require_visible_material_samples_before_pixel_hits() {
    let visible_terrain = SceneSampleAudit {
        kind: "terrain_surface".to_string(),
        label: "foreground".to_string(),
        expected_material: "terrain".to_string(),
        material_variant: "terrain_lush_meadow".to_string(),
        in_viewport: true,
        visibility: "visible".to_string(),
        screen_x: Some(12.0),
        screen_y: Some(12.0),
        semantic_pixel_hits: MIN_SAMPLE_PIXEL_HITS,
        passed: true,
    };
    let occluded_cloud = SceneSampleAudit {
        kind: "weather_cloud".to_string(),
        label: "blocked cloud".to_string(),
        expected_material: "cloud".to_string(),
        material_variant: "cloud".to_string(),
        in_viewport: true,
        visibility: "occluded".to_string(),
        screen_x: Some(24.0),
        screen_y: Some(24.0),
        semantic_pixel_hits: MIN_SAMPLE_PIXEL_HITS,
        passed: false,
    };
    let checkpoint = CheckpointAudit {
        metadata_path: "checkpoint.markers.json".to_string(),
        screenshot_path: "checkpoint.png".to_string(),
        scenario: "default".to_string(),
        checkpoint: "test".to_string(),
        in_viewport_scene_sample_count: 2,
        occluded_scene_sample_count: 1,
        visible_scene_sample_count: 1,
        scene_sample_pixel_hit_count: 1,
        visible_scene_material_count: 1,
        scene_material_pixel_hit_count: 1,
        visible_scene_sample_kind_count: 1,
        scene_sample_kind_pixel_hit_count: 1,
        visible_terrain_material_variant_count: 1,
        terrain_material_variant_pixel_hit_count: 1,
        passed: false,
        samples: vec![visible_terrain, occluded_cloud],
        materials: Vec::new(),
    };

    let checks = report_checks(&[checkpoint]);
    let cloud_visible = checks
        .iter()
        .find(|check| check.name == "cloud_visible_scene_samples")
        .expect("cloud visible check");
    let cloud_hits = checks
        .iter()
        .find(|check| check.name == "cloud_scene_sample_pixel_hits")
        .expect("cloud hit check");

    assert!(!cloud_visible.passed);
    assert_eq!(cloud_visible.value, 0.0);
    assert!(!cloud_hits.passed);
    assert_eq!(cloud_hits.value, 0.0);
    let canopy_visible = checks
        .iter()
        .find(|check| check.name == "scene_kind_tree_canopy_visible_samples")
        .expect("tree canopy visible check");
    let canopy_hits = checks
        .iter()
        .find(|check| check.name == "scene_kind_tree_canopy_pixel_hits")
        .expect("tree canopy hit check");

    assert!(!canopy_visible.passed);
    assert_eq!(canopy_visible.value, 0.0);
    assert!(!canopy_hits.passed);
    assert_eq!(canopy_hits.value, 0.0);
    assert!(!checks.iter().any(|check| check.name.starts_with("wind_")));
}

#[test]
fn visible_wind_samples_fail_report_and_checkpoint_coverage_without_wind_pixels() {
    let temp_dir = unique_temp_dir("semantic_scene_wind");
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let screenshot_path = temp_dir.join("checkpoint.png");
    let metadata_path = temp_dir.join("checkpoint.markers.json");
    let mut image = RgbImage::from_pixel(100, 80, Rgb([130, 170, 220]));
    image.put_pixel(20, 15, Rgb([54, 128, 70]));
    image.put_pixel(21, 15, Rgb([34, 100, 62]));
    image.put_pixel(22, 15, Rgb([70, 150, 94]));
    image.put_pixel(50, 20, Rgb([44, 126, 32]));
    image.put_pixel(51, 20, Rgb([48, 132, 36]));
    image.put_pixel(52, 20, Rgb([52, 138, 34]));
    image.put_pixel(80, 20, Rgb([158, 166, 174]));
    image.put_pixel(81, 20, Rgb([166, 174, 184]));
    image.put_pixel(82, 20, Rgb([148, 158, 168]));
    image.save(&screenshot_path).expect("screenshot");
    fs::write(
        &metadata_path,
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"viewport\": {{\"width\": 100, \"height\": 80}}, \"scene_samples\": [\
             {{\"kind\": \"terrain_surface\", \"label\": \"terrain\", \"expected_material\": \"terrain\", \"in_viewport\": true, \"screen\": {{\"x\": 20, \"y\": 15}}}},\
             {{\"kind\": \"tree_canopy\", \"label\": \"foliage\", \"expected_material\": \"foliage\", \"in_viewport\": true, \"screen\": {{\"x\": 50, \"y\": 20}}}},\
             {{\"kind\": \"weather_cloud\", \"label\": \"cloud\", \"expected_material\": \"cloud\", \"in_viewport\": true, \"screen\": {{\"x\": 80, \"y\": 20}}}},\
             {{\"kind\": \"updraft_wind_visual\", \"label\": \"updraft wind mote\", \"expected_material\": \"wind\", \"material_variant\": \"wind_updraft\", \"in_viewport\": true, \"screen\": {{\"x\": 10, \"y\": 70}}}}]}}",
            json_string(&screenshot_path.to_string_lossy())
        ),
    )
    .expect("metadata");

    let audit = audit_checkpoint_path(&metadata_path).expect("audit");
    let checks = report_checks(std::slice::from_ref(&audit));
    let wind_hits = checks
        .iter()
        .find(|check| check.name == "wind_scene_sample_pixel_hits")
        .expect("wind hit check");
    let wind_coverage = checks
        .iter()
        .find(|check| check.name == "wind_scene_sample_pixel_coverage")
        .expect("wind coverage check");
    let wind_kind_hits = checks
        .iter()
        .find(|check| check.name == "wind_scene_sample_kind_pixel_hits")
        .expect("wind kind hit check");
    let wind_checkpoint_hits = checks
        .iter()
        .find(|check| check.name == "wind_checkpoint_pixel_hits")
        .expect("wind checkpoint hit check");

    assert!(audit.passed);
    assert_eq!(audit.visible_scene_sample_kind_count, 3);
    assert_eq!(audit.scene_sample_kind_pixel_hit_count, 3);
    assert_eq!(audit.visible_scene_material_count, 3);
    assert_eq!(audit.scene_material_pixel_hit_count, 3);
    assert!(!wind_hits.passed);
    assert_eq!(wind_hits.value, 0.0);
    assert_eq!(wind_hits.threshold, 1.0);
    assert!(!wind_coverage.passed);
    assert_eq!(wind_coverage.value, 0.0);
    assert!(!wind_kind_hits.passed);
    assert_eq!(wind_kind_hits.value, 0.0);
    assert!(!wind_checkpoint_hits.passed);
    assert_eq!(wind_checkpoint_hits.value, 0.0);
    assert_eq!(wind_checkpoint_hits.threshold, 1.0);
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn report_checks_require_wind_pixels_at_each_visible_wind_checkpoint() {
    let missed_wind = SceneSampleAudit {
        kind: "player_wind_shear_visual".to_string(),
        label: "player wind body wrap".to_string(),
        expected_material: "wind".to_string(),
        material_variant: "wind_player_shear".to_string(),
        in_viewport: true,
        visibility: "visible".to_string(),
        screen_x: Some(12.0),
        screen_y: Some(12.0),
        semantic_pixel_hits: 0,
        passed: false,
    };
    let hit_wind = scene_audit_sample(
        "player_wind_shear_visual",
        "player wind body wrap",
        "wind",
        "wind_player_shear",
        MIN_WIND_PIXEL_COVERAGE_PER_VISIBLE_SAMPLE,
    );
    let checks = report_checks(&[
        checkpoint_with_scene_samples("entry", vec![missed_wind]),
        checkpoint_with_scene_samples("glide", vec![hit_wind]),
    ]);
    let aggregate_hits = checks
        .iter()
        .find(|check| check.name == "wind_scene_sample_pixel_hits")
        .expect("aggregate wind hit check");
    let aggregate_coverage = checks
        .iter()
        .find(|check| check.name == "wind_scene_sample_pixel_coverage")
        .expect("aggregate wind coverage check");
    let checkpoint_hits = checks
        .iter()
        .find(|check| check.name == "wind_checkpoint_pixel_hits")
        .expect("checkpoint wind hit check");

    assert!(aggregate_hits.passed);
    assert!(aggregate_coverage.passed);
    assert!(!checkpoint_hits.passed);
    assert_eq!(checkpoint_hits.value, 1.0);
    assert_eq!(checkpoint_hits.threshold, 2.0);
}

#[test]
fn sparse_translucent_wind_sample_hit_satisfies_checkpoint_material_audit() {
    let temp_dir = unique_temp_dir("semantic_scene_sparse_wind");
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let screenshot_path = temp_dir.join("checkpoint.png");
    let metadata_path = temp_dir.join("checkpoint.markers.json");
    let mut image = RgbImage::from_pixel(100, 80, Rgb([130, 170, 220]));
    image.put_pixel(20, 15, Rgb([54, 128, 70]));
    image.put_pixel(21, 15, Rgb([34, 100, 62]));
    image.put_pixel(22, 15, Rgb([70, 150, 94]));
    image.put_pixel(50, 20, Rgb([44, 126, 32]));
    image.put_pixel(51, 20, Rgb([48, 132, 36]));
    image.put_pixel(52, 20, Rgb([52, 138, 34]));
    image.put_pixel(80, 20, Rgb([158, 166, 174]));
    image.put_pixel(81, 20, Rgb([166, 174, 184]));
    image.put_pixel(82, 20, Rgb([148, 158, 168]));
    image.put_pixel(10, 70, Rgb([68, 174, 208]));
    image.put_pixel(11, 70, Rgb([70, 176, 210]));
    image.put_pixel(12, 70, Rgb([72, 178, 212]));
    image.save(&screenshot_path).expect("screenshot");
    fs::write(
        &metadata_path,
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"viewport\": {{\"width\": 100, \"height\": 80}}, \"scene_samples\": [\
             {{\"kind\": \"terrain_surface\", \"label\": \"terrain\", \"expected_material\": \"terrain\", \"in_viewport\": true, \"screen\": {{\"x\": 20, \"y\": 15}}}},\
             {{\"kind\": \"tree_canopy\", \"label\": \"foliage\", \"expected_material\": \"foliage\", \"in_viewport\": true, \"screen\": {{\"x\": 50, \"y\": 20}}}},\
             {{\"kind\": \"weather_cloud\", \"label\": \"cloud\", \"expected_material\": \"cloud\", \"in_viewport\": true, \"screen\": {{\"x\": 80, \"y\": 20}}}},\
             {{\"kind\": \"updraft_wind_visual\", \"label\": \"updraft wind ribbon upper\", \"expected_material\": \"wind\", \"material_variant\": \"wind_updraft\", \"in_viewport\": true, \"screen\": {{\"x\": 10, \"y\": 70}}}},\
             {{\"kind\": \"updraft_wind_visual\", \"label\": \"updraft wind ribbon middle\", \"expected_material\": \"wind\", \"material_variant\": \"wind_updraft\", \"in_viewport\": true, \"screen\": {{\"x\": 50, \"y\": 70}}}},\
             {{\"kind\": \"updraft_wind_visual\", \"label\": \"updraft wind ribbon lower\", \"expected_material\": \"wind\", \"material_variant\": \"wind_updraft\", \"in_viewport\": true, \"screen\": {{\"x\": 70, \"y\": 70}}}},\
             {{\"kind\": \"updraft_wind_visual\", \"label\": \"updraft wind mote\", \"expected_material\": \"wind\", \"material_variant\": \"wind_updraft\", \"in_viewport\": true, \"screen\": {{\"x\": 90, \"y\": 70}}}}]}}",
            json_string(&screenshot_path.to_string_lossy())
        ),
    )
    .expect("metadata");

    let audit = audit_checkpoint_path(&metadata_path).expect("audit");
    let wind_material = audit
        .materials
        .iter()
        .find(|material| material.expected_material == "wind")
        .expect("wind material audit");

    assert!(audit.passed);
    assert_eq!(wind_material.visible_sample_count, 4);
    assert_eq!(wind_material.sample_pixel_hit_count, 1);
    assert_eq!(wind_material.min_sample_pixel_hit_count, 1);
    assert!(wind_material.passed);
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn report_checks_require_scene_pixel_coverage_not_just_hit_counts() {
    let thin_terrain = SceneSampleAudit {
        kind: "terrain_surface".to_string(),
        label: "foreground".to_string(),
        expected_material: "terrain".to_string(),
        material_variant: "terrain_lush_meadow".to_string(),
        in_viewport: true,
        visibility: "visible".to_string(),
        screen_x: Some(12.0),
        screen_y: Some(12.0),
        semantic_pixel_hits: MIN_SAMPLE_PIXEL_HITS,
        passed: true,
    };
    let checkpoint = CheckpointAudit {
        metadata_path: "checkpoint.markers.json".to_string(),
        screenshot_path: "checkpoint.png".to_string(),
        scenario: "default".to_string(),
        checkpoint: "test".to_string(),
        in_viewport_scene_sample_count: 1,
        occluded_scene_sample_count: 0,
        visible_scene_sample_count: 1,
        scene_sample_pixel_hit_count: 1,
        visible_scene_material_count: 1,
        scene_material_pixel_hit_count: 1,
        visible_scene_sample_kind_count: 1,
        scene_sample_kind_pixel_hit_count: 1,
        visible_terrain_material_variant_count: 1,
        terrain_material_variant_pixel_hit_count: 1,
        passed: false,
        samples: vec![thin_terrain],
        materials: Vec::new(),
    };

    let checks = report_checks(&[checkpoint]);
    let terrain_hits = checks
        .iter()
        .find(|check| check.name == "terrain_scene_sample_pixel_hits")
        .expect("terrain hit check");
    let terrain_coverage = checks
        .iter()
        .find(|check| check.name == "terrain_scene_sample_pixel_coverage")
        .expect("terrain coverage check");
    let kind_coverage = checks
        .iter()
        .find(|check| check.name == "scene_kind_terrain_surface_pixel_coverage")
        .expect("terrain kind coverage check");

    assert!(terrain_hits.passed);
    assert!(!terrain_coverage.passed);
    assert_eq!(terrain_coverage.value, MIN_SAMPLE_PIXEL_HITS as f64);
    assert!(!kind_coverage.passed);
    assert_eq!(kind_coverage.value, MIN_SAMPLE_PIXEL_HITS as f64);
}

#[test]
fn report_checks_require_terrain_material_variant_diversity() {
    let checkpoint = CheckpointAudit {
        metadata_path: "checkpoint.markers.json".to_string(),
        screenshot_path: "checkpoint.png".to_string(),
        scenario: "default".to_string(),
        checkpoint: "test".to_string(),
        in_viewport_scene_sample_count: 2,
        occluded_scene_sample_count: 0,
        visible_scene_sample_count: 2,
        scene_sample_pixel_hit_count: 2,
        visible_scene_material_count: 1,
        scene_material_pixel_hit_count: 1,
        visible_scene_sample_kind_count: 1,
        scene_sample_kind_pixel_hit_count: 1,
        visible_terrain_material_variant_count: 2,
        terrain_material_variant_pixel_hit_count: 2,
        passed: false,
        samples: vec![
            terrain_audit_sample("launch mesa", "terrain_lush_meadow"),
            terrain_audit_sample("midpoint shelf", "terrain_gold_meadow"),
        ],
        materials: Vec::new(),
    };

    let checks = report_checks(&[checkpoint]);
    let visible_variants = checks
        .iter()
        .find(|check| check.name == "visible_terrain_material_variant_count")
        .expect("visible terrain variant check");
    let hit_variants = checks
        .iter()
        .find(|check| check.name == "terrain_material_variant_pixel_hit_count")
        .expect("terrain variant hit check");

    assert!(!visible_variants.passed);
    assert_eq!(visible_variants.value, 2.0);
    assert_eq!(
        visible_variants.threshold,
        MIN_VISIBLE_TERRAIN_MATERIAL_VARIANTS as f64
    );
    assert!(!hit_variants.passed);
    assert_eq!(hit_variants.value, 2.0);
    assert_eq!(
        hit_variants.threshold,
        MIN_PASSED_TERRAIN_MATERIAL_VARIANTS as f64
    );
}

#[test]
fn report_checks_require_visible_terrain_material_variants_to_hit() {
    let checkpoint = CheckpointAudit {
        metadata_path: "checkpoint.markers.json".to_string(),
        screenshot_path: "checkpoint.png".to_string(),
        scenario: "default".to_string(),
        checkpoint: "test".to_string(),
        in_viewport_scene_sample_count: 3,
        occluded_scene_sample_count: 0,
        visible_scene_sample_count: 3,
        scene_sample_pixel_hit_count: 1,
        visible_scene_material_count: 1,
        scene_material_pixel_hit_count: 1,
        visible_scene_sample_kind_count: 1,
        scene_sample_kind_pixel_hit_count: 1,
        visible_terrain_material_variant_count: 3,
        terrain_material_variant_pixel_hit_count: 1,
        passed: false,
        samples: vec![
            terrain_audit_sample("launch mesa", "terrain_lush_meadow"),
            visible_failed_terrain_audit_sample("midpoint shelf", "terrain_gold_meadow"),
            visible_failed_terrain_audit_sample("landing garden", "terrain_copper_clay"),
        ],
        materials: Vec::new(),
    };

    let checks = report_checks(&[checkpoint]);
    let checkpoint_variants = checks
        .iter()
        .find(|check| check.name == "checkpoint_terrain_material_variant_hits")
        .expect("checkpoint terrain variant hit check");
    let gold_hits = checks
        .iter()
        .find(|check| check.name == "terrain_gold_meadow_terrain_sample_pixel_hits")
        .expect("gold terrain hit check");

    assert!(!checkpoint_variants.passed);
    assert_eq!(checkpoint_variants.value, 0.0);
    assert!(!gold_hits.passed);
    assert_eq!(gold_hits.value, 0.0);
    assert_eq!(gold_hits.threshold, 1.0);
}

#[test]
fn report_checks_require_per_variant_terrain_pixel_coverage() {
    let checkpoint = CheckpointAudit {
        metadata_path: "checkpoint.markers.json".to_string(),
        screenshot_path: "checkpoint.png".to_string(),
        scenario: "default".to_string(),
        checkpoint: "test".to_string(),
        in_viewport_scene_sample_count: 1,
        occluded_scene_sample_count: 0,
        visible_scene_sample_count: 1,
        scene_sample_pixel_hit_count: 1,
        visible_scene_material_count: 1,
        scene_material_pixel_hit_count: 1,
        visible_scene_sample_kind_count: 1,
        scene_sample_kind_pixel_hit_count: 1,
        visible_terrain_material_variant_count: 1,
        terrain_material_variant_pixel_hit_count: 1,
        passed: false,
        samples: vec![terrain_audit_sample_with_hits(
            "launch mesa",
            "terrain_lush_meadow",
            MIN_SAMPLE_PIXEL_HITS,
        )],
        materials: Vec::new(),
    };

    let checks = report_checks(&[checkpoint]);
    let lush_coverage = checks
        .iter()
        .find(|check| check.name == "terrain_lush_meadow_terrain_sample_pixel_coverage")
        .expect("lush terrain coverage check");

    assert!(!lush_coverage.passed);
    assert_eq!(lush_coverage.value, MIN_SAMPLE_PIXEL_HITS as f64);
    assert_eq!(
        lush_coverage.threshold,
        MIN_TERRAIN_MATERIAL_VARIANT_PIXEL_COVERAGE as f64
    );
}

#[test]
fn world_collision_contact_report_profile_focuses_close_obstruction_scene() {
    let checkpoint = CheckpointAudit {
        metadata_path: "checkpoint.markers.json".to_string(),
        screenshot_path: "checkpoint.png".to_string(),
        scenario: "world_collision_contact".to_string(),
        checkpoint: "blocked_by_tree".to_string(),
        in_viewport_scene_sample_count: 3,
        occluded_scene_sample_count: 0,
        visible_scene_sample_count: 3,
        scene_sample_pixel_hit_count: 3,
        visible_scene_material_count: 3,
        scene_material_pixel_hit_count: 3,
        visible_scene_sample_kind_count: 3,
        scene_sample_kind_pixel_hit_count: 3,
        visible_terrain_material_variant_count: 1,
        terrain_material_variant_pixel_hit_count: 1,
        passed: true,
        samples: vec![
            terrain_audit_sample_with_hits(
                "launch mesa",
                "terrain_lush_meadow",
                MIN_TERRAIN_PIXEL_COVERAGE,
            ),
            scene_audit_sample(
                "tree_canopy",
                "launch mesa",
                "foliage",
                "foliage",
                MIN_FOLIAGE_PIXEL_COVERAGE,
            ),
            scene_audit_sample(
                "distant_island",
                "midpoint shelf",
                "distant_island",
                "distant_island",
                MIN_DISTANT_ISLAND_PIXEL_COVERAGE,
            ),
        ],
        materials: Vec::new(),
    };

    let checkpoints = vec![checkpoint];
    let checks = report_checks(&checkpoints);
    let report =
        serde_json::from_str::<Value>(&report_json(true, &checks, &checkpoints)).expect("report");
    let profile = report.get("profile").expect("profile");

    assert!(checks.iter().all(|check| check.passed));
    assert!(!checks.iter().any(|check| check.name.contains("cloud")));
    assert!(
        !checks
            .iter()
            .any(|check| check.name == "visible_terrain_material_variant_count")
    );
    assert_eq!(
        profile.get("name").and_then(Value::as_str),
        Some("close_obstruction")
    );
    assert_eq!(
        profile
            .get("require_terrain_material_variants")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        profile
            .get("expected_materials")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(3)
    );
}

#[test]
fn plateau_vista_report_profile_focuses_authored_landmarks() {
    let checkpoints = vec![
        checkpoint_with_scene_samples(
            "plateau_arrival_reveal",
            vec![
                scene_audit_sample(
                    "plateau_arrival_ruin",
                    "plateau arrival ruin marker",
                    "stone",
                    "stone_ruin",
                    MIN_SAMPLE_PIXEL_HITS,
                ),
                terrain_audit_sample("great sky plateau", "terrain_alpine_mist"),
            ],
        ),
        checkpoint_with_scene_samples(
            "waterfall_vista",
            vec![
                scene_audit_sample(
                    "waterfall_water",
                    "broken edge waterfall",
                    "water",
                    "water",
                    MIN_SAMPLE_PIXEL_HITS,
                ),
                terrain_audit_sample("great sky plateau", "terrain_alpine_mist"),
            ],
        ),
    ]
    .into_iter()
    .map(|mut checkpoint| {
        checkpoint.scenario = "great_sky_plateau_vistas".to_string();
        checkpoint.visible_scene_material_count = 2;
        checkpoint.scene_material_pixel_hit_count = 2;
        checkpoint.visible_scene_sample_kind_count = 2;
        checkpoint.scene_sample_kind_pixel_hit_count = 2;
        checkpoint
    })
    .collect::<Vec<_>>();

    let checks = report_checks(&checkpoints);
    let report =
        serde_json::from_str::<Value>(&report_json(true, &checks, &checkpoints)).expect("report");
    let profile = report.get("profile").expect("profile");

    assert!(checks.iter().all(|check| check.passed));
    assert!(!checks.iter().any(|check| check.name.contains("cloud")));
    assert_eq!(
        profile.get("name").and_then(Value::as_str),
        Some("plateau_vistas")
    );
    assert_eq!(
        profile
            .get("require_all_visible_families")
            .and_then(Value::as_bool),
        Some(false)
    );
}

#[test]
fn semantic_scene_audit_scales_logical_viewport_to_retina_screenshot() {
    let temp_dir = unique_temp_dir("semantic_scene_retina");
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let screenshot_path = temp_dir.join("checkpoint.png");
    let metadata_path = temp_dir.join("checkpoint.markers.json");
    let mut image = RgbImage::from_pixel(160, 120, Rgb([130, 170, 220]));
    image.put_pixel(40, 30, Rgb([104, 82, 48]));
    image.put_pixel(41, 30, Rgb([92, 74, 46]));
    image.put_pixel(42, 30, Rgb([74, 68, 62]));
    image.put_pixel(140, 110, Rgb([44, 126, 32]));
    image.put_pixel(141, 110, Rgb([48, 132, 36]));
    image.put_pixel(142, 110, Rgb([52, 138, 34]));
    image.put_pixel(120, 86, Rgb([158, 166, 174]));
    image.put_pixel(121, 86, Rgb([166, 174, 184]));
    image.put_pixel(122, 86, Rgb([148, 158, 168]));
    image.save(&screenshot_path).expect("screenshot");
    fs::write(
        &metadata_path,
        semantic_metadata_json(&screenshot_path, 20.0, 15.0, 80.0, 60.0),
    )
    .expect("metadata");

    let audit = audit_checkpoint_path(&metadata_path).expect("audit");

    assert!(audit.passed);
    assert_eq!(audit.visible_scene_sample_count, 3);
    assert_eq!(audit.scene_sample_pixel_hit_count, 3);
    assert_eq!(audit.visible_scene_material_count, 3);
    assert_eq!(audit.scene_material_pixel_hit_count, 3);
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn checkpoint_requires_each_visible_material_family_to_hit() {
    let temp_dir = unique_temp_dir("semantic_scene_materials");
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let screenshot_path = temp_dir.join("checkpoint.png");
    let metadata_path = temp_dir.join("checkpoint.markers.json");
    let mut image = RgbImage::from_pixel(80, 60, Rgb([130, 170, 220]));
    image.put_pixel(20, 15, Rgb([54, 128, 70]));
    image.put_pixel(21, 15, Rgb([34, 100, 62]));
    image.put_pixel(22, 15, Rgb([70, 150, 94]));
    image.save(&screenshot_path).expect("screenshot");
    fs::write(
        &metadata_path,
        semantic_metadata_json(&screenshot_path, 20.0, 15.0, 80.0, 60.0),
    )
    .expect("metadata");

    let audit = audit_checkpoint_path(&metadata_path).expect("audit");

    assert!(!audit.passed);
    assert_eq!(audit.visible_scene_material_count, 3);
    assert_eq!(audit.scene_material_pixel_hit_count, 1);
    assert!(
        audit
            .materials
            .iter()
            .any(|material| material.expected_material == "terrain" && material.passed)
    );
    assert!(
        audit
            .materials
            .iter()
            .any(|material| material.expected_material == "foliage" && !material.passed)
    );
    assert!(
        audit
            .materials
            .iter()
            .any(|material| material.expected_material == "cloud" && !material.passed)
    );
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn checkpoint_requires_enough_visible_terrain_material_variants_to_hit() {
    let temp_dir = unique_temp_dir("semantic_scene_terrain_variants");
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let screenshot_path = temp_dir.join("checkpoint.png");
    let metadata_path = temp_dir.join("checkpoint.markers.json");
    let mut image = RgbImage::from_pixel(80, 60, Rgb([130, 170, 220]));
    image.put_pixel(20, 15, Rgb([54, 128, 70]));
    image.put_pixel(21, 15, Rgb([34, 100, 62]));
    image.put_pixel(22, 15, Rgb([70, 150, 94]));
    image.put_pixel(40, 30, Rgb([44, 126, 32]));
    image.put_pixel(41, 30, Rgb([48, 132, 36]));
    image.put_pixel(42, 30, Rgb([52, 138, 34]));
    image.put_pixel(60, 45, Rgb([158, 166, 174]));
    image.put_pixel(61, 45, Rgb([166, 174, 184]));
    image.put_pixel(62, 45, Rgb([148, 158, 168]));
    image.save(&screenshot_path).expect("screenshot");
    fs::write(
        &metadata_path,
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"viewport\": {{\"width\": 80, \"height\": 60}}, \"scene_samples\": [\
             {{\"kind\": \"terrain_surface\", \"label\": \"launch mesa\", \"expected_material\": \"terrain\", \"material_variant\": \"terrain_lush_meadow\", \"in_viewport\": true, \"screen\": {{\"x\": 20, \"y\": 15}}}},\
             {{\"kind\": \"terrain_surface\", \"label\": \"midpoint shelf\", \"expected_material\": \"terrain\", \"material_variant\": \"terrain_gold_meadow\", \"in_viewport\": true, \"screen\": {{\"x\": 10, \"y\": 50}}}},\
             {{\"kind\": \"terrain_surface\", \"label\": \"landing garden\", \"expected_material\": \"terrain\", \"material_variant\": \"terrain_copper_clay\", \"in_viewport\": true, \"screen\": {{\"x\": 70, \"y\": 50}}}},\
             {{\"kind\": \"tree_canopy\", \"label\": \"foliage\", \"expected_material\": \"foliage\", \"in_viewport\": true, \"screen\": {{\"x\": 40, \"y\": 30}}}},\
             {{\"kind\": \"weather_cloud\", \"label\": \"cloud\", \"expected_material\": \"cloud\", \"in_viewport\": true, \"screen\": {{\"x\": 60, \"y\": 45}}}}]}}",
            json_string(&screenshot_path.to_string_lossy())
        ),
    )
    .expect("metadata");

    let audit = audit_checkpoint_path(&metadata_path).expect("audit");

    assert!(!audit.passed);
    assert_eq!(audit.visible_scene_material_count, 3);
    assert_eq!(audit.scene_material_pixel_hit_count, 3);
    assert_eq!(audit.visible_scene_sample_kind_count, 3);
    assert_eq!(audit.scene_sample_kind_pixel_hit_count, 3);
    assert_eq!(audit.visible_terrain_material_variant_count, 3);
    assert_eq!(audit.terrain_material_variant_pixel_hit_count, 1);
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn checkpoint_requires_visible_scene_sample_kind_diversity() {
    let temp_dir = unique_temp_dir("semantic_scene_kinds");
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let screenshot_path = temp_dir.join("checkpoint.png");
    let metadata_path = temp_dir.join("checkpoint.markers.json");
    let mut image = RgbImage::from_pixel(80, 60, Rgb([130, 170, 220]));
    image.put_pixel(20, 15, Rgb([104, 82, 48]));
    image.put_pixel(21, 15, Rgb([92, 74, 46]));
    image.put_pixel(22, 15, Rgb([74, 68, 62]));
    image.put_pixel(40, 30, Rgb([44, 126, 32]));
    image.put_pixel(41, 30, Rgb([48, 132, 36]));
    image.put_pixel(42, 30, Rgb([52, 138, 34]));
    image.put_pixel(60, 45, Rgb([158, 166, 174]));
    image.put_pixel(61, 45, Rgb([166, 174, 184]));
    image.put_pixel(62, 45, Rgb([148, 158, 168]));
    image.save(&screenshot_path).expect("screenshot");
    fs::write(
        &metadata_path,
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"viewport\": {{\"width\": 80, \"height\": 60}}, \"scene_samples\": [\
             {{\"kind\": \"terrain_surface\", \"label\": \"terrain\", \"expected_material\": \"terrain\", \"in_viewport\": true, \"screen\": {{\"x\": 20, \"y\": 15}}}},\
             {{\"kind\": \"terrain_surface\", \"label\": \"foliage substitute\", \"expected_material\": \"foliage\", \"in_viewport\": true, \"screen\": {{\"x\": 40, \"y\": 30}}}},\
             {{\"kind\": \"terrain_surface\", \"label\": \"cloud substitute\", \"expected_material\": \"cloud\", \"in_viewport\": true, \"screen\": {{\"x\": 60, \"y\": 45}}}}]}}",
            json_string(&screenshot_path.to_string_lossy())
        ),
    )
    .expect("metadata");

    let audit = audit_checkpoint_path(&metadata_path).expect("audit");

    assert!(!audit.passed);
    assert_eq!(audit.visible_scene_material_count, 3);
    assert_eq!(audit.scene_material_pixel_hit_count, 3);
    assert_eq!(audit.visible_scene_sample_kind_count, 1);
    assert_eq!(audit.scene_sample_kind_pixel_hit_count, 1);
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn water_stone_and_plateau_shelf_materials_match_generated_palette_colors() {
    for color in [
        [54.0, 154.0, 210.0],
        [22.0, 92.0, 156.0],
        [160.0, 220.0, 244.0],
    ] {
        assert!(material_matches("water", color[0], color[1], color[2]));
    }
    for color in [[104.0, 82.0, 48.0], [92.0, 74.0, 46.0], [74.0, 68.0, 62.0]] {
        assert!(material_matches("stone", color[0], color[1], color[2]));
    }
    for color in [
        [210.0, 50.0, 96.0],
        [124.0, 28.0, 80.0],
        [255.0, 126.0, 162.0],
    ] {
        assert!(material_matches("flower", color[0], color[1], color[2]));
    }
    assert!(!material_matches("water", 130.0, 170.0, 220.0));
    assert!(!material_matches("water", 97.0, 122.0, 163.0));
}

#[test]
fn checkpoint_honors_false_top_level_sidecar_result() {
    let (temp_dir, metadata_path) =
        checkpoint_fixture("sidecar_false", false, "default", "test", Vec::new(), &[]);

    let audit = audit_checkpoint_path(&metadata_path).expect("audit");

    assert!(!audit.passed);
    assert_eq!(audit.scene_material_pixel_hit_count, 3);
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn plateau_landmark_requirements_do_not_change_generic_scenarios() {
    for checkpoint in ["waterfall_vista", "plateau_arrival_reveal"] {
        let name = format!("generic_{checkpoint}");
        let (temp_dir, metadata_path) =
            checkpoint_fixture(&name, true, "default", checkpoint, Vec::new(), &[]);

        assert!(audit_checkpoint_path(&metadata_path).expect("audit").passed);
        let _ = fs::remove_dir_all(temp_dir);
    }
}

#[test]
fn visible_conditional_materials_contribute_to_checkpoint_counts() {
    let water_sample = projected_sample("water_surface", "low basin lake", "water", 280.0, 150.0);
    let (temp_dir, metadata_path) = checkpoint_fixture(
        "visible_conditional_material",
        true,
        "default",
        "test",
        vec![water_sample],
        &[],
    );

    let audit = audit_checkpoint_path(&metadata_path).expect("audit");
    let water = audit
        .materials
        .iter()
        .find(|material| material.expected_material == "water")
        .expect("visible water material audit");

    assert_eq!(audit.visible_scene_material_count, 4);
    assert_eq!(audit.scene_material_pixel_hit_count, 3);
    assert_eq!(water.visible_sample_count, 1);
    assert!(!water.passed);
    assert!(!audit.passed);
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn waterfall_vista_requires_a_pixel_backed_waterfall_sample() {
    let (missing_dir, missing_path) = checkpoint_fixture(
        "waterfall_missing",
        true,
        "great_sky_plateau_route",
        "waterfall_vista",
        Vec::new(),
        &[],
    );
    assert!(
        !audit_checkpoint_path(&missing_path)
            .expect("missing audit")
            .passed
    );

    let waterfall = projected_sample(
        "waterfall_water",
        "north rim waterfall",
        "water",
        280.0,
        150.0,
    );
    let water_pixels = [
        (279, 150, Rgb([30, 88, 150])),
        (280, 150, Rgb([22, 92, 156])),
        (281, 150, Rgb([40, 94, 160])),
    ];
    let (hit_dir, hit_path) = checkpoint_fixture(
        "waterfall_hit",
        true,
        "great_sky_plateau_route",
        "waterfall_vista",
        vec![waterfall],
        &water_pixels,
    );
    assert!(audit_checkpoint_path(&hit_path).expect("hit audit").passed);

    let _ = fs::remove_dir_all(missing_dir);
    let _ = fs::remove_dir_all(hit_dir);
}

#[test]
fn waterfall_vista_rejects_sky_colored_pixels_at_projected_sample() {
    let waterfall = projected_sample(
        "waterfall_water",
        "north rim waterfall",
        "water",
        280.0,
        150.0,
    );
    let sky_pixels = [
        (279, 150, Rgb([54, 154, 210])),
        (280, 150, Rgb([70, 130, 175])),
        (281, 150, Rgb([160, 220, 244])),
    ];
    let (temp_dir, metadata_path) = checkpoint_fixture(
        "waterfall_sky_false_positive",
        true,
        "great_sky_plateau_route",
        "waterfall_vista",
        vec![waterfall],
        &sky_pixels,
    );

    assert!(
        !audit_checkpoint_path(&metadata_path)
            .expect("sky-colored waterfall audit")
            .passed
    );

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn plateau_arrival_reveal_accepts_pixel_backed_ruin_or_landing_shelf() {
    let (missing_dir, missing_path) = checkpoint_fixture(
        "plateau_arrival_missing",
        true,
        "great_sky_plateau_route",
        "plateau_arrival_reveal",
        Vec::new(),
        &[],
    );
    assert!(
        !audit_checkpoint_path(&missing_path)
            .expect("missing audit")
            .passed
    );

    let ruin = projected_sample(
        "plateau_arrival_ruin",
        "plateau arrival ruin marker",
        "stone",
        280.0,
        150.0,
    );
    let stone_pixels = [
        (279, 150, Rgb([104, 82, 48])),
        (280, 150, Rgb([92, 74, 46])),
        (281, 150, Rgb([74, 68, 62])),
    ];
    let (ruin_dir, ruin_path) = checkpoint_fixture(
        "plateau_arrival_ruin",
        true,
        "great_sky_plateau_route",
        "plateau_arrival_reveal",
        vec![ruin],
        &stone_pixels,
    );
    assert!(
        audit_checkpoint_path(&ruin_path)
            .expect("ruin audit")
            .passed
    );

    let shelf = projected_sample(
        "plateau_arrival_shelf",
        "plateau meadow landing shelf",
        "flower",
        280.0,
        150.0,
    );
    let flower_pixels = [
        (279, 150, Rgb([210, 50, 96])),
        (280, 150, Rgb([124, 28, 80])),
        (281, 150, Rgb([255, 126, 162])),
    ];
    let (shelf_dir, shelf_path) = checkpoint_fixture(
        "plateau_arrival_shelf",
        true,
        "great_sky_plateau_route",
        "plateau_arrival_reveal",
        vec![shelf],
        &flower_pixels,
    );
    assert!(
        audit_checkpoint_path(&shelf_path)
            .expect("shelf audit")
            .passed
    );

    let _ = fs::remove_dir_all(missing_dir);
    let _ = fs::remove_dir_all(ruin_dir);
    let _ = fs::remove_dir_all(shelf_dir);
}

#[test]
fn plateau_vista_checkpoint_allows_unrelated_visible_sample_misses() {
    let ruin = projected_sample(
        "plateau_arrival_ruin",
        "plateau arrival ruin marker",
        "stone",
        280.0,
        150.0,
    );
    let missed_water = projected_sample("water_surface", "off-axis pool", "water", 280.0, 175.0);
    let stone_pixels = [
        (279, 150, Rgb([104, 82, 48])),
        (280, 150, Rgb([92, 74, 46])),
        (281, 150, Rgb([74, 68, 62])),
    ];
    let (temp_dir, metadata_path) = checkpoint_fixture(
        "plateau_vista_partial_family",
        true,
        "great_sky_plateau_vistas",
        "plateau_arrival_reveal",
        vec![ruin, missed_water],
        &stone_pixels,
    );

    assert!(
        audit_checkpoint_path(&metadata_path)
            .expect("plateau vista audit")
            .passed
    );
    let _ = fs::remove_dir_all(temp_dir);
}

fn checkpoint_fixture(
    name: &str,
    sidecar_passed: bool,
    scenario: &str,
    checkpoint: &str,
    extra_samples: Vec<Value>,
    extra_pixels: &[(u32, u32, Rgb<u8>)],
) -> (PathBuf, PathBuf) {
    let temp_dir = unique_temp_dir(name);
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let screenshot_path = temp_dir.join("checkpoint.png");
    let metadata_path = temp_dir.join("checkpoint.markers.json");
    let mut image = RgbImage::from_pixel(320, 200, Rgb([130, 170, 220]));
    for (x, y, color) in [
        (39, 40, Rgb([54, 128, 70])),
        (40, 40, Rgb([34, 100, 62])),
        (41, 40, Rgb([70, 150, 94])),
        (119, 40, Rgb([44, 126, 32])),
        (120, 40, Rgb([48, 132, 36])),
        (121, 40, Rgb([52, 138, 34])),
        (199, 40, Rgb([158, 166, 174])),
        (200, 40, Rgb([166, 174, 184])),
        (201, 40, Rgb([148, 158, 168])),
    ] {
        image.put_pixel(x, y, color);
    }
    for &(x, y, color) in extra_pixels {
        image.put_pixel(x, y, color);
    }
    image.save(&screenshot_path).expect("screenshot");

    let mut scene_samples = vec![
        projected_sample("terrain_surface", "launch mesa", "terrain", 40.0, 40.0),
        projected_sample("tree_canopy", "foliage", "foliage", 120.0, 40.0),
        projected_sample("weather_cloud", "cloud", "cloud", 200.0, 40.0),
    ];
    scene_samples.extend(extra_samples);
    let metadata = serde_json::json!({
        "passed": sidecar_passed,
        "scenario": scenario,
        "checkpoint": checkpoint,
        "screenshot": screenshot_path.to_string_lossy(),
        "viewport": {"width": 320, "height": 200},
        "scene_samples": scene_samples,
    });
    fs::write(
        &metadata_path,
        serde_json::to_string(&metadata).expect("metadata json"),
    )
    .expect("metadata");

    (temp_dir, metadata_path)
}

fn projected_sample(kind: &str, label: &str, expected_material: &str, x: f64, y: f64) -> Value {
    serde_json::json!({
        "kind": kind,
        "label": label,
        "expected_material": expected_material,
        "in_viewport": true,
        "visibility": "visible",
        "screen": {"x": x, "y": y},
    })
}

fn unique_temp_dir(name: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "nau_{name}_{}_{}",
        process::id(),
        std::thread::current().name().unwrap_or("test")
    ))
}

fn terrain_audit_sample(label: &str, material_variant: &str) -> SceneSampleAudit {
    terrain_audit_sample_with_hits(label, material_variant, MIN_SAMPLE_PIXEL_HITS)
}

fn terrain_audit_sample_with_hits(
    label: &str,
    material_variant: &str,
    semantic_pixel_hits: usize,
) -> SceneSampleAudit {
    SceneSampleAudit {
        kind: "terrain_surface".to_string(),
        label: label.to_string(),
        expected_material: "terrain".to_string(),
        material_variant: material_variant.to_string(),
        in_viewport: true,
        visibility: "visible".to_string(),
        screen_x: Some(12.0),
        screen_y: Some(12.0),
        semantic_pixel_hits,
        passed: true,
    }
}

fn scene_audit_sample(
    kind: &str,
    label: &str,
    expected_material: &str,
    material_variant: &str,
    semantic_pixel_hits: usize,
) -> SceneSampleAudit {
    SceneSampleAudit {
        kind: kind.to_string(),
        label: label.to_string(),
        expected_material: expected_material.to_string(),
        material_variant: material_variant.to_string(),
        in_viewport: true,
        visibility: "visible".to_string(),
        screen_x: Some(12.0),
        screen_y: Some(12.0),
        semantic_pixel_hits,
        passed: true,
    }
}

fn checkpoint_with_scene_samples(
    checkpoint: &str,
    samples: Vec<SceneSampleAudit>,
) -> CheckpointAudit {
    let visible_scene_sample_count = samples.iter().filter(|sample| sample.is_visible()).count();
    let scene_sample_pixel_hit_count = samples.iter().filter(|sample| sample.passed).count();
    CheckpointAudit {
        metadata_path: format!("{checkpoint}.markers.json"),
        screenshot_path: format!("{checkpoint}.png"),
        scenario: "default".to_string(),
        checkpoint: checkpoint.to_string(),
        in_viewport_scene_sample_count: visible_scene_sample_count,
        occluded_scene_sample_count: 0,
        visible_scene_sample_count,
        scene_sample_pixel_hit_count,
        visible_scene_material_count: 1,
        scene_material_pixel_hit_count: usize::from(scene_sample_pixel_hit_count > 0),
        visible_scene_sample_kind_count: 1,
        scene_sample_kind_pixel_hit_count: usize::from(scene_sample_pixel_hit_count > 0),
        visible_terrain_material_variant_count: 0,
        terrain_material_variant_pixel_hit_count: 0,
        passed: scene_sample_pixel_hit_count > 0,
        samples,
        materials: Vec::new(),
    }
}

fn visible_failed_terrain_audit_sample(label: &str, material_variant: &str) -> SceneSampleAudit {
    SceneSampleAudit {
        kind: "terrain_surface".to_string(),
        label: label.to_string(),
        expected_material: "terrain".to_string(),
        material_variant: material_variant.to_string(),
        in_viewport: true,
        visibility: "visible".to_string(),
        screen_x: Some(12.0),
        screen_y: Some(12.0),
        semantic_pixel_hits: 0,
        passed: false,
    }
}

fn semantic_metadata_json(
    screenshot_path: &Path,
    x: f64,
    y: f64,
    viewport_width: f64,
    viewport_height: f64,
) -> String {
    format!(
        "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"viewport\": {{\"width\": {}, \"height\": {}}}, \"scene_samples\": [{{\"kind\": \"terrain_surface\", \"label\": \"terrain\", \"expected_material\": \"terrain\", \"in_viewport\": true, \"screen\": {{\"x\": {}, \"y\": {}}}}}, {{\"kind\": \"tree_canopy\", \"label\": \"foliage\", \"expected_material\": \"foliage\", \"in_viewport\": true, \"screen\": {{\"x\": 70.0000, \"y\": 55.0000}}}}, {{\"kind\": \"weather_cloud\", \"label\": \"cloud\", \"expected_material\": \"cloud\", \"in_viewport\": true, \"screen\": {{\"x\": 60.0000, \"y\": 43.0000}}}}]}}",
        json_string(&screenshot_path.to_string_lossy()),
        json_number(viewport_width),
        json_number(viewport_height),
        json_number(x),
        json_number(y)
    )
}
