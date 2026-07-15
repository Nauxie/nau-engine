use image::{ImageReader, RgbImage};
use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const MARKER_SEARCH_RADIUS_PX: i32 = 22;
const MIN_MARKER_PIXEL_HITS: usize = 2;

fn main() {
    let paths = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("Usage: cargo run --bin marker_projection_audit -- <markers.json> [...]");
        process::exit(2);
    }

    let mut checkpoints = Vec::with_capacity(paths.len());
    for path in &paths {
        match audit_checkpoint_path(path) {
            Ok(checkpoint) => checkpoints.push(checkpoint),
            Err(error) => {
                eprintln!("failed to audit {}: {error}", path.display());
                process::exit(2);
            }
        }
    }

    let passed = checkpoints.iter().all(|checkpoint| checkpoint.passed);
    println!("{}", report_json(passed, &checkpoints));
    if !passed {
        process::exit(1);
    }
}

#[derive(Clone, Debug)]
struct MarkerAudit {
    kind: String,
    label: String,
    current_objective: bool,
    in_viewport: bool,
    visibility: String,
    screen_x: Option<f64>,
    screen_y: Option<f64>,
    marker_pixel_hits: usize,
    passed: bool,
}

#[derive(Clone, Debug)]
struct CheckpointAudit {
    metadata_path: String,
    screenshot_path: String,
    checkpoint: String,
    route_marker_projection_required: bool,
    in_viewport_marker_count: usize,
    occluded_marker_count: usize,
    visible_marker_count: usize,
    marker_pixel_hit_count: usize,
    passed: bool,
    markers: Vec<MarkerAudit>,
}

fn audit_checkpoint_path(path: &Path) -> Result<CheckpointAudit, String> {
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

    let markers = parsed
        .get("markers")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing markers array".to_string())?
        .iter()
        .map(|marker| audit_marker(marker, &image, scale))
        .collect::<Result<Vec<_>, _>>()?;
    let in_viewport_marker_count = markers.iter().filter(|marker| marker.in_viewport).count();
    let occluded_marker_count = markers
        .iter()
        .filter(|marker| marker.visibility == "occluded")
        .count();
    let visible_marker_count = markers
        .iter()
        .filter(|marker| marker.visibility == "visible")
        .count();
    let marker_pixel_hit_count = markers.iter().filter(|marker| marker.passed).count();
    let route_marker_projection_required = parsed
        .get("route_marker_projection_required")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let passed = if visible_marker_count > 0 {
        marker_pixel_hit_count > 0
    } else {
        !route_marker_projection_required
    };
    let checkpoint = parsed
        .get("checkpoint")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    Ok(CheckpointAudit {
        metadata_path: path.to_string_lossy().into_owned(),
        screenshot_path: screenshot_path.to_string_lossy().into_owned(),
        checkpoint,
        route_marker_projection_required,
        in_viewport_marker_count,
        occluded_marker_count,
        visible_marker_count,
        marker_pixel_hit_count,
        passed,
        markers,
    })
}

fn resolve_screenshot_path(metadata_path: &Path, screenshot_path: &str) -> PathBuf {
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

fn screenshot_scale(metadata: &Value, image: &RgbImage) -> (f64, f64) {
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

fn audit_marker(
    marker: &Value,
    image: &RgbImage,
    screenshot_scale: (f64, f64),
) -> Result<MarkerAudit, String> {
    let kind = marker
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let label = marker
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let current_objective = marker
        .get("current_objective")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let in_viewport = marker
        .get("in_viewport")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let visibility = marker_visibility(marker, in_viewport);
    let screen = marker.get("screen");
    let screen_x = screen
        .and_then(|screen| screen.get("x"))
        .and_then(Value::as_f64);
    let screen_y = screen
        .and_then(|screen| screen.get("y"))
        .and_then(Value::as_f64);
    let marker_pixel_hits = match (visibility == "visible", screen_x, screen_y) {
        (true, Some(x), Some(y)) => marker_pixel_hits(image, x, y, screenshot_scale),
        _ => 0,
    };

    Ok(MarkerAudit {
        kind,
        label,
        current_objective,
        in_viewport,
        passed: visibility == "visible" && marker_pixel_hits >= MIN_MARKER_PIXEL_HITS,
        visibility,
        screen_x,
        screen_y,
        marker_pixel_hits,
    })
}

fn marker_visibility(marker: &Value, in_viewport: bool) -> String {
    marker
        .get("visibility")
        .and_then(Value::as_str)
        .filter(|visibility| {
            matches!(
                *visibility,
                "visible" | "occluded" | "offscreen" | "behind_camera"
            )
        })
        .unwrap_or(if in_viewport { "visible" } else { "offscreen" })
        .to_string()
}

fn marker_pixel_hits(
    image: &RgbImage,
    screen_x: f64,
    screen_y: f64,
    screenshot_scale: (f64, f64),
) -> usize {
    if !screen_x.is_finite() || !screen_y.is_finite() {
        return 0;
    }

    let width = image.width() as i32;
    let height = image.height() as i32;
    let scale_x = screenshot_scale.0.max(0.1);
    let scale_y = screenshot_scale.1.max(0.1);
    let center_x = (screen_x * scale_x).round() as i32;
    let center_y = (screen_y * scale_y).round() as i32;
    let radius_x = (MARKER_SEARCH_RADIUS_PX as f64 * scale_x).ceil() as i32;
    let radius_y = (MARKER_SEARCH_RADIUS_PX as f64 * scale_y).ceil() as i32;
    let mut hits = 0usize;

    for y in (center_y - radius_y).max(0)..=(center_y + radius_y).min(height.saturating_sub(1)) {
        for x in (center_x - radius_x).max(0)..=(center_x + radius_x).min(width.saturating_sub(1)) {
            let [r, g, b] = image.get_pixel(x as u32, y as u32).0;
            if is_route_marker_like(r as f64, g as f64, b as f64) {
                hits += 1;
            }
        }
    }

    hits
}

fn is_route_marker_like(r: f64, g: f64, b: f64) -> bool {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if luma < 90.0 {
        return false;
    }

    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let saturation = max_channel - min_channel;
    max_channel >= 190.0
        && saturation >= 90.0
        && (r >= g + 60.0 || g >= r + 60.0 || b >= g + 50.0 || b >= r + 50.0)
}

fn report_json(passed: bool, checkpoints: &[CheckpointAudit]) -> String {
    let checkpoint_count = checkpoints.len();
    let passed_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.passed)
        .count();
    let checkpoints_json = checkpoints
        .iter()
        .map(checkpoint_json)
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "{{\n  \"passed\": {},\n  \"checkpoint_count\": {},\n  \"checks\": [\n    {{\"name\": \"checkpoint_marker_pixel_hits\", \"passed\": {}, \"value\": {}, \"comparator\": \">=\", \"threshold\": {}, \"unit\": \"checkpoints\"}}\n  ],\n  \"checkpoints\": [\n{}\n  ]\n}}",
        passed,
        checkpoint_count,
        passed_checkpoint_count >= checkpoint_count,
        passed_checkpoint_count,
        checkpoint_count,
        checkpoints_json
    )
}

fn checkpoint_json(checkpoint: &CheckpointAudit) -> String {
    let markers_json = checkpoint
        .markers
        .iter()
        .map(marker_json)
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "    {{\n      \"metadata_path\": {},\n      \"screenshot_path\": {},\n      \"checkpoint\": {},\n      \"route_marker_projection_required\": {},\n      \"passed\": {},\n      \"in_viewport_marker_count\": {},\n      \"occluded_marker_count\": {},\n      \"visible_marker_count\": {},\n      \"marker_pixel_hit_count\": {},\n      \"markers\": [\n{}\n      ]\n    }}",
        json_string(&checkpoint.metadata_path),
        json_string(&checkpoint.screenshot_path),
        json_string(&checkpoint.checkpoint),
        checkpoint.route_marker_projection_required,
        checkpoint.passed,
        checkpoint.in_viewport_marker_count,
        checkpoint.occluded_marker_count,
        checkpoint.visible_marker_count,
        checkpoint.marker_pixel_hit_count,
        markers_json
    )
}

fn marker_json(marker: &MarkerAudit) -> String {
    let screen = match (marker.screen_x, marker.screen_y) {
        (Some(x), Some(y)) => format!("{{\"x\": {}, \"y\": {}}}", json_number(x), json_number(y)),
        _ => "null".to_string(),
    };
    format!(
        "        {{\"kind\": {}, \"label\": {}, \"current_objective\": {}, \"in_viewport\": {}, \"visibility\": {}, \"screen\": {}, \"marker_pixel_hits\": {}, \"passed\": {}}}",
        json_string(&marker.kind),
        json_string(&marker.label),
        marker.current_objective,
        marker.in_viewport,
        json_string(&marker.visibility),
        screen,
        marker.marker_pixel_hits,
        marker.passed
    )
}

fn json_number(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "0.0000".to_string()
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    #[test]
    fn marker_projection_audit_accepts_pixels_near_visible_marker() {
        let temp_dir = unique_temp_dir("marker_projection_accepts");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        let mut image = RgbImage::from_pixel(80, 60, Rgb([72, 118, 172]));
        image.put_pixel(40, 30, Rgb([246, 58, 142]));
        image.put_pixel(41, 30, Rgb([246, 58, 142]));
        image.save(&screenshot_path).expect("screenshot");
        fs::write(
            &metadata_path,
            marker_metadata_json(&screenshot_path, 40.0, 30.0),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(audit.passed);
        assert_eq!(audit.marker_pixel_hit_count, 1);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn marker_projection_audit_rejects_missing_projected_pixels() {
        let temp_dir = unique_temp_dir("marker_projection_rejects");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        let image = RgbImage::from_pixel(80, 60, Rgb([72, 118, 172]));
        image.save(&screenshot_path).expect("screenshot");
        fs::write(
            &metadata_path,
            marker_metadata_json(&screenshot_path, 40.0, 30.0),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(!audit.passed);
        assert_eq!(audit.marker_pixel_hit_count, 0);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn marker_projection_audit_checks_visible_markers_when_projection_is_optional() {
        let temp_dir = unique_temp_dir("marker_projection_optional_visible");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        RgbImage::from_pixel(80, 60, Rgb([72, 118, 172]))
            .save(&screenshot_path)
            .expect("screenshot");
        fs::write(
            &metadata_path,
            marker_metadata_json_with_viewport(&screenshot_path, 40.0, 30.0, 0.0, 0.0, false),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(!audit.passed);
        assert!(!audit.route_marker_projection_required);
        assert_eq!(audit.visible_marker_count, 1);
        assert_eq!(audit.marker_pixel_hit_count, 0);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn marker_projection_audit_scales_logical_viewport_to_retina_screenshot() {
        let temp_dir = unique_temp_dir("marker_projection_retina");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        let mut image = RgbImage::from_pixel(160, 120, Rgb([72, 118, 172]));
        image.put_pixel(80, 60, Rgb([246, 58, 142]));
        image.put_pixel(81, 60, Rgb([246, 58, 142]));
        image.save(&screenshot_path).expect("screenshot");
        fs::write(
            &metadata_path,
            marker_metadata_json_with_viewport(&screenshot_path, 40.0, 30.0, 80.0, 60.0, true),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(audit.passed);
        assert_eq!(audit.marker_pixel_hit_count, 1);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn marker_projection_audit_classifies_occluded_markers_without_pixel_hits() {
        let temp_dir = unique_temp_dir("marker_projection_occluded");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        let mut image = RgbImage::from_pixel(80, 60, Rgb([72, 118, 172]));
        image.put_pixel(40, 30, Rgb([246, 58, 142]));
        image.put_pixel(41, 30, Rgb([246, 58, 142]));
        image.save(&screenshot_path).expect("screenshot");
        fs::write(
            &metadata_path,
            marker_metadata_json_with_visibility(&screenshot_path),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(audit.passed);
        assert_eq!(audit.in_viewport_marker_count, 2);
        assert_eq!(audit.occluded_marker_count, 1);
        assert_eq!(audit.visible_marker_count, 1);
        assert_eq!(audit.marker_pixel_hit_count, 1);
        assert_eq!(audit.markers[1].marker_pixel_hits, 0);
        assert!(!audit.markers[1].passed);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn marker_projection_audit_accepts_optional_markerless_checkpoint() {
        let temp_dir = unique_temp_dir("marker_projection_optional_markerless");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        RgbImage::from_pixel(80, 60, Rgb([72, 118, 172]))
            .save(&screenshot_path)
            .expect("screenshot");
        fs::write(
            &metadata_path,
            markerless_metadata_json(&screenshot_path, false),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(audit.passed);
        assert!(!audit.route_marker_projection_required);
        assert_eq!(audit.visible_marker_count, 0);
        assert_eq!(audit.marker_pixel_hit_count, 0);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn marker_projection_audit_rejects_required_markerless_checkpoint() {
        let temp_dir = unique_temp_dir("marker_projection_required_markerless");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        RgbImage::from_pixel(80, 60, Rgb([72, 118, 172]))
            .save(&screenshot_path)
            .expect("screenshot");
        fs::write(
            &metadata_path,
            markerless_metadata_json(&screenshot_path, true),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(!audit.passed);
        assert!(audit.route_marker_projection_required);
        assert_eq!(audit.visible_marker_count, 0);
        assert_eq!(audit.marker_pixel_hit_count, 0);
        let _ = fs::remove_dir_all(temp_dir);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "nau_{name}_{}_{}",
            process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
    }

    fn marker_metadata_json(screenshot_path: &Path, x: f64, y: f64) -> String {
        marker_metadata_json_with_viewport(screenshot_path, x, y, 0.0, 0.0, true)
    }

    fn marker_metadata_json_with_viewport(
        screenshot_path: &Path,
        x: f64,
        y: f64,
        viewport_width: f64,
        viewport_height: f64,
        route_marker_projection_required: bool,
    ) -> String {
        let viewport_json = if viewport_width > 0.0 && viewport_height > 0.0 {
            format!(
                ", \"viewport\": {{\"width\": {}, \"height\": {}}}",
                json_number(viewport_width),
                json_number(viewport_height)
            )
        } else {
            String::new()
        };
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}{}, \"route_marker_projection_required\": {}, \"markers\": [{{\"kind\": \"route_cairn\", \"label\": \"test\", \"current_objective\": false, \"in_viewport\": true, \"visibility\": \"visible\", \"screen\": {{\"x\": {}, \"y\": {}}}}}]}}",
            json_string(&screenshot_path.to_string_lossy()),
            viewport_json,
            route_marker_projection_required,
            json_number(x),
            json_number(y)
        )
    }

    fn marker_metadata_json_with_visibility(screenshot_path: &Path) -> String {
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"route_marker_projection_required\": true, \"markers\": [{{\"kind\": \"route_cairn\", \"label\": \"visible\", \"current_objective\": false, \"in_viewport\": true, \"visibility\": \"visible\", \"screen\": {{\"x\": 40.0000, \"y\": 30.0000}}}}, {{\"kind\": \"route_cairn\", \"label\": \"occluded\", \"current_objective\": false, \"in_viewport\": true, \"visibility\": \"occluded\", \"screen\": {{\"x\": 20.0000, \"y\": 15.0000}}, \"occluder\": {{\"kind\": \"sky_island\", \"label\": \"test\", \"distance_m\": 12.0000}}}}]}}",
            json_string(&screenshot_path.to_string_lossy())
        )
    }

    fn markerless_metadata_json(
        screenshot_path: &Path,
        route_marker_projection_required: bool,
    ) -> String {
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"route_marker_projection_required\": {}, \"markers\": []}}",
            json_string(&screenshot_path.to_string_lossy()),
            route_marker_projection_required
        )
    }
}
