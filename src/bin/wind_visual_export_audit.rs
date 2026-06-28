use bevy::prelude::*;
use nau_engine::environment::{GAMEPLAY_LIFT_ROUTE, WindField, visual_crosswind_fields};
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const EXPECTED_SCHEMA: &str = "nau_wind_visual_export.v1";
const MIN_UPDRAFT_FIELDS: u64 = 2;
const MIN_CROSSWIND_FIELDS: u64 = 4;
const MIN_UPDRAFT_GUIDES: u64 = 210;
const MIN_UPDRAFT_RIBBONS: u64 = 12;
const MIN_CROSSWIND_GUIDES: u64 = 240;
const MIN_CROSSWIND_RIBBONS: u64 = 28;
const MIN_TOTAL_TRACKS: u64 = 1100;
const MIN_TOTAL_COHERENT_TRACKS: u64 = 950;
const MIN_UPDRAFT_GUIDE_COHERENT_TRACKS: u64 = 400;
const MIN_UPDRAFT_RIBBON_COHERENT_TRACKS: u64 = 40;
const MIN_CROSSWIND_GUIDE_COHERENT_TRACKS: u64 = 450;
const MIN_CROSSWIND_RIBBON_COHERENT_TRACKS: u64 = 75;
const MAX_STATIC_TRACKS: u64 = 0;
const MAX_OFF_FIELD_TRACKS: u64 = 55;
const MAX_LOW_ALIGNMENT_TRACKS: u64 = 180;
const MIN_FAMILY_MAX_DISPLACEMENT_M: f64 = 0.25;
const MIN_TRACK_DISPLACEMENT_M: f32 = 0.01;
const WIND_VISUAL_ALIGNMENT_MIN_DOT: f32 = 0.55;

#[derive(Clone, Copy, Debug, Default)]
struct FamilyMetrics {
    track_count: u64,
    coherent_track_count: u64,
    static_track_count: u64,
    off_field_track_count: u64,
    low_alignment_track_count: u64,
    max_displacement_m: f64,
}

#[derive(Clone, Copy, Debug, Default)]
struct TrackMetrics {
    total: FamilyMetrics,
    updraft_guide: FamilyMetrics,
    updraft_ribbon: FamilyMetrics,
    crosswind_guide: FamilyMetrics,
    crosswind_ribbon: FamilyMetrics,
    displacement_mismatches: u64,
    field_containment_mismatches: u64,
    coherence_mismatches: u64,
    unknown_family_count: u64,
    missing_field_count: u64,
    malformed_track_count: u64,
}

#[derive(Clone, Copy, Debug)]
struct ParsedTrack {
    family: &'static str,
    sample_kind: &'static str,
    field_index: usize,
    elapsed_secs: f32,
    current: Vec3,
    next: Vec3,
    manifest_displacement_m: f64,
    manifest_current_inside_field: bool,
    manifest_next_inside_field: bool,
    manifest_coherent: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct ObjAudit {
    vertex_count: u64,
    segment_count: u64,
}

fn main() {
    let args = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if args.len() != 1 {
        eprintln!("Usage: cargo run --bin wind_visual_export_audit -- <manifest.json>");
        process::exit(2);
    }

    match audit_manifest_path(&args[0]) {
        Ok(report) => {
            let passed = report
                .get("passed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("audit report should serialize")
            );
            if !passed {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("wind visual export audit failed: {error}");
            process::exit(2);
        }
    }
}

fn audit_manifest_path(path: &Path) -> Result<Value, String> {
    let manifest_text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let manifest = serde_json::from_str::<Value>(&manifest_text).map_err(|error| {
        format!(
            "could not parse wind visual manifest {}: {error}",
            path.display()
        )
    })?;
    let root_dir = path.parent().unwrap_or_else(|| Path::new("."));

    Ok(audit_manifest(&manifest, root_dir, &path.to_string_lossy()))
}

fn audit_manifest(manifest: &Value, root_dir: &Path, manifest_path: &str) -> Value {
    let mut checks = Vec::new();
    let schema = manifest.get("schema").and_then(Value::as_str).unwrap_or("");
    checks.push(check_eq_str("schema", schema, EXPECTED_SCHEMA, "schema"));

    let artifacts = manifest.get("artifacts").unwrap_or(&Value::Null);
    let counts = manifest.get("counts").unwrap_or(&Value::Null);
    let coverage = manifest.get("coverage").unwrap_or(&Value::Null);
    let motion = manifest.get("motion").unwrap_or(&Value::Null);
    let track_count = value_u64(counts, "track_count");

    checks.push(check_at_least_u64(
        "updraft_field_count",
        value_u64(counts, "updraft_field_count"),
        MIN_UPDRAFT_FIELDS,
        "fields",
    ));
    checks.push(check_at_least_u64(
        "crosswind_field_count",
        value_u64(counts, "crosswind_field_count"),
        MIN_CROSSWIND_FIELDS,
        "fields",
    ));
    checks.push(check_at_least_u64(
        "updraft_guide_count",
        value_u64(counts, "updraft_guide_count"),
        MIN_UPDRAFT_GUIDES,
        "visuals",
    ));
    checks.push(check_at_least_u64(
        "updraft_ribbon_count",
        value_u64(counts, "updraft_ribbon_count"),
        MIN_UPDRAFT_RIBBONS,
        "visuals",
    ));
    checks.push(check_at_least_u64(
        "crosswind_guide_count",
        value_u64(counts, "crosswind_guide_count"),
        MIN_CROSSWIND_GUIDES,
        "visuals",
    ));
    checks.push(check_at_least_u64(
        "crosswind_ribbon_count",
        value_u64(counts, "crosswind_ribbon_count"),
        MIN_CROSSWIND_RIBBONS,
        "visuals",
    ));
    checks.push(check_at_least_u64(
        "track_count",
        track_count,
        MIN_TOTAL_TRACKS,
        "tracks",
    ));
    checks.push(check_eq_u64(
        "track_vertex_count",
        value_u64(counts, "track_vertex_count"),
        track_count.saturating_mul(2),
        "vertices",
    ));
    checks.push(check_eq_u64(
        "track_segment_count",
        value_u64(counts, "track_segment_count"),
        track_count,
        "segments",
    ));
    checks.push(check_at_least_u64(
        "updraft_fields_with_guides_and_ribbons",
        value_u64(coverage, "updraft_fields_with_guides_and_ribbons_count"),
        MIN_UPDRAFT_FIELDS,
        "fields",
    ));
    checks.push(check_at_least_u64(
        "crosswind_fields_with_guides_and_ribbons",
        value_u64(coverage, "crosswind_fields_with_guides_and_ribbons_count"),
        MIN_CROSSWIND_FIELDS,
        "fields",
    ));

    let (obj_audit, obj_error) = match relative_path(artifacts, "track_obj") {
        Some(path) => match audit_obj_path(&root_dir.join(&path)) {
            Ok(audit) => (audit, None),
            Err(error) => (ObjAudit::default(), Some(error)),
        },
        None => (
            ObjAudit::default(),
            Some("missing track_obj artifact".to_string()),
        ),
    };
    let (track_metrics, ndjson_line_count, ndjson_error) =
        match relative_path(artifacts, "track_ndjson") {
            Some(path) => match audit_track_ndjson(&root_dir.join(&path)) {
                Ok((metrics, line_count)) => (metrics, line_count, None),
                Err(error) => (TrackMetrics::default(), 0, Some(error)),
            },
            None => (
                TrackMetrics::default(),
                0,
                Some("missing track_ndjson artifact".to_string()),
            ),
        };

    checks.push(check_bool(
        "track_obj_present",
        obj_error.is_none(),
        "artifact",
    ));
    checks.push(check_bool(
        "track_ndjson_present",
        ndjson_error.is_none(),
        "artifact",
    ));
    checks.push(check_eq_u64(
        "obj_vertex_count",
        obj_audit.vertex_count,
        track_count.saturating_mul(2),
        "vertices",
    ));
    checks.push(check_eq_u64(
        "obj_segment_count",
        obj_audit.segment_count,
        track_count,
        "segments",
    ));
    checks.push(check_eq_u64(
        "ndjson_line_count",
        ndjson_line_count,
        track_count,
        "tracks",
    ));
    checks.push(check_eq_u64(
        "malformed_track_count",
        track_metrics.malformed_track_count,
        0,
        "tracks",
    ));
    checks.push(check_eq_u64(
        "unknown_family_count",
        track_metrics.unknown_family_count,
        0,
        "tracks",
    ));
    checks.push(check_eq_u64(
        "missing_field_count",
        track_metrics.missing_field_count,
        0,
        "tracks",
    ));
    checks.push(check_eq_u64(
        "track_displacement_mismatch_count",
        track_metrics.displacement_mismatches,
        0,
        "tracks",
    ));
    checks.push(check_eq_u64(
        "track_field_containment_mismatch_count",
        track_metrics.field_containment_mismatches,
        0,
        "tracks",
    ));
    checks.push(check_eq_u64(
        "track_coherence_mismatch_count",
        track_metrics.coherence_mismatches,
        0,
        "tracks",
    ));

    push_motion_checks(&mut checks, motion, "total", track_metrics.total);
    push_motion_checks(
        &mut checks,
        motion,
        "updraft_guide",
        track_metrics.updraft_guide,
    );
    push_motion_checks(
        &mut checks,
        motion,
        "updraft_ribbon",
        track_metrics.updraft_ribbon,
    );
    push_motion_checks(
        &mut checks,
        motion,
        "crosswind_guide",
        track_metrics.crosswind_guide,
    );
    push_motion_checks(
        &mut checks,
        motion,
        "crosswind_ribbon",
        track_metrics.crosswind_ribbon,
    );

    checks.push(check_at_least_u64(
        "total_coherent_tracks",
        track_metrics.total.coherent_track_count,
        MIN_TOTAL_COHERENT_TRACKS,
        "tracks",
    ));
    checks.push(check_at_least_u64(
        "updraft_guide_coherent_tracks",
        track_metrics.updraft_guide.coherent_track_count,
        MIN_UPDRAFT_GUIDE_COHERENT_TRACKS,
        "tracks",
    ));
    checks.push(check_at_least_u64(
        "updraft_ribbon_coherent_tracks",
        track_metrics.updraft_ribbon.coherent_track_count,
        MIN_UPDRAFT_RIBBON_COHERENT_TRACKS,
        "tracks",
    ));
    checks.push(check_at_least_u64(
        "crosswind_guide_coherent_tracks",
        track_metrics.crosswind_guide.coherent_track_count,
        MIN_CROSSWIND_GUIDE_COHERENT_TRACKS,
        "tracks",
    ));
    checks.push(check_at_least_u64(
        "crosswind_ribbon_coherent_tracks",
        track_metrics.crosswind_ribbon.coherent_track_count,
        MIN_CROSSWIND_RIBBON_COHERENT_TRACKS,
        "tracks",
    ));
    checks.push(check_at_most_u64(
        "static_track_count",
        track_metrics.total.static_track_count,
        MAX_STATIC_TRACKS,
        "tracks",
    ));
    checks.push(check_at_most_u64(
        "off_field_track_count",
        track_metrics.total.off_field_track_count,
        MAX_OFF_FIELD_TRACKS,
        "tracks",
    ));
    checks.push(check_at_most_u64(
        "low_alignment_track_count",
        track_metrics.total.low_alignment_track_count,
        MAX_LOW_ALIGNMENT_TRACKS,
        "tracks",
    ));
    for (name, metrics) in [
        ("updraft_guide", track_metrics.updraft_guide),
        ("updraft_ribbon", track_metrics.updraft_ribbon),
        ("crosswind_guide", track_metrics.crosswind_guide),
        ("crosswind_ribbon", track_metrics.crosswind_ribbon),
    ] {
        checks.push(check_at_least_f64(
            &format!("{name}_max_displacement"),
            metrics.max_displacement_m,
            MIN_FAMILY_MAX_DISPLACEMENT_M,
            "m",
        ));
    }

    let passed = checks.iter().all(|check| {
        check
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });

    json!({
        "schema": "nau_wind_visual_export_audit.v1",
        "manifest": manifest_path,
        "passed": passed,
        "checks": checks,
        "artifacts": {
            "track_obj_error": obj_error,
            "track_ndjson_error": ndjson_error,
            "obj_vertex_count": obj_audit.vertex_count,
            "obj_segment_count": obj_audit.segment_count,
            "ndjson_line_count": ndjson_line_count,
        },
        "metrics": {
            "total": family_metrics_json(track_metrics.total),
            "updraft_guide": family_metrics_json(track_metrics.updraft_guide),
            "updraft_ribbon": family_metrics_json(track_metrics.updraft_ribbon),
            "crosswind_guide": family_metrics_json(track_metrics.crosswind_guide),
            "crosswind_ribbon": family_metrics_json(track_metrics.crosswind_ribbon),
            "malformed_track_count": track_metrics.malformed_track_count,
            "unknown_family_count": track_metrics.unknown_family_count,
            "missing_field_count": track_metrics.missing_field_count,
            "displacement_mismatch_count": track_metrics.displacement_mismatches,
            "field_containment_mismatch_count": track_metrics.field_containment_mismatches,
            "coherence_mismatch_count": track_metrics.coherence_mismatches,
        }
    })
}

fn audit_obj_path(path: &Path) -> Result<ObjAudit, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut audit = ObjAudit::default();
    for line in text.lines() {
        if line.starts_with("v ") {
            audit.vertex_count += 1;
        } else if line.starts_with("l ") {
            audit.segment_count += 1;
        }
    }
    Ok(audit)
}

fn audit_track_ndjson(path: &Path) -> Result<(TrackMetrics, u64), String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut metrics = TrackMetrics::default();
    let mut line_count = 0;

    for line in text.lines() {
        line_count += 1;
        let value = match serde_json::from_str::<Value>(line) {
            Ok(value) => value,
            Err(_) => {
                metrics.malformed_track_count += 1;
                continue;
            }
        };
        let Some(track) = parse_track(&value) else {
            metrics.malformed_track_count += 1;
            continue;
        };
        observe_track(&mut metrics, track);
    }

    Ok((metrics, line_count))
}

fn parse_track(value: &Value) -> Option<ParsedTrack> {
    let family = intern_family(value.get("family")?.as_str()?)?;
    let sample_kind = intern_sample_kind(value.get("sample_kind")?.as_str()?)?;
    Some(ParsedTrack {
        family,
        sample_kind,
        field_index: value.get("field_index")?.as_u64()? as usize,
        elapsed_secs: value.get("elapsed_secs")?.as_f64()? as f32,
        current: value_vec3(value.get("current")?)?,
        next: value_vec3(value.get("next")?)?,
        manifest_displacement_m: value.get("displacement_m")?.as_f64()?,
        manifest_current_inside_field: value.get("current_inside_field")?.as_bool()?,
        manifest_next_inside_field: value.get("next_inside_field")?.as_bool()?,
        manifest_coherent: value.get("coherent")?.as_bool()?,
    })
}

fn observe_track(metrics: &mut TrackMetrics, track: ParsedTrack) {
    let Some((field, include_vertical, allow_center_fallback)) = track_field(track) else {
        metrics.missing_field_count += 1;
        return;
    };
    let displacement_m = track.current.distance(track.next) as f64;
    let current_inside = field.contains(track.current);
    let next_inside = field.contains(track.next);
    let alignment = visual_flow_alignment(
        field,
        track.current,
        track.next,
        track.elapsed_secs,
        include_vertical,
        allow_center_fallback,
    );
    let coherent = alignment.is_some_and(|value| value >= WIND_VISUAL_ALIGNMENT_MIN_DOT);

    if (displacement_m - track.manifest_displacement_m).abs() > 0.005 {
        metrics.displacement_mismatches += 1;
    }
    if current_inside != track.manifest_current_inside_field
        || next_inside != track.manifest_next_inside_field
    {
        metrics.field_containment_mismatches += 1;
    }
    if coherent != track.manifest_coherent {
        metrics.coherence_mismatches += 1;
    }

    let observed = ObservedTrack {
        displacement_m,
        current_inside,
        next_inside,
        coherent,
    };
    metrics.total.observe(observed);
    match track.family {
        "updraft_guide" => metrics.updraft_guide.observe(observed),
        "updraft_ribbon" => metrics.updraft_ribbon.observe(observed),
        "crosswind_guide" => metrics.crosswind_guide.observe(observed),
        "crosswind_ribbon" => metrics.crosswind_ribbon.observe(observed),
        _ => metrics.unknown_family_count += 1,
    }
}

#[derive(Clone, Copy)]
struct ObservedTrack {
    displacement_m: f64,
    current_inside: bool,
    next_inside: bool,
    coherent: bool,
}

impl FamilyMetrics {
    fn observe(&mut self, track: ObservedTrack) {
        self.track_count += 1;
        self.max_displacement_m = self.max_displacement_m.max(track.displacement_m);
        if track.displacement_m < MIN_TRACK_DISPLACEMENT_M as f64 {
            self.static_track_count += 1;
        }
        if !track.current_inside || !track.next_inside {
            self.off_field_track_count += 1;
        }
        if track.coherent {
            self.coherent_track_count += 1;
        } else {
            self.low_alignment_track_count += 1;
        }
    }
}

fn track_field(track: ParsedTrack) -> Option<(WindField, bool, bool)> {
    match (track.family, track.sample_kind) {
        ("updraft_guide", _) => updraft_field(track.field_index).map(|field| (field, true, true)),
        ("updraft_ribbon", "center") => {
            updraft_field(track.field_index).map(|field| (field, true, true))
        }
        ("updraft_ribbon", _) => updraft_field(track.field_index).map(|field| (field, true, false)),
        ("crosswind_guide", _) => {
            crosswind_field(track.field_index).map(|field| (field, false, true))
        }
        ("crosswind_ribbon", _) => {
            crosswind_field(track.field_index).map(|field| (field, false, false))
        }
        _ => None,
    }
}

fn updraft_field(index: usize) -> Option<WindField> {
    GAMEPLAY_LIFT_ROUTE
        .get(index)
        .copied()
        .map(|node| node.visual_field())
}

fn crosswind_field(index: usize) -> Option<WindField> {
    visual_crosswind_fields().get(index).copied()
}

fn visual_flow_alignment(
    field: WindField,
    current: Vec3,
    next: Vec3,
    elapsed_secs: f32,
    include_vertical: bool,
    allow_center_fallback: bool,
) -> Option<f32> {
    let displacement = next - current;
    let max_step = field.half_extents.max_element().max(1.0) * 0.5;
    if displacement.length_squared() <= 0.0001 || displacement.length() > max_step {
        return None;
    }
    if !allow_center_fallback && !field.contains(next) {
        return None;
    }

    let flow = match field.flow_at(current, elapsed_secs) {
        Some(flow) => flow,
        None if allow_center_fallback => field.flow_at(field.center, elapsed_secs)?,
        None => return None,
    };
    let motion = if include_vertical {
        displacement
    } else {
        Vec3::new(displacement.x, 0.0, displacement.z)
    };
    let flow_vector = if include_vertical {
        flow.vector
    } else {
        Vec3::new(flow.vector.x, 0.0, flow.vector.z)
    };

    if motion.length_squared() <= 0.0001 || flow_vector.length_squared() <= 0.0001 {
        return None;
    }

    Some(
        motion
            .normalize()
            .dot(flow_vector.normalize())
            .clamp(-1.0, 1.0),
    )
}

fn push_motion_checks(
    checks: &mut Vec<Value>,
    motion: &Value,
    family: &str,
    recomputed: FamilyMetrics,
) {
    let family_motion = motion.get(family).unwrap_or(&Value::Null);
    checks.push(check_eq_u64(
        &format!("{family}_track_count_manifest_matches"),
        value_u64(family_motion, "track_count"),
        recomputed.track_count,
        "tracks",
    ));
    checks.push(check_eq_u64(
        &format!("{family}_coherent_count_manifest_matches"),
        value_u64(family_motion, "coherent_track_count"),
        recomputed.coherent_track_count,
        "tracks",
    ));
    checks.push(check_eq_u64(
        &format!("{family}_static_count_manifest_matches"),
        value_u64(family_motion, "static_track_count"),
        recomputed.static_track_count,
        "tracks",
    ));
    checks.push(check_eq_u64(
        &format!("{family}_off_field_count_manifest_matches"),
        value_u64(family_motion, "off_field_track_count"),
        recomputed.off_field_track_count,
        "tracks",
    ));
}

fn value_vec3(value: &Value) -> Option<Vec3> {
    let values = value.as_array()?;
    if values.len() != 3 {
        return None;
    }
    Some(Vec3::new(
        values[0].as_f64()? as f32,
        values[1].as_f64()? as f32,
        values[2].as_f64()? as f32,
    ))
}

fn relative_path(parent: &Value, key: &str) -> Option<PathBuf> {
    parent.get(key).and_then(Value::as_str).map(PathBuf::from)
}

fn value_u64(parent: &Value, key: &str) -> u64 {
    parent.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn intern_family(value: &str) -> Option<&'static str> {
    match value {
        "updraft_guide" => Some("updraft_guide"),
        "updraft_ribbon" => Some("updraft_ribbon"),
        "crosswind_guide" => Some("crosswind_guide"),
        "crosswind_ribbon" => Some("crosswind_ribbon"),
        _ => None,
    }
}

fn intern_sample_kind(value: &str) -> Option<&'static str> {
    match value {
        "center" => Some("center"),
        "mesh" => Some("mesh"),
        _ => None,
    }
}

fn family_metrics_json(metrics: FamilyMetrics) -> Value {
    json!({
        "track_count": metrics.track_count,
        "coherent_track_count": metrics.coherent_track_count,
        "static_track_count": metrics.static_track_count,
        "off_field_track_count": metrics.off_field_track_count,
        "low_alignment_track_count": metrics.low_alignment_track_count,
        "max_displacement_m": metrics.max_displacement_m,
    })
}

fn check_eq_str(name: &str, value: &str, expected: &str, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == expected,
        "value": value,
        "comparator": "==",
        "threshold": expected,
        "unit": unit,
    })
}

fn check_eq_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_at_least_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_at_most_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value <= threshold,
        "value": value,
        "comparator": "<=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_at_least_f64(name: &str, value: f64, threshold: f64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_bool(name: &str, value: bool, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value,
        "value": value,
        "comparator": "==",
        "threshold": true,
        "unit": unit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_flow_alignment_rejects_static_track() {
        let field = GAMEPLAY_LIFT_ROUTE[0].visual_field();

        let alignment = visual_flow_alignment(field, field.center, field.center, 0.0, true, true);

        assert_eq!(alignment, None);
    }

    #[test]
    fn audit_rejects_missing_track_artifacts() {
        let manifest = json!({
            "schema": EXPECTED_SCHEMA,
            "artifacts": {},
            "counts": {
                "updraft_field_count": 2,
                "crosswind_field_count": 4,
                "updraft_guide_count": 210,
                "updraft_ribbon_count": 12,
                "crosswind_guide_count": 240,
                "crosswind_ribbon_count": 28,
                "track_count": 1164,
                "track_vertex_count": 2328,
                "track_segment_count": 1164
            },
            "coverage": {
                "updraft_fields_with_guides_and_ribbons_count": 2,
                "crosswind_fields_with_guides_and_ribbons_count": 4
            },
            "motion": {}
        });

        let report = audit_manifest(&manifest, Path::new("."), "missing.json");

        assert!(!report.get("passed").and_then(Value::as_bool).unwrap());
    }
}
