use crate::{
    checkpoint::{audit_checkpoint_path, audit_scene_sample, terrain_material_variant_for_label},
    materials::sample_pixel_hits,
    report::{json_number, json_string, report_checks},
    thresholds::{
        MIN_PASSED_TERRAIN_MATERIAL_VARIANTS, MIN_SAMPLE_PIXEL_HITS,
        MIN_TERRAIN_MATERIAL_VARIANT_PIXEL_COVERAGE, MIN_VISIBLE_TERRAIN_MATERIAL_VARIANTS,
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
    image.put_pixel(32, 32, Rgb([104, 82, 48]));
    image.put_pixel(33, 32, Rgb([92, 74, 46]));
    image.put_pixel(34, 32, Rgb([74, 68, 62]));
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
fn audit_scene_sample_derives_legacy_terrain_variant_from_label() {
    let mut image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
    image.put_pixel(32, 32, Rgb([104, 82, 48]));
    image.put_pixel(33, 32, Rgb([92, 74, 46]));
    image.put_pixel(34, 32, Rgb([74, 68, 62]));
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
        checkpoint: "test".to_string(),
        in_viewport_scene_sample_count: 2,
        occluded_scene_sample_count: 0,
        visible_scene_sample_count: 2,
        scene_sample_pixel_hit_count: 1,
        visible_scene_material_count: 1,
        scene_material_pixel_hit_count: 1,
        visible_scene_sample_kind_count: 1,
        scene_sample_kind_pixel_hit_count: 1,
        visible_terrain_material_variant_count: 2,
        terrain_material_variant_pixel_hit_count: 1,
        passed: false,
        samples: vec![
            terrain_audit_sample("launch mesa", "terrain_lush_meadow"),
            visible_failed_terrain_audit_sample("midpoint shelf", "terrain_gold_meadow"),
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
    image.put_pixel(20, 15, Rgb([104, 82, 48]));
    image.put_pixel(21, 15, Rgb([92, 74, 46]));
    image.put_pixel(22, 15, Rgb([74, 68, 62]));
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
fn checkpoint_requires_each_visible_terrain_material_variant_to_hit() {
    let temp_dir = unique_temp_dir("semantic_scene_terrain_variants");
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
             {{\"kind\": \"terrain_surface\", \"label\": \"launch mesa\", \"expected_material\": \"terrain\", \"material_variant\": \"terrain_lush_meadow\", \"in_viewport\": true, \"screen\": {{\"x\": 20, \"y\": 15}}}},\
             {{\"kind\": \"terrain_surface\", \"label\": \"midpoint shelf\", \"expected_material\": \"terrain\", \"material_variant\": \"terrain_gold_meadow\", \"in_viewport\": true, \"screen\": {{\"x\": 10, \"y\": 50}}}},\
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
    assert_eq!(audit.visible_terrain_material_variant_count, 2);
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
