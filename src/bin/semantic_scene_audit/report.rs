use crate::{
    checkpoint::min_terrain_material_variant_hit_count,
    materials::min_material_sample_pixel_hit_count,
    thresholds::{
        EXPECTED_MATERIALS, EXPECTED_SCENE_SAMPLE_KINDS, EXPECTED_TERRAIN_MATERIAL_VARIANTS,
        MAX_PLAYER_WIND_SHEAR_PIXEL_COVERAGE_PER_CHECKPOINT,
        MAX_WIND_PIXEL_COVERAGE_PER_CHECKPOINT, MIN_PASSED_TERRAIN_MATERIAL_VARIANTS,
        MIN_TERRAIN_MATERIAL_VARIANT_PIXEL_COVERAGE, MIN_VISIBLE_MATERIALS_PER_CHECKPOINT,
        MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT, MIN_VISIBLE_TERRAIN_MATERIAL_VARIANTS,
        MIN_WIND_PIXEL_COVERAGE_PER_VISIBLE_SAMPLE, expected_material_pixel_coverage_floor,
        expected_scene_kind_pixel_coverage_floor,
    },
    types::{Check, CheckpointAudit, MaterialAudit, SceneSampleAudit},
};
use std::collections::{BTreeMap, HashSet};

const CLOSE_OBSTRUCTION_EXPECTED_MATERIALS: &[&str] = &["terrain", "foliage", "distant_island"];
const CLOSE_OBSTRUCTION_EXPECTED_SAMPLE_KINDS: &[&str] =
    &["terrain_surface", "tree_canopy", "distant_island"];

#[derive(Clone, Copy)]
struct AuditProfile {
    name: &'static str,
    min_visible_materials_per_checkpoint: usize,
    min_visible_sample_kinds_per_checkpoint: usize,
    expected_materials: &'static [&'static str],
    expected_scene_sample_kinds: &'static [&'static str],
    require_terrain_material_variants: bool,
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
                    >= checkpoint.visible_scene_material_count
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
                    >= checkpoint.visible_scene_sample_kind_count
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
            expected_material_pixel_coverage_floor(material) as f64,
            "pixels",
        ));
    }

    let visible_wind_samples = *visible_material_counts.get("wind").unwrap_or(&0);
    if visible_wind_samples > 0 {
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
            expected_scene_kind_pixel_coverage_floor(kind) as f64,
            "pixels",
        ));
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

fn audit_profile(checkpoints: &[CheckpointAudit]) -> AuditProfile {
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
            expected_scene_sample_kinds: CLOSE_OBSTRUCTION_EXPECTED_SAMPLE_KINDS,
            require_terrain_material_variants: false,
        };
    }

    AuditProfile {
        name: "full_scene",
        min_visible_materials_per_checkpoint: MIN_VISIBLE_MATERIALS_PER_CHECKPOINT,
        min_visible_sample_kinds_per_checkpoint: MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT,
        expected_materials: &EXPECTED_MATERIALS,
        expected_scene_sample_kinds: &EXPECTED_SCENE_SAMPLE_KINDS,
        require_terrain_material_variants: true,
    }
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
        "{{\"name\": {}, \"expected_materials\": {}, \"expected_scene_sample_kinds\": {}, \"require_terrain_material_variants\": {}}}",
        json_string(profile.name),
        json_string_array(profile.expected_materials),
        json_string_array(profile.expected_scene_sample_kinds),
        profile.require_terrain_material_variants
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
    format!(
        "    {{\n      \"metadata_path\": {},\n      \"screenshot_path\": {},\n      \"scenario\": {},\n      \"checkpoint\": {},\n      \"passed\": {},\n      \"in_viewport_scene_sample_count\": {},\n      \"occluded_scene_sample_count\": {},\n      \"visible_scene_sample_count\": {},\n      \"scene_sample_pixel_hit_count\": {},\n      \"visible_scene_material_count\": {},\n      \"scene_material_pixel_hit_count\": {},\n      \"visible_scene_sample_kind_count\": {},\n      \"scene_sample_kind_pixel_hit_count\": {},\n      \"visible_terrain_material_variant_count\": {},\n      \"terrain_material_variant_pixel_hit_count\": {},\n      \"materials\": [\n{}\n      ],\n      \"samples\": [\n{}\n      ]\n    }}",
        json_string(&checkpoint.metadata_path),
        json_string(&checkpoint.screenshot_path),
        json_string(&checkpoint.scenario),
        json_string(&checkpoint.checkpoint),
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
        materials_json,
        samples_json
    )
}

pub(crate) fn material_json(material: &MaterialAudit) -> String {
    format!(
        "        {{\"expected_material\": {}, \"visible_sample_count\": {}, \"sample_pixel_hit_count\": {}, \"min_sample_pixel_hit_count\": {}, \"hit_ratio\": {}, \"passed\": {}}}",
        json_string(&material.expected_material),
        material.visible_sample_count,
        material.sample_pixel_hit_count,
        material.min_sample_pixel_hit_count,
        json_number(material.hit_ratio),
        material.passed
    )
}

pub(crate) fn sample_json(sample: &SceneSampleAudit) -> String {
    let screen = match (sample.screen_x, sample.screen_y) {
        (Some(x), Some(y)) => format!("{{\"x\": {}, \"y\": {}}}", json_number(x), json_number(y)),
        _ => "null".to_string(),
    };
    format!(
        "        {{\"kind\": {}, \"label\": {}, \"expected_material\": {}, \"material_variant\": {}, \"in_viewport\": {}, \"visibility\": {}, \"screen\": {}, \"semantic_pixel_hits\": {}, \"passed\": {}}}",
        json_string(&sample.kind),
        json_string(&sample.label),
        json_string(&sample.expected_material),
        json_string(&sample.material_variant),
        sample.in_viewport,
        json_string(&sample.visibility),
        screen,
        sample.semantic_pixel_hits,
        sample.passed
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
