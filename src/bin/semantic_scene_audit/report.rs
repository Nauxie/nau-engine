use crate::{
    checkpoint::{island_water_story_sample_matches, min_terrain_material_variant_hit_count},
    materials::{aggregate_water_metrics, min_material_sample_pixel_hit_count},
    thresholds::{
        EXPECTED_MATERIALS, EXPECTED_SCENE_SAMPLE_KINDS, EXPECTED_TERRAIN_MATERIAL_VARIANTS,
        MAX_PLAYER_WIND_SHEAR_PIXEL_COVERAGE_PER_CHECKPOINT,
        MAX_WIND_PIXEL_COVERAGE_PER_CHECKPOINT, MIN_PASSED_TERRAIN_MATERIAL_VARIANTS,
        MIN_SAMPLE_PIXEL_HITS, MIN_TERRAIN_MATERIAL_VARIANT_PIXEL_COVERAGE,
        MIN_VISIBLE_MATERIALS_PER_CHECKPOINT, MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT,
        MIN_VISIBLE_TERRAIN_MATERIAL_VARIANTS, MIN_WIND_PIXEL_COVERAGE_PER_VISIBLE_SAMPLE,
        expected_material_pixel_coverage_floor, expected_scene_kind_pixel_coverage_floor,
    },
    types::{
        Check, CheckpointAudit, MaterialAudit, SceneSampleAudit, WaterAggregateMetrics,
        WaterLocalMetrics,
    },
};
use nau_engine::world::{IslandWaterStory, island_art_directions};
use std::collections::{BTreeMap, HashMap, HashSet};

const ISLAND_HERO_GALLERY_TARGET_COUNT: usize = 41;
const ISLAND_HERO_GALLERY_CHECKPOINT_COUNT: usize = 123;
const ISLAND_HERO_GALLERY_VIEWS: [&str; 3] = ["near", "mid", "traversal"];
const CLOSE_OBSTRUCTION_EXPECTED_MATERIALS: &[&str] = &["terrain", "foliage", "distant_island"];
const CLOSE_OBSTRUCTION_EXPECTED_SAMPLE_KINDS: &[&str] =
    &["terrain_surface", "tree_canopy", "distant_island"];
const PLATEAU_VISTA_EXPECTED_MATERIALS: &[&str] = &["stone", "water"];
const PLATEAU_VISTA_EXPECTED_SAMPLE_KINDS: &[&str] = &["plateau_arrival_ruin", "waterfall_water"];
const ISLAND_SURFACE_REVIEW_EXPECTED_MATERIALS: &[&str] = &["stone", "foliage", "water"];
const ISLAND_SURFACE_REVIEW_CONDITIONAL_MATERIALS: &[&str] = &["flower"];
const ISLAND_SURFACE_REVIEW_ISLAND: &str = "great sky plateau";
const ISLAND_SURFACE_REVIEW_EXPECTED_SAMPLE_KINDS: &[&str] = &[
    "ruin_complex",
    "rock_formation",
    "flora_cluster",
    "water_surface",
    "river_channel",
    "waterfall_water",
    "water_detail_waterfall_lip",
    "water_detail_plunge_pool",
];

#[derive(Clone, Copy)]
struct AuditProfile {
    name: &'static str,
    min_visible_materials_per_checkpoint: usize,
    min_visible_sample_kinds_per_checkpoint: usize,
    expected_materials: &'static [&'static str],
    conditional_expected_materials: &'static [&'static str],
    expected_scene_sample_kinds: &'static [&'static str],
    require_terrain_material_variants: bool,
    require_all_visible_families: bool,
    audit_visible_wind_samples: bool,
}

#[derive(Default)]
struct WaterQualityCheckCounts {
    required: usize,
    area_span_passed: usize,
    color_bucket_passed: usize,
    luma_variation_passed: usize,
    internal_edge_density_passed: usize,
    quality_passed: usize,
}

pub(crate) fn report_checks(checkpoints: &[CheckpointAudit]) -> Vec<Check> {
    let profile = audit_profile(checkpoints);
    let passed_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.passed)
        .count();
    let material_family_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| {
            checkpoint.visible_scene_material_count >= profile.min_visible_materials_per_checkpoint
                && checkpoint.scene_material_pixel_hit_count
                    >= if profile.require_all_visible_families {
                        checkpoint.visible_scene_material_count
                    } else {
                        profile.min_visible_materials_per_checkpoint
                    }
        })
        .count();
    let min_visible_material_count = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.visible_scene_material_count)
        .min()
        .unwrap_or(0);
    let material_counts = material_hit_counts(checkpoints);
    let material_pixel_coverage = material_pixel_coverage_counts(checkpoints);
    let visible_material_counts = material_visible_counts(checkpoints);
    let kind_counts = sample_kind_hit_counts(checkpoints);
    let kind_pixel_coverage = sample_kind_pixel_coverage_counts(checkpoints);
    let visible_kind_counts = sample_kind_visible_counts(checkpoints);
    let visible_terrain_variant_counts = terrain_material_variant_visible_counts(checkpoints);
    let terrain_variant_hit_counts = terrain_material_variant_hit_counts(checkpoints);
    let terrain_variant_pixel_coverage =
        terrain_material_variant_pixel_coverage_counts(checkpoints);
    let terrain_variant_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| {
            checkpoint.terrain_material_variant_pixel_hit_count
                >= min_terrain_material_variant_hit_count(
                    checkpoint.visible_terrain_material_variant_count,
                )
        })
        .count();
    let kind_family_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| {
            checkpoint.visible_scene_sample_kind_count
                >= profile.min_visible_sample_kinds_per_checkpoint
                && checkpoint.scene_sample_kind_pixel_hit_count
                    >= if profile.require_all_visible_families {
                        checkpoint.visible_scene_sample_kind_count
                    } else {
                        profile.min_visible_sample_kinds_per_checkpoint
                    }
        })
        .count();
    let min_visible_kind_count = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.visible_scene_sample_kind_count)
        .min()
        .unwrap_or(0);
    let mut checks = vec![Check::at_least(
        "checkpoint_scene_pixel_hits",
        passed_checkpoint_count as f64,
        checkpoints.len() as f64,
        "checkpoints",
    )];

    checks.push(Check::at_least(
        "checkpoint_scene_material_family_hits",
        material_family_checkpoint_count as f64,
        checkpoints.len() as f64,
        "checkpoints",
    ));
    checks.push(Check::at_least(
        "min_visible_scene_material_count",
        min_visible_material_count as f64,
        profile.min_visible_materials_per_checkpoint as f64,
        "materials",
    ));
    checks.push(Check::at_least(
        "checkpoint_scene_sample_kind_hits",
        kind_family_checkpoint_count as f64,
        checkpoints.len() as f64,
        "checkpoints",
    ));
    checks.push(Check::at_least(
        "min_visible_scene_sample_kind_count",
        min_visible_kind_count as f64,
        profile.min_visible_sample_kinds_per_checkpoint as f64,
        "sample_kinds",
    ));
    if profile.require_terrain_material_variants {
        checks.push(Check::at_least(
            "visible_terrain_material_variant_count",
            visible_terrain_variant_counts.len() as f64,
            MIN_VISIBLE_TERRAIN_MATERIAL_VARIANTS as f64,
            "variants",
        ));
        checks.push(Check::at_least(
            "terrain_material_variant_pixel_hit_count",
            terrain_variant_hit_counts.len() as f64,
            MIN_PASSED_TERRAIN_MATERIAL_VARIANTS as f64,
            "variants",
        ));
        checks.push(Check::at_least(
            "checkpoint_terrain_material_variant_hits",
            terrain_variant_checkpoint_count as f64,
            checkpoints.len() as f64,
            "checkpoints",
        ));
    }

    for material in profile.expected_materials {
        checks.push(Check::at_least(
            format!("{material}_visible_scene_samples"),
            *visible_material_counts.get(material).unwrap_or(&0) as f64,
            1.0,
            "samples",
        ));
        checks.push(Check::at_least(
            format!("{material}_scene_sample_pixel_hits"),
            *material_counts.get(material).unwrap_or(&0) as f64,
            1.0,
            "samples",
        ));
        checks.push(Check::at_least(
            format!("{material}_scene_sample_pixel_coverage"),
            *material_pixel_coverage.get(material).unwrap_or(&0) as f64,
            profile_material_pixel_coverage_floor(profile, material) as f64,
            "pixels",
        ));
    }

    for material in profile.conditional_expected_materials {
        let visible_sample_count = *visible_material_counts.get(material).unwrap_or(&0);
        if visible_sample_count == 0 {
            continue;
        }
        let min_pixel_hits = min_material_sample_pixel_hit_count(visible_sample_count);
        checks.push(Check::at_least(
            format!("{material}_visible_scene_samples"),
            visible_sample_count as f64,
            1.0,
            "samples",
        ));
        checks.push(Check::at_least(
            format!("{material}_scene_sample_pixel_hits"),
            *material_counts.get(material).unwrap_or(&0) as f64,
            min_pixel_hits as f64,
            "samples",
        ));
        checks.push(Check::at_least(
            format!("{material}_scene_sample_pixel_coverage"),
            *material_pixel_coverage.get(material).unwrap_or(&0) as f64,
            (min_pixel_hits * MIN_SAMPLE_PIXEL_HITS) as f64,
            "pixels",
        ));
    }

    let visible_wind_samples = *visible_material_counts.get("wind").unwrap_or(&0);
    if profile.audit_visible_wind_samples && visible_wind_samples > 0 {
        let min_wind_pixel_hits = min_material_sample_pixel_hit_count(visible_wind_samples);
        let visible_wind_checkpoints = visible_wind_checkpoint_count(checkpoints);
        checks.push(Check::at_least(
            "wind_visible_scene_samples",
            visible_wind_samples as f64,
            1.0,
            "samples",
        ));
        checks.push(Check::at_least(
            "wind_scene_sample_pixel_hits",
            *material_counts.get("wind").unwrap_or(&0) as f64,
            min_wind_pixel_hits as f64,
            "samples",
        ));
        checks.push(Check::at_least(
            "wind_scene_sample_pixel_coverage",
            *material_pixel_coverage.get("wind").unwrap_or(&0) as f64,
            (min_wind_pixel_hits * MIN_WIND_PIXEL_COVERAGE_PER_VISIBLE_SAMPLE) as f64,
            "pixels",
        ));
        checks.push(Check::at_most(
            "wind_scene_sample_pixel_coverage_ceiling",
            *material_pixel_coverage.get("wind").unwrap_or(&0) as f64,
            (checkpoints.len() * MAX_WIND_PIXEL_COVERAGE_PER_CHECKPOINT) as f64,
            "pixels",
        ));
        let player_wind_shear_pixel_coverage = *kind_pixel_coverage
            .get("player_wind_shear_visual")
            .unwrap_or(&0);
        if player_wind_shear_pixel_coverage > 0 {
            checks.push(Check::at_most(
                "player_wind_shear_scene_pixel_coverage_ceiling",
                player_wind_shear_pixel_coverage as f64,
                (checkpoints.len() * MAX_PLAYER_WIND_SHEAR_PIXEL_COVERAGE_PER_CHECKPOINT) as f64,
                "pixels",
            ));
        }
        checks.push(Check::at_least(
            "wind_scene_sample_kind_pixel_hits",
            wind_sample_kind_hit_count(checkpoints) as f64,
            1.0,
            "sample_kinds",
        ));
        checks.push(Check::at_least(
            "wind_checkpoint_pixel_hits",
            wind_checkpoint_pixel_hit_count(checkpoints) as f64,
            visible_wind_checkpoints as f64,
            "checkpoints",
        ));
    }

    let water_quality = substantial_water_quality_check_counts(checkpoints);
    if water_quality.required > 0 {
        for (name, value) in [
            (
                "water_substantial_local_area_span_passes",
                water_quality.area_span_passed,
            ),
            (
                "water_substantial_quantized_color_bucket_passes",
                water_quality.color_bucket_passed,
            ),
            (
                "water_substantial_luma_p95_p5_passes",
                water_quality.luma_variation_passed,
            ),
            (
                "water_substantial_internal_edge_density_passes",
                water_quality.internal_edge_density_passed,
            ),
            (
                "water_substantial_semantic_quality_passes",
                water_quality.quality_passed,
            ),
        ] {
            checks.push(Check::at_least(
                name,
                value as f64,
                water_quality.required as f64,
                "samples",
            ));
        }
    }

    for kind in profile.expected_scene_sample_kinds {
        checks.push(Check::at_least(
            format!("scene_kind_{kind}_visible_samples"),
            *visible_kind_counts.get(kind).unwrap_or(&0) as f64,
            1.0,
            "samples",
        ));
        checks.push(Check::at_least(
            format!("scene_kind_{kind}_pixel_hits"),
            *kind_counts.get(kind).unwrap_or(&0) as f64,
            1.0,
            "samples",
        ));
        checks.push(Check::at_least(
            format!("scene_kind_{kind}_pixel_coverage"),
            *kind_pixel_coverage.get(kind).unwrap_or(&0) as f64,
            profile_scene_kind_pixel_coverage_floor(profile, kind) as f64,
            "pixels",
        ));
    }

    if profile.name == "island_surface_review" {
        checks.extend(island_surface_review_checkpoint_checks(checkpoints));
    }
    if profile.name == "island_hero_gallery" {
        checks.extend(island_hero_gallery_report_checks(checkpoints));
    }

    if profile.require_terrain_material_variants {
        for variant in EXPECTED_TERRAIN_MATERIAL_VARIANTS {
            let visible_variant_samples =
                *visible_terrain_variant_counts.get(variant).unwrap_or(&0);
            checks.push(Check::at_least(
                format!("{variant}_visible_terrain_samples"),
                visible_variant_samples as f64,
                0.0,
                "samples",
            ));
            checks.push(Check::at_least(
                format!("{variant}_terrain_sample_pixel_hits"),
                *terrain_variant_hit_counts.get(variant).unwrap_or(&0) as f64,
                min_terrain_material_variant_hit_count(visible_variant_samples) as f64,
                "samples",
            ));
            let variant_hit_samples = *terrain_variant_hit_counts.get(variant).unwrap_or(&0);
            checks.push(Check::at_least(
                format!("{variant}_terrain_sample_pixel_coverage"),
                *terrain_variant_pixel_coverage.get(variant).unwrap_or(&0) as f64,
                (variant_hit_samples.max(1) * MIN_TERRAIN_MATERIAL_VARIANT_PIXEL_COVERAGE) as f64,
                "pixels",
            ));
        }
    }

    checks
}

fn substantial_water_quality_check_counts(
    checkpoints: &[CheckpointAudit],
) -> WaterQualityCheckCounts {
    let mut counts = WaterQualityCheckCounts::default();
    for checkpoint in checkpoints {
        let Some(metrics) = aggregate_water_metrics(&checkpoint.samples) else {
            continue;
        };
        counts.required += metrics.quality_required_sample_count;
        counts.area_span_passed += metrics.area_span_passed_sample_count;
        counts.color_bucket_passed += metrics.color_bucket_passed_sample_count;
        counts.luma_variation_passed += metrics.luma_variation_passed_sample_count;
        counts.internal_edge_density_passed += metrics.internal_edge_density_passed_sample_count;
        counts.quality_passed += metrics.quality_passed_sample_count;
    }
    counts
}

fn audit_profile(checkpoints: &[CheckpointAudit]) -> AuditProfile {
    let island_hero_gallery = !checkpoints.is_empty()
        && checkpoints
            .iter()
            .all(|checkpoint| checkpoint.scenario == "island_hero_gallery");
    if island_hero_gallery {
        return AuditProfile {
            name: "island_hero_gallery",
            min_visible_materials_per_checkpoint: 0,
            min_visible_sample_kinds_per_checkpoint: 0,
            expected_materials: &[],
            conditional_expected_materials: &[],
            expected_scene_sample_kinds: &[],
            require_terrain_material_variants: false,
            require_all_visible_families: false,
            audit_visible_wind_samples: false,
        };
    }

    let island_surface_review = !checkpoints.is_empty()
        && checkpoints
            .iter()
            .all(|checkpoint| checkpoint.scenario == "island_surface_review");
    if island_surface_review {
        return AuditProfile {
            name: "island_surface_review",
            min_visible_materials_per_checkpoint: 1,
            min_visible_sample_kinds_per_checkpoint: 1,
            expected_materials: ISLAND_SURFACE_REVIEW_EXPECTED_MATERIALS,
            conditional_expected_materials: ISLAND_SURFACE_REVIEW_CONDITIONAL_MATERIALS,
            expected_scene_sample_kinds: ISLAND_SURFACE_REVIEW_EXPECTED_SAMPLE_KINDS,
            require_terrain_material_variants: false,
            require_all_visible_families: false,
            audit_visible_wind_samples: false,
        };
    }

    let plateau_vistas = !checkpoints.is_empty()
        && checkpoints
            .iter()
            .all(|checkpoint| checkpoint.scenario == "great_sky_plateau_vistas");
    if plateau_vistas {
        return AuditProfile {
            name: "plateau_vistas",
            min_visible_materials_per_checkpoint: 2,
            min_visible_sample_kinds_per_checkpoint: 2,
            expected_materials: PLATEAU_VISTA_EXPECTED_MATERIALS,
            conditional_expected_materials: &[],
            expected_scene_sample_kinds: PLATEAU_VISTA_EXPECTED_SAMPLE_KINDS,
            require_terrain_material_variants: false,
            require_all_visible_families: false,
            audit_visible_wind_samples: true,
        };
    }

    let close_obstruction = !checkpoints.is_empty()
        && checkpoints
            .iter()
            .all(|checkpoint| checkpoint.scenario == "world_collision_contact");

    if close_obstruction {
        return AuditProfile {
            name: "close_obstruction",
            min_visible_materials_per_checkpoint: 3,
            min_visible_sample_kinds_per_checkpoint: 3,
            expected_materials: CLOSE_OBSTRUCTION_EXPECTED_MATERIALS,
            conditional_expected_materials: &[],
            expected_scene_sample_kinds: CLOSE_OBSTRUCTION_EXPECTED_SAMPLE_KINDS,
            require_terrain_material_variants: false,
            require_all_visible_families: true,
            audit_visible_wind_samples: true,
        };
    }

    AuditProfile {
        name: "full_scene",
        min_visible_materials_per_checkpoint: MIN_VISIBLE_MATERIALS_PER_CHECKPOINT,
        min_visible_sample_kinds_per_checkpoint: MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT,
        expected_materials: &EXPECTED_MATERIALS,
        conditional_expected_materials: &[],
        expected_scene_sample_kinds: &EXPECTED_SCENE_SAMPLE_KINDS,
        require_terrain_material_variants: true,
        require_all_visible_families: true,
        audit_visible_wind_samples: true,
    }
}

fn profile_material_pixel_coverage_floor(profile: AuditProfile, material: &str) -> usize {
    if profile.name == "island_surface_review" {
        MIN_SAMPLE_PIXEL_HITS
    } else {
        expected_material_pixel_coverage_floor(material)
    }
}

fn profile_scene_kind_pixel_coverage_floor(profile: AuditProfile, kind: &str) -> usize {
    if profile.name == "island_surface_review" {
        MIN_SAMPLE_PIXEL_HITS
    } else {
        expected_scene_kind_pixel_coverage_floor(kind)
    }
}

fn island_surface_review_checkpoint_checks(checkpoints: &[CheckpointAudit]) -> Vec<Check> {
    let mut checks = Vec::new();
    for (checkpoint, kind, expected_materials, threshold) in [
        ("ruins_and_rock_detail", "ruin_complex", &["stone"][..], 1),
        ("ruins_and_rock_detail", "rock_formation", &["stone"][..], 1),
        (
            "dense_flora_detail",
            "flora_cluster",
            &["foliage", "flower"][..],
            3,
        ),
        (
            "lake_river_waterfall_detail",
            "water_surface",
            &["water"][..],
            1,
        ),
        (
            "lake_river_waterfall_detail",
            "river_channel",
            &["water"][..],
            1,
        ),
        (
            "lake_river_waterfall_detail",
            "waterfall_water",
            &["water"][..],
            1,
        ),
        (
            "lake_river_waterfall_detail",
            "water_detail_waterfall_lip",
            &["stone"][..],
            1,
        ),
        (
            "lake_river_waterfall_detail",
            "water_detail_plunge_pool",
            &["water"][..],
            1,
        ),
    ] {
        checks.push(Check::at_least(
            format!("{checkpoint}_{kind}_pixel_hits"),
            minimum_checkpoint_kind_hit_count(checkpoints, checkpoint, kind, expected_materials)
                as f64,
            threshold as f64,
            "samples",
        ));
    }

    let dense_flora_checkpoints = checkpoints
        .iter()
        .filter(|checkpoint| {
            checkpoint.scenario == "island_surface_review"
                && checkpoint.checkpoint == "dense_flora_detail"
        })
        .collect::<Vec<_>>();
    let distinct_label_checkpoint_count = dense_flora_checkpoints
        .iter()
        .filter(|checkpoint| {
            let passed = distinct_kind_label_count(
                &checkpoint.samples,
                "flora_cluster",
                &["foliage", "flower"],
                true,
            );
            passed >= 2
        })
        .count();
    checks.push(Check::at_least(
        "dense_flora_detail_distinct_flora_labels",
        distinct_label_checkpoint_count as f64,
        dense_flora_checkpoints.len().max(1) as f64,
        "checkpoints",
    ));

    checks
}

fn island_hero_gallery_report_checks(checkpoints: &[CheckpointAudit]) -> Vec<Check> {
    let profiles = island_art_directions();
    let authored_targets = profiles
        .iter()
        .map(|profile| profile.island_name.to_string())
        .collect::<HashSet<_>>();
    let expected_heroes = profiles
        .iter()
        .map(|profile| {
            (
                profile.island_name.to_string(),
                profile.hero_landmark.label().to_string(),
            )
        })
        .collect::<HashSet<_>>();
    let expected_flora = profiles
        .iter()
        .flat_map(|profile| {
            profile
                .flora_kinds
                .iter()
                .take(usize::from(profile.flora_count))
                .map(|kind| (profile.island_name.to_string(), kind.label().to_string()))
        })
        .collect::<HashSet<_>>();
    let expected_formations = profiles
        .iter()
        .flat_map(|profile| {
            profile
                .formation_kinds
                .iter()
                .take(usize::from(profile.formation_count))
                .map(|kind| (profile.island_name.to_string(), kind.label().to_string()))
        })
        .collect::<HashSet<_>>();
    let expected_ruins = profiles
        .iter()
        .flat_map(|profile| {
            profile
                .ruin_kinds
                .iter()
                .take(usize::from(profile.ruin_count))
                .map(|kind| (profile.island_name.to_string(), kind.label().to_string()))
        })
        .collect::<HashSet<_>>();
    let expected_water_islands = profiles
        .iter()
        .filter(|profile| profile.water_story != IslandWaterStory::DryWindCarved)
        .map(|profile| profile.island_name.to_string())
        .collect::<HashSet<_>>();
    let water_story_by_target = profiles
        .iter()
        .map(|profile| (profile.island_name, profile.water_story))
        .collect::<HashMap<_, _>>();

    let mut targets = HashSet::new();
    let mut authored_target_hits = HashSet::new();
    let mut target_view_pairs = HashSet::new();
    let mut views_by_target = HashMap::<String, Vec<String>>::new();
    let mut metadata_checkpoint_count = 0;
    let mut terrain_islands = HashSet::new();
    let mut hero_hits = HashSet::new();
    let mut flora_hits = HashSet::new();
    let mut formation_hits = HashSet::new();
    let mut ruin_hits = HashSet::new();
    let mut water_islands = HashSet::new();

    for checkpoint in checkpoints {
        let (Some(target), Some(view)) = (
            checkpoint.target_island.as_deref(),
            checkpoint.review_view.as_deref(),
        ) else {
            continue;
        };
        targets.insert(target.to_string());
        if authored_targets.contains(target) {
            authored_target_hits.insert(target.to_string());
        }
        if ISLAND_HERO_GALLERY_VIEWS.contains(&view) {
            metadata_checkpoint_count += 1;
            target_view_pairs.insert((target.to_string(), view.to_string()));
            views_by_target
                .entry(target.to_string())
                .or_default()
                .push(view.to_string());
        }
        let near_view = view == "near";

        for sample in checkpoint
            .samples
            .iter()
            .filter(|sample| sample.passed && sample.island_name.as_deref() == Some(target))
        {
            let target_label = (target.to_string(), sample.label.clone());
            match sample.kind.as_str() {
                "terrain_surface" if sample.expected_material == "terrain" => {
                    terrain_islands.insert(target.to_string());
                }
                "hero_landmark" if expected_heroes.contains(&target_label) => {
                    hero_hits.insert(target_label);
                }
                "flora_cluster" if near_view => {
                    if let Some(label) = canonical_flora_label(&sample.label) {
                        let feature = (target.to_string(), label.to_string());
                        if expected_flora.contains(&feature) {
                            flora_hits.insert(feature);
                        }
                    }
                }
                "rock_formation" if near_view => {
                    if let Some(label) = canonical_formation_label(&sample.label) {
                        let feature = (target.to_string(), label.to_string());
                        if expected_formations.contains(&feature) {
                            formation_hits.insert(feature);
                        }
                    }
                }
                "ruin_complex" if near_view => {
                    if let Some(label) = canonical_ruin_label(&sample.label) {
                        let feature = (target.to_string(), label.to_string());
                        if expected_ruins.contains(&feature) {
                            ruin_hits.insert(feature);
                        }
                    }
                }
                _ if near_view
                    && water_story_by_target
                        .get(target)
                        .is_some_and(|story| island_water_story_sample_matches(*story, sample)) =>
                {
                    water_islands.insert(target.to_string());
                }
                _ => {}
            }
        }
    }

    let targets_with_all_views = profiles
        .iter()
        .filter(|profile| {
            let Some(views) = views_by_target.get(profile.island_name) else {
                return false;
            };
            views.len() == ISLAND_HERO_GALLERY_VIEWS.len()
                && ISLAND_HERO_GALLERY_VIEWS
                    .iter()
                    .all(|view| views.iter().any(|candidate| candidate == view))
        })
        .count();
    let passing_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.passed)
        .count();

    vec![
        Check::exactly(
            "island_hero_gallery_checkpoint_count",
            checkpoints.len() as f64,
            ISLAND_HERO_GALLERY_CHECKPOINT_COUNT as f64,
            "checkpoints",
        ),
        Check::exactly(
            "island_hero_gallery_passing_checkpoint_count",
            passing_checkpoint_count as f64,
            ISLAND_HERO_GALLERY_CHECKPOINT_COUNT as f64,
            "checkpoints",
        ),
        Check::exactly(
            "island_hero_gallery_checkpoint_metadata_count",
            metadata_checkpoint_count as f64,
            ISLAND_HERO_GALLERY_CHECKPOINT_COUNT as f64,
            "checkpoints",
        ),
        Check::exactly(
            "island_hero_gallery_unique_target_count",
            targets.len() as f64,
            ISLAND_HERO_GALLERY_TARGET_COUNT as f64,
            "islands",
        ),
        Check::exactly(
            "island_hero_gallery_authored_target_count",
            authored_target_hits.len() as f64,
            ISLAND_HERO_GALLERY_TARGET_COUNT as f64,
            "islands",
        ),
        Check::exactly(
            "island_hero_gallery_unique_target_view_count",
            target_view_pairs.len() as f64,
            ISLAND_HERO_GALLERY_CHECKPOINT_COUNT as f64,
            "views",
        ),
        Check::exactly(
            "island_hero_gallery_targets_with_all_views",
            targets_with_all_views as f64,
            ISLAND_HERO_GALLERY_TARGET_COUNT as f64,
            "islands",
        ),
        Check::exactly(
            "island_hero_gallery_target_terrain_coverage",
            terrain_islands.len() as f64,
            ISLAND_HERO_GALLERY_TARGET_COUNT as f64,
            "islands",
        ),
        Check::exactly(
            "island_hero_gallery_authored_hero_coverage",
            hero_hits.len() as f64,
            expected_heroes.len() as f64,
            "features",
        ),
        Check::exactly(
            "island_hero_gallery_authored_flora_coverage",
            flora_hits.len() as f64,
            expected_flora.len() as f64,
            "features",
        ),
        Check::exactly(
            "island_hero_gallery_authored_formation_coverage",
            formation_hits.len() as f64,
            expected_formations.len() as f64,
            "features",
        ),
        Check::exactly(
            "island_hero_gallery_authored_ruin_coverage",
            ruin_hits.len() as f64,
            expected_ruins.len() as f64,
            "features",
        ),
        Check::exactly(
            "island_hero_gallery_authored_water_coverage",
            water_islands.len() as f64,
            expected_water_islands.len() as f64,
            "islands",
        ),
    ]
}

fn canonical_flora_label(label: &str) -> Option<&'static str> {
    match label {
        "fern_grove" | "fern grove" => Some("fern_grove"),
        "flower_thicket" | "flower thicket" => Some("flower_thicket"),
        "reed_bed" | "reed bed" => Some("reed_bed"),
        "wind_shrub" | "wind-shaped shrub" => Some("wind_shrub"),
        "broadleaf_patch" | "broadleaf patch" => Some("broadleaf_patch"),
        "mushroom_ring" | "mushroom ring" => Some("mushroom_ring"),
        _ => None,
    }
}

fn canonical_formation_label(label: &str) -> Option<&'static str> {
    match label {
        "basalt_crown" | "clustered basalt crown" => Some("basalt_crown"),
        "weathered_arch" | "weathered rock arch" => Some("weathered_arch"),
        "boulder_spine" | "fractured boulder spine" => Some("boulder_spine"),
        "stacked_monoliths" | "stacked leaning monoliths" => Some("stacked_monoliths"),
        "crystal_outcrop" | "faceted crystal outcrop" => Some("crystal_outcrop"),
        _ => None,
    }
}

fn canonical_ruin_label(label: &str) -> Option<&'static str> {
    match label {
        "colonnade" | "ruined perimeter colonnade" => Some("colonnade"),
        "sunken_sanctum" | "sunken open-air sanctum" => Some("sunken_sanctum"),
        "watchtower" | "collapsed watchtower" => Some("watchtower"),
        "broken_aqueduct" | "broken aqueduct arcade" => Some("broken_aqueduct"),
        "processional_stairs" | "processional ruin stairs" => Some("processional_stairs"),
        _ => None,
    }
}

fn minimum_checkpoint_kind_hit_count(
    checkpoints: &[CheckpointAudit],
    checkpoint_name: &str,
    kind: &str,
    expected_materials: &[&str],
) -> usize {
    checkpoints
        .iter()
        .filter(|checkpoint| {
            checkpoint.scenario == "island_surface_review"
                && checkpoint.checkpoint == checkpoint_name
        })
        .map(|checkpoint| {
            checkpoint
                .samples
                .iter()
                .filter(|sample| {
                    sample.passed
                        && sample.island_name.as_deref() == Some(ISLAND_SURFACE_REVIEW_ISLAND)
                        && sample.kind == kind
                        && expected_materials.contains(&sample.expected_material.as_str())
                })
                .count()
        })
        .min()
        .unwrap_or(0)
}

fn distinct_kind_label_count(
    samples: &[SceneSampleAudit],
    kind: &str,
    expected_materials: &[&str],
    passed_only: bool,
) -> usize {
    samples
        .iter()
        .filter(|sample| {
            (!passed_only || sample.passed)
                && sample.island_name.as_deref() == Some(ISLAND_SURFACE_REVIEW_ISLAND)
                && sample.kind == kind
                && expected_materials.contains(&sample.expected_material.as_str())
        })
        .map(|sample| sample.label.as_str())
        .collect::<HashSet<_>>()
        .len()
}

pub(crate) fn material_visible_counts(checkpoints: &[CheckpointAudit]) -> BTreeMap<&str, usize> {
    let mut unique_visible = HashSet::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.is_visible() {
                unique_visible.insert((sample.expected_material.as_str(), sample.label.as_str()));
            }
        }
    }

    let mut counts = BTreeMap::new();
    for (material, _) in unique_visible {
        *counts.entry(material).or_default() += 1;
    }
    counts
}

pub(crate) fn material_hit_counts(checkpoints: &[CheckpointAudit]) -> BTreeMap<&str, usize> {
    let mut unique_hits = HashSet::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.passed {
                unique_hits.insert((sample.expected_material.as_str(), sample.label.as_str()));
            }
        }
    }

    let mut counts = BTreeMap::new();
    for (material, _) in unique_hits {
        *counts.entry(material).or_default() += 1;
    }
    counts
}

pub(crate) fn material_pixel_coverage_counts(
    checkpoints: &[CheckpointAudit],
) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.passed {
                *counts.entry(sample.expected_material.as_str()).or_default() +=
                    sample.semantic_pixel_hits;
            }
        }
    }
    counts
}

pub(crate) fn sample_kind_visible_counts(checkpoints: &[CheckpointAudit]) -> BTreeMap<&str, usize> {
    let mut unique_visible = HashSet::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.is_visible() {
                unique_visible.insert((sample.kind.as_str(), sample.label.as_str()));
            }
        }
    }

    let mut counts = BTreeMap::new();
    for (kind, _) in unique_visible {
        *counts.entry(kind).or_default() += 1;
    }
    counts
}

pub(crate) fn sample_kind_hit_counts(checkpoints: &[CheckpointAudit]) -> BTreeMap<&str, usize> {
    let mut unique_hits = HashSet::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.passed {
                unique_hits.insert((sample.kind.as_str(), sample.label.as_str()));
            }
        }
    }

    let mut counts = BTreeMap::new();
    for (kind, _) in unique_hits {
        *counts.entry(kind).or_default() += 1;
    }
    counts
}

pub(crate) fn sample_kind_pixel_coverage_counts(
    checkpoints: &[CheckpointAudit],
) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.passed {
                *counts.entry(sample.kind.as_str()).or_default() += sample.semantic_pixel_hits;
            }
        }
    }
    counts
}

pub(crate) fn wind_sample_kind_hit_count(checkpoints: &[CheckpointAudit]) -> usize {
    let mut unique_hits = HashSet::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.passed && sample.expected_material == "wind" {
                unique_hits.insert(sample.kind.as_str());
            }
        }
    }

    unique_hits.len()
}

pub(crate) fn visible_wind_checkpoint_count(checkpoints: &[CheckpointAudit]) -> usize {
    checkpoints
        .iter()
        .filter(|checkpoint| {
            checkpoint
                .samples
                .iter()
                .any(|sample| sample.is_visible() && sample.expected_material == "wind")
        })
        .count()
}

pub(crate) fn wind_checkpoint_pixel_hit_count(checkpoints: &[CheckpointAudit]) -> usize {
    checkpoints
        .iter()
        .filter(|checkpoint| {
            let visible_wind_samples = checkpoint
                .samples
                .iter()
                .filter(|sample| sample.is_visible() && sample.expected_material == "wind")
                .count();
            let hit_wind_samples = checkpoint
                .samples
                .iter()
                .filter(|sample| sample.passed && sample.expected_material == "wind")
                .count();

            visible_wind_samples > 0
                && hit_wind_samples >= min_material_sample_pixel_hit_count(visible_wind_samples)
        })
        .count()
}

pub(crate) fn terrain_material_variant_visible_counts(
    checkpoints: &[CheckpointAudit],
) -> BTreeMap<&str, usize> {
    let mut unique_visible = HashSet::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.is_visible() && sample.expected_material == "terrain" {
                unique_visible.insert((sample.material_variant.as_str(), sample.label.as_str()));
            }
        }
    }

    let mut counts = BTreeMap::new();
    for (variant, _) in unique_visible {
        *counts.entry(variant).or_default() += 1;
    }
    counts
}

pub(crate) fn terrain_material_variant_hit_counts(
    checkpoints: &[CheckpointAudit],
) -> BTreeMap<&str, usize> {
    let mut unique_hits = HashSet::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.passed && sample.expected_material == "terrain" {
                unique_hits.insert((sample.material_variant.as_str(), sample.label.as_str()));
            }
        }
    }

    let mut counts = BTreeMap::new();
    for (variant, _) in unique_hits {
        *counts.entry(variant).or_default() += 1;
    }
    counts
}

pub(crate) fn terrain_material_variant_pixel_coverage_counts(
    checkpoints: &[CheckpointAudit],
) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.passed && sample.expected_material == "terrain" {
                *counts.entry(sample.material_variant.as_str()).or_default() +=
                    sample.semantic_pixel_hits;
            }
        }
    }
    counts
}

pub(crate) fn report_json(
    passed: bool,
    checks: &[Check],
    checkpoints: &[CheckpointAudit],
) -> String {
    let checkpoints_json = checkpoints
        .iter()
        .map(checkpoint_json)
        .collect::<Vec<_>>()
        .join(",\n");
    let checks_json = checks
        .iter()
        .map(check_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let profile_json = audit_profile_json(audit_profile(checkpoints));

    format!(
        "{{\n  \"passed\": {},\n  \"checkpoint_count\": {},\n  \"profile\": {},\n  \"checks\": [\n    {}\n  ],\n  \"checkpoints\": [\n{}\n  ]\n}}",
        passed,
        checkpoints.len(),
        profile_json,
        checks_json,
        checkpoints_json
    )
}

fn audit_profile_json(profile: AuditProfile) -> String {
    format!(
        "{{\"name\": {}, \"expected_materials\": {}, \"conditional_expected_materials\": {}, \"expected_scene_sample_kinds\": {}, \"require_terrain_material_variants\": {}, \"require_all_visible_families\": {}, \"audit_visible_wind_samples\": {}}}",
        json_string(profile.name),
        json_string_array(profile.expected_materials),
        json_string_array(profile.conditional_expected_materials),
        json_string_array(profile.expected_scene_sample_kinds),
        profile.require_terrain_material_variants,
        profile.require_all_visible_families,
        profile.audit_visible_wind_samples
    )
}

pub(crate) fn checkpoint_json(checkpoint: &CheckpointAudit) -> String {
    let samples_json = checkpoint
        .samples
        .iter()
        .map(sample_json)
        .collect::<Vec<_>>()
        .join(",\n");
    let materials_json = checkpoint
        .materials
        .iter()
        .map(material_json)
        .collect::<Vec<_>>()
        .join(",\n");
    let target_island = checkpoint
        .target_island
        .as_deref()
        .map(json_string)
        .unwrap_or_else(|| "null".to_string());
    let review_view = checkpoint
        .review_view
        .as_deref()
        .map(json_string)
        .unwrap_or_else(|| "null".to_string());
    let water_metrics = aggregate_water_metrics(&checkpoint.samples)
        .as_ref()
        .map(water_aggregate_metrics_json)
        .unwrap_or_else(|| "null".to_string());
    format!(
        "    {{\n      \"metadata_path\": {},\n      \"screenshot_path\": {},\n      \"scenario\": {},\n      \"checkpoint\": {},\n      \"target_island\": {},\n      \"review_view\": {},\n      \"passed\": {},\n      \"in_viewport_scene_sample_count\": {},\n      \"occluded_scene_sample_count\": {},\n      \"visible_scene_sample_count\": {},\n      \"scene_sample_pixel_hit_count\": {},\n      \"visible_scene_material_count\": {},\n      \"scene_material_pixel_hit_count\": {},\n      \"visible_scene_sample_kind_count\": {},\n      \"scene_sample_kind_pixel_hit_count\": {},\n      \"visible_terrain_material_variant_count\": {},\n      \"terrain_material_variant_pixel_hit_count\": {},\n      \"water_metrics\": {},\n      \"materials\": [\n{}\n      ],\n      \"samples\": [\n{}\n      ]\n    }}",
        json_string(&checkpoint.metadata_path),
        json_string(&checkpoint.screenshot_path),
        json_string(&checkpoint.scenario),
        json_string(&checkpoint.checkpoint),
        target_island,
        review_view,
        checkpoint.passed,
        checkpoint.in_viewport_scene_sample_count,
        checkpoint.occluded_scene_sample_count,
        checkpoint.visible_scene_sample_count,
        checkpoint.scene_sample_pixel_hit_count,
        checkpoint.visible_scene_material_count,
        checkpoint.scene_material_pixel_hit_count,
        checkpoint.visible_scene_sample_kind_count,
        checkpoint.scene_sample_kind_pixel_hit_count,
        checkpoint.visible_terrain_material_variant_count,
        checkpoint.terrain_material_variant_pixel_hit_count,
        water_metrics,
        materials_json,
        samples_json
    )
}

pub(crate) fn material_json(material: &MaterialAudit) -> String {
    let water_metrics = material
        .water_metrics
        .as_ref()
        .map(water_aggregate_metrics_json)
        .unwrap_or_else(|| "null".to_string());
    format!(
        "        {{\"expected_material\": {}, \"visible_sample_count\": {}, \"sample_pixel_hit_count\": {}, \"min_sample_pixel_hit_count\": {}, \"hit_ratio\": {}, \"passed\": {}, \"water_metrics\": {}}}",
        json_string(&material.expected_material),
        material.visible_sample_count,
        material.sample_pixel_hit_count,
        material.min_sample_pixel_hit_count,
        json_number(material.hit_ratio),
        material.passed,
        water_metrics
    )
}

pub(crate) fn sample_json(sample: &SceneSampleAudit) -> String {
    let island = sample
        .island_name
        .as_deref()
        .map(json_string)
        .unwrap_or_else(|| "null".to_string());
    let screen = match (sample.screen_x, sample.screen_y) {
        (Some(x), Some(y)) => format!("{{\"x\": {}, \"y\": {}}}", json_number(x), json_number(y)),
        _ => "null".to_string(),
    };
    let water_local_metrics = sample
        .water_local_metrics
        .as_ref()
        .map(water_local_metrics_json)
        .unwrap_or_else(|| "null".to_string());
    format!(
        "        {{\"island\": {}, \"kind\": {}, \"label\": {}, \"expected_material\": {}, \"material_variant\": {}, \"in_viewport\": {}, \"visibility\": {}, \"screen\": {}, \"semantic_pixel_hits\": {}, \"passed\": {}, \"water_local_metrics\": {}}}",
        island,
        json_string(&sample.kind),
        json_string(&sample.label),
        json_string(&sample.expected_material),
        json_string(&sample.material_variant),
        sample.in_viewport,
        json_string(&sample.visibility),
        screen,
        sample.semantic_pixel_hits,
        sample.passed,
        water_local_metrics
    )
}

fn water_local_metrics_json(metrics: &WaterLocalMetrics) -> String {
    format!(
        "{{\"local_hit_count\": {}, \"x_span_px\": {}, \"y_span_px\": {}, \"quantized_color_bucket_count\": {}, \"luma_p95_p5\": {}, \"internal_edge_density\": {}, \"bounding_box_fill_ratio\": {}, \"quality_required\": {}, \"area_span_passed\": {}, \"color_bucket_passed\": {}, \"luma_variation_passed\": {}, \"internal_edge_density_passed\": {}, \"passed\": {}}}",
        metrics.local_hit_count,
        metrics.x_span,
        metrics.y_span,
        metrics.quantized_color_bucket_count,
        json_number(metrics.luma_p95_p5),
        json_number(metrics.internal_edge_density),
        json_number(metrics.bounding_box_fill_ratio),
        metrics.quality_required,
        metrics.area_span_passed,
        metrics.color_bucket_passed,
        metrics.luma_variation_passed,
        metrics.internal_edge_density_passed,
        metrics.passed
    )
}

fn water_aggregate_metrics_json(metrics: &WaterAggregateMetrics) -> String {
    format!(
        "{{\"visible_sample_count\": {}, \"projected_quality_required_sample_count\": {}, \"quality_required_sample_count\": {}, \"area_span_passed_sample_count\": {}, \"color_bucket_passed_sample_count\": {}, \"luma_variation_passed_sample_count\": {}, \"internal_edge_density_passed_sample_count\": {}, \"quality_passed_sample_count\": {}, \"total_local_hit_count\": {}, \"max_x_span_px\": {}, \"max_y_span_px\": {}, \"max_quantized_color_bucket_count\": {}, \"max_luma_p95_p5\": {}, \"mean_internal_edge_density\": {}, \"passed\": {}}}",
        metrics.visible_sample_count,
        metrics.projected_quality_required_sample_count,
        metrics.quality_required_sample_count,
        metrics.area_span_passed_sample_count,
        metrics.color_bucket_passed_sample_count,
        metrics.luma_variation_passed_sample_count,
        metrics.internal_edge_density_passed_sample_count,
        metrics.quality_passed_sample_count,
        metrics.total_local_hit_count,
        metrics.max_x_span,
        metrics.max_y_span,
        metrics.max_quantized_color_bucket_count,
        json_number(metrics.max_luma_p95_p5),
        json_number(metrics.mean_internal_edge_density),
        metrics.passed
    )
}

pub(crate) fn check_json(check: &Check) -> String {
    format!(
        "{{\"name\": {}, \"passed\": {}, \"value\": {}, \"comparator\": {}, \"threshold\": {}, \"unit\": {}}}",
        json_string(&check.name),
        check.passed,
        json_number(check.value),
        json_string(check.comparator),
        json_number(check.threshold),
        json_string(check.unit)
    )
}

pub(crate) fn json_number(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "0.0000".to_string()
    }
}

pub(crate) fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            character if character <= '\u{1f}' => {
                use std::fmt::Write as _;
                write!(&mut escaped, "\\u{:04x}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            character => escaped.push(character),
        }
    }
    format!("\"{escaped}\"")
}

fn json_string_array(values: &[&str]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
