use crate::{
    materials::{material_audits, sample_pixel_hits},
    thresholds::{
        MIN_PASSED_SAMPLES_PER_CHECKPOINT, MIN_SAMPLE_PIXEL_HITS,
        MIN_VISIBLE_MATERIALS_PER_CHECKPOINT, MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT,
        MIN_VISIBLE_SAMPLES_PER_CHECKPOINT,
    },
    types::{CheckpointAudit, SceneSampleAudit},
};
use image::{ImageReader, RgbImage};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

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
    let passed = visible_scene_sample_count >= MIN_VISIBLE_SAMPLES_PER_CHECKPOINT
        && scene_sample_pixel_hit_count >= MIN_PASSED_SAMPLES_PER_CHECKPOINT
        && visible_scene_material_count >= MIN_VISIBLE_MATERIALS_PER_CHECKPOINT
        && scene_material_pixel_hit_count >= visible_scene_material_count
        && visible_scene_sample_kind_count >= MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT
        && scene_sample_kind_pixel_hit_count >= visible_scene_sample_kind_count
        && terrain_material_variant_pixel_hit_count >= visible_terrain_material_variant_count;
    let checkpoint = parsed
        .get("checkpoint")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    Ok(CheckpointAudit {
        metadata_path: path.to_string_lossy().into_owned(),
        screenshot_path: screenshot_path.to_string_lossy().into_owned(),
        checkpoint,
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

pub(crate) fn visible_scene_sample_kind_count(samples: &[SceneSampleAudit]) -> usize {
    samples
        .iter()
        .filter(|sample| sample.is_visible())
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
    let semantic_pixel_hits = match (visible, screen_x, screen_y) {
        (true, Some(x), Some(y)) => {
            sample_pixel_hits(image, x, y, &expected_material, screenshot_scale)
        }
        _ => 0,
    };

    Ok(SceneSampleAudit {
        kind,
        label,
        expected_material,
        material_variant,
        in_viewport,
        visibility,
        screen_x,
        screen_y,
        semantic_pixel_hits,
        passed: visible && semantic_pixel_hits >= MIN_SAMPLE_PIXEL_HITS,
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
