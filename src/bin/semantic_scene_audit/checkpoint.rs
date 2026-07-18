use crate::{
    materials::{material_audits, sample_pixel_hits_with_variant, water_local_metrics},
    thresholds::{
        MIN_PASSED_SAMPLES_PER_CHECKPOINT, MIN_SAMPLE_PIXEL_HITS,
        MIN_VISIBLE_MATERIALS_PER_CHECKPOINT, MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT,
        MIN_VISIBLE_SAMPLES_PER_CHECKPOINT,
    },
    types::{CheckpointAudit, SceneSampleAudit},
};
use image::{ImageReader, RgbImage};
use nau_engine::world::{IslandWaterStory, authored_island_art_direction};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const ISLAND_SURFACE_REVIEW_ISLAND: &str = "great sky plateau";

pub(crate) fn audit_checkpoint_path(path: &Path) -> Result<CheckpointAudit, String> {
    let metadata = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let parsed = serde_json::from_str::<Value>(&metadata).map_err(|error| error.to_string())?;
    let screenshot_path = parsed
        .get("screenshot")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing screenshot path".to_string())?;
    let screenshot_path = resolve_screenshot_path(path, screenshot_path);
    let image = ImageReader::open(&screenshot_path)
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())?
        .to_rgb8();
    let scale = screenshot_scale(&parsed, &image);

    let samples = parsed
        .get("scene_samples")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing scene_samples array".to_string())?
        .iter()
        .map(|sample| audit_scene_sample(sample, &image, scale))
        .collect::<Result<Vec<_>, _>>()?;
    let in_viewport_scene_sample_count = samples.iter().filter(|sample| sample.in_viewport).count();
    let occluded_scene_sample_count = samples
        .iter()
        .filter(|sample| sample.visibility == "occluded")
        .count();
    let visible_scene_sample_count = samples.iter().filter(|sample| sample.is_visible()).count();
    let scene_sample_pixel_hit_count = samples.iter().filter(|sample| sample.passed).count();
    let materials = material_audits(&samples);
    let visible_scene_material_count = materials.len();
    let scene_material_pixel_hit_count =
        materials.iter().filter(|material| material.passed).count();
    let visible_scene_sample_kind_count = visible_scene_sample_kind_count(&samples);
    let scene_sample_kind_pixel_hit_count = scene_sample_kind_pixel_hit_count(&samples);
    let visible_terrain_material_variant_count = visible_terrain_material_variant_count(&samples);
    let terrain_material_variant_pixel_hit_count =
        terrain_material_variant_pixel_hit_count(&samples);
    let checkpoint = parsed
        .get("checkpoint")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let scenario = parsed
        .get("scenario")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let target_island = parsed
        .get("target_island")
        .and_then(Value::as_str)
        .map(str::to_string);
    let review_view = parsed
        .get("review_view")
        .or_else(|| parsed.get("target_view"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let sidecar_passed = parsed
        .get("passed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let passed = if scenario == "island_hero_gallery" {
        sidecar_passed
            && island_hero_gallery_requirement_met(
                target_island.as_deref(),
                review_view.as_deref(),
                &samples,
            )
    } else if scenario == "great_sky_plateau_vistas" {
        sidecar_passed
            && visible_scene_sample_count >= 2
            && scene_sample_pixel_hit_count >= 1
            && visible_scene_material_count >= 2
            && scene_material_pixel_hit_count >= 2
            && visible_scene_sample_kind_count >= 2
            && scene_sample_kind_pixel_hit_count >= 2
            && checkpoint_landmark_requirement_met(&scenario, &checkpoint, &samples)
    } else if scenario == "island_surface_review" {
        sidecar_passed && checkpoint_landmark_requirement_met(&scenario, &checkpoint, &samples)
    } else {
        sidecar_passed
            && visible_scene_sample_count >= MIN_VISIBLE_SAMPLES_PER_CHECKPOINT
            && scene_sample_pixel_hit_count >= MIN_PASSED_SAMPLES_PER_CHECKPOINT
            && visible_scene_material_count >= MIN_VISIBLE_MATERIALS_PER_CHECKPOINT
            && scene_material_pixel_hit_count >= visible_scene_material_count
            && visible_scene_sample_kind_count >= MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT
            && scene_sample_kind_pixel_hit_count >= visible_scene_sample_kind_count
            && terrain_material_variant_pixel_hit_count
                >= min_terrain_material_variant_hit_count(visible_terrain_material_variant_count)
            && checkpoint_landmark_requirement_met(&scenario, &checkpoint, &samples)
    };

    Ok(CheckpointAudit {
        metadata_path: path.to_string_lossy().into_owned(),
        screenshot_path: screenshot_path.to_string_lossy().into_owned(),
        scenario,
        checkpoint,
        target_island,
        review_view,
        in_viewport_scene_sample_count,
        occluded_scene_sample_count,
        visible_scene_sample_count,
        scene_sample_pixel_hit_count,
        visible_scene_material_count,
        scene_material_pixel_hit_count,
        visible_scene_sample_kind_count,
        scene_sample_kind_pixel_hit_count,
        visible_terrain_material_variant_count,
        terrain_material_variant_pixel_hit_count,
        passed,
        samples,
        materials,
    })
}

fn island_hero_gallery_requirement_met(
    target_island: Option<&str>,
    review_view: Option<&str>,
    samples: &[SceneSampleAudit],
) -> bool {
    let (Some(target_island), Some(review_view)) = (target_island, review_view) else {
        return false;
    };
    let Some(profile) = authored_island_art_direction(target_island) else {
        return false;
    };
    let target_sample = |sample: &&SceneSampleAudit| {
        sample.passed && sample.island_name.as_deref() == Some(target_island)
    };
    let target_terrain = samples
        .iter()
        .filter(target_sample)
        .any(|sample| sample.kind == "terrain_surface" && sample.expected_material == "terrain");
    let target_hero = samples.iter().filter(target_sample).any(|sample| {
        sample.kind == "hero_landmark" && sample.label == profile.hero_landmark.label()
    });
    if !target_terrain || !target_hero {
        return false;
    }

    match review_view {
        "mid" | "traversal" => true,
        "near" => {
            let target_flora = samples.iter().filter(target_sample).any(|sample| {
                sample.kind == "flora_cluster"
                    && matches!(sample.expected_material.as_str(), "foliage" | "flower")
            });
            let target_formation = samples.iter().filter(target_sample).any(|sample| {
                sample.kind == "rock_formation" && sample.expected_material == "stone"
            });
            let target_ruin = profile.ruin_count == 0
                || samples.iter().filter(target_sample).any(|sample| {
                    sample.kind == "ruin_complex" && sample.expected_material == "stone"
                });
            let target_water = match profile.water_story {
                IslandWaterStory::DryWindCarved => !samples
                    .iter()
                    .filter(target_sample)
                    .any(is_gallery_water_sample),
                story => samples
                    .iter()
                    .filter(target_sample)
                    .any(|sample| island_water_story_sample_matches(story, sample)),
            };

            target_flora && target_formation && target_ruin && target_water
        }
        _ => false,
    }
}

fn is_gallery_water_sample(sample: &SceneSampleAudit) -> bool {
    sample.expected_material == "water"
        && matches!(
            sample.kind.as_str(),
            "water_surface" | "river_channel" | "waterfall_water" | "waterfall_mist"
        )
}

pub(crate) fn island_water_story_sample_matches(
    story: IslandWaterStory,
    sample: &SceneSampleAudit,
) -> bool {
    if sample.expected_material != "water" {
        return false;
    }

    match story {
        IslandWaterStory::DryWindCarved => false,
        IslandWaterStory::SpringPond => {
            sample.kind == "water_surface"
                && matches!(sample.label.as_str(), "spring pond" | "spring_pond")
        }
        IslandWaterStory::ReflectingBasin => {
            sample.kind == "water_surface"
                && matches!(
                    sample.label.as_str(),
                    "route lake surface" | "reflecting_basin"
                )
        }
        IslandWaterStory::ReedyLake => {
            sample.kind == "water_surface"
                && matches!(sample.label.as_str(), "route lake surface" | "reedy_lake")
        }
        IslandWaterStory::CascadeRun => {
            sample.kind == "river_channel"
                && matches!(sample.label.as_str(), "cascade run" | "cascade_run")
        }
        IslandWaterStory::WaterfallGarden => {
            sample.kind == "waterfall_water"
                && matches!(
                    sample.label.as_str(),
                    "route edge waterfall"
                        | "broken edge waterfall"
                        | "north rim waterfall"
                        | "waterfall_garden"
                )
        }
        IslandWaterStory::MistPool => {
            sample.kind == "water_surface"
                && matches!(sample.label.as_str(), "mist pool" | "mist_pool")
        }
        IslandWaterStory::CaveSeep => {
            sample.kind == "water_surface"
                && matches!(sample.label.as_str(), "cave seep pool" | "cave_seep")
        }
    }
}

pub(crate) fn min_terrain_material_variant_hit_count(visible_variant_count: usize) -> usize {
    crate::materials::min_sample_hit_count_with_ratio(
        visible_variant_count,
        crate::thresholds::MIN_TERRAIN_MATERIAL_VARIANT_HIT_RATIO,
    )
}

pub(crate) fn visible_scene_sample_kind_count(samples: &[SceneSampleAudit]) -> usize {
    samples
        .iter()
        .filter(|sample| sample_counts_toward_visible_kind(sample))
        .map(|sample| sample.kind.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

pub(crate) fn scene_sample_kind_pixel_hit_count(samples: &[SceneSampleAudit]) -> usize {
    samples
        .iter()
        .filter(|sample| sample.passed)
        .map(|sample| sample.kind.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

fn sample_counts_toward_visible_kind(sample: &SceneSampleAudit) -> bool {
    sample.is_visible() && (sample.expected_material != "wind" || sample.passed)
}

fn checkpoint_landmark_requirement_met(
    scenario: &str,
    checkpoint: &str,
    samples: &[SceneSampleAudit],
) -> bool {
    match (scenario, checkpoint) {
        ("great_sky_plateau_route" | "great_sky_plateau_vistas", "waterfall_vista") => {
            samples.iter().any(|sample| {
                sample.passed
                    && sample.kind == "waterfall_water"
                    && sample.expected_material == "water"
            })
        }
        ("great_sky_plateau_route" | "great_sky_plateau_vistas", "plateau_arrival_reveal") => {
            samples.iter().any(|sample| {
                sample.passed
                    && matches!(
                        sample.kind.as_str(),
                        "plateau_arrival_ruin" | "plateau_arrival_shelf"
                    )
            })
        }
        ("island_surface_review", "ruins_and_rock_detail") => {
            passed_kind_material_count(samples, "ruin_complex", &["stone"]) >= 1
                && passed_kind_material_count(samples, "rock_formation", &["stone"]) >= 1
        }
        ("island_surface_review", "dense_flora_detail") => {
            let passed_labels =
                distinct_kind_labels(samples, "flora_cluster", &["foliage", "flower"], true);
            passed_kind_material_count(samples, "flora_cluster", &["foliage", "flower"]) >= 3
                && passed_labels >= 2
        }
        ("island_surface_review", "lake_river_waterfall_detail") => {
            passed_kind_material_count(samples, "water_surface", &["water"]) >= 1
                && passed_kind_material_count(samples, "river_channel", &["water"]) >= 1
                && passed_kind_material_count(samples, "waterfall_water", &["water"]) >= 1
                && passed_kind_material_count(samples, "water_detail_waterfall_lip", &["stone"])
                    >= 1
                && passed_kind_material_count(samples, "water_detail_plunge_pool", &["water"]) >= 1
        }
        ("island_surface_review", _) => false,
        _ => true,
    }
}

fn passed_kind_material_count(
    samples: &[SceneSampleAudit],
    kind: &str,
    expected_materials: &[&str],
) -> usize {
    samples
        .iter()
        .filter(|sample| {
            sample.passed
                && sample.island_name.as_deref() == Some(ISLAND_SURFACE_REVIEW_ISLAND)
                && sample.kind == kind
                && expected_materials.contains(&sample.expected_material.as_str())
        })
        .count()
}

fn distinct_kind_labels(
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
        .collect::<BTreeSet<_>>()
        .len()
}

pub(crate) fn visible_terrain_material_variant_count(samples: &[SceneSampleAudit]) -> usize {
    samples
        .iter()
        .filter(|sample| sample.is_visible() && sample.expected_material == "terrain")
        .map(|sample| sample.material_variant.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

pub(crate) fn terrain_material_variant_pixel_hit_count(samples: &[SceneSampleAudit]) -> usize {
    samples
        .iter()
        .filter(|sample| sample.passed && sample.expected_material == "terrain")
        .map(|sample| sample.material_variant.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

pub(crate) fn resolve_screenshot_path(metadata_path: &Path, screenshot_path: &str) -> PathBuf {
    let direct_path = PathBuf::from(screenshot_path);
    if direct_path.exists() || direct_path.is_absolute() {
        return direct_path;
    }

    if let Some(sibling_path) = metadata_path
        .parent()
        .map(|parent| parent.join(&direct_path))
        .filter(|path| path.exists())
    {
        return sibling_path;
    }

    if let Some(eval_relative_path) = metadata_path
        .parent()
        .and_then(Path::parent)
        .map(|eval_dir| eval_dir.join(&direct_path))
        .filter(|path| path.exists())
    {
        return eval_relative_path;
    }

    direct_path
}

pub(crate) fn screenshot_scale(metadata: &Value, image: &RgbImage) -> (f64, f64) {
    let viewport = metadata.get("viewport");
    let Some(viewport_width) = viewport
        .and_then(|viewport| viewport.get("width"))
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
    else {
        return (1.0, 1.0);
    };
    let Some(viewport_height) = viewport
        .and_then(|viewport| viewport.get("height"))
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
    else {
        return (1.0, 1.0);
    };

    (
        image.width() as f64 / viewport_width,
        image.height() as f64 / viewport_height,
    )
}

pub(crate) fn audit_scene_sample(
    sample: &Value,
    image: &RgbImage,
    screenshot_scale: (f64, f64),
) -> Result<SceneSampleAudit, String> {
    let island_name = sample
        .get("island")
        .and_then(Value::as_str)
        .map(str::to_string);
    let kind = sample
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let label = sample
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let expected_material = sample
        .get("expected_material")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let material_variant = sample
        .get("material_variant")
        .and_then(Value::as_str)
        .filter(|variant| !variant.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            default_scene_sample_material_variant(&expected_material, &label).to_string()
        });
    let in_viewport = sample
        .get("in_viewport")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let visibility = sample
        .get("visibility")
        .and_then(Value::as_str)
        .unwrap_or(if in_viewport { "visible" } else { "offscreen" })
        .to_string();
    let screen = sample.get("screen");
    let screen_x = screen
        .and_then(|screen| screen.get("x"))
        .and_then(Value::as_f64);
    let screen_y = screen
        .and_then(|screen| screen.get("y"))
        .and_then(Value::as_f64);
    let visible = in_viewport && visibility == "visible";
    let water_local_metrics = if visible && expected_material == "water" {
        Some(water_local_metrics(
            image,
            screen_x.unwrap_or(f64::NAN),
            screen_y.unwrap_or(f64::NAN),
            &kind,
            &material_variant,
            screenshot_scale,
        ))
    } else {
        None
    };
    let semantic_pixel_hits = match (water_local_metrics.as_ref(), visible, screen_x, screen_y) {
        (Some(metrics), _, _, _) => metrics.local_hit_count,
        (None, true, Some(x), Some(y)) => sample_pixel_hits_with_variant(
            image,
            x,
            y,
            &expected_material,
            &material_variant,
            screenshot_scale,
        ),
        _ => 0,
    };
    let water_quality_passed = water_local_metrics
        .as_ref()
        .is_none_or(|metrics| metrics.passed);

    Ok(SceneSampleAudit {
        island_name,
        kind,
        label,
        expected_material,
        material_variant,
        in_viewport,
        visibility,
        screen_x,
        screen_y,
        semantic_pixel_hits,
        water_local_metrics,
        passed: visible && semantic_pixel_hits >= MIN_SAMPLE_PIXEL_HITS && water_quality_passed,
    })
}

pub(crate) fn default_scene_sample_material_variant(
    expected_material: &str,
    label: &str,
) -> &'static str {
    if expected_material != "terrain" {
        return match expected_material {
            "foliage" => "foliage",
            "cloud" => "cloud",
            "distant_island" => "distant_island",
            "wind" => "wind",
            "water" => "water",
            "stone" => "stone_ruin",
            "wood" => "wood",
            "flower" => "flower",
            _ => "unknown",
        };
    }

    terrain_material_variant_for_label(label).unwrap_or("terrain_unknown")
}

pub(crate) fn terrain_material_variant_for_label(label: &str) -> Option<&'static str> {
    match label {
        "launch mesa" | "copper stair" | "far needle" => Some("terrain_lush_meadow"),
        "midpoint shelf" | "sunlit terrace" | "sapphire basin" => Some("terrain_gold_meadow"),
        "landing garden" | "western refuge" => Some("terrain_copper_clay"),
        "distant crown" | "storm porch" => Some("terrain_alpine_mist"),
        "wind overlook" | "high orchard" => Some("terrain_highland_grass"),
        _ => None,
    }
}
