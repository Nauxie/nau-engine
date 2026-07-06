use bevy::prelude::*;
use nau_engine::environment::{
    GAMEPLAY_LIFT_ROUTE, LiftRoutePurpose, LiftRouteStage, VISUAL_CROSSWIND_FIELD_COUNT, WindField,
    visual_crosswind_fields,
};
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const EXPECTED_SCHEMA: &str = "nau_wind_visual_export.v1";
const MIN_UPDRAFT_FIELDS: u64 = GAMEPLAY_LIFT_ROUTE.len() as u64;
const MIN_CROSSWIND_FIELDS: u64 = VISUAL_CROSSWIND_FIELD_COUNT as u64;
const MIN_UPDRAFT_GUIDES: u64 = MIN_UPDRAFT_FIELDS * 105;
const MIN_UPDRAFT_RIBBONS: u64 = MIN_UPDRAFT_FIELDS * 6;
const MIN_CROSSWIND_GUIDES: u64 = MIN_CROSSWIND_FIELDS * 60;
const MIN_CROSSWIND_RIBBONS: u64 = MIN_CROSSWIND_FIELDS * 7;
const MIN_TOTAL_TRACKS: u64 = 1100;
const MIN_TOTAL_COHERENT_TRACKS: u64 = 1040;
const MIN_UPDRAFT_GUIDE_COHERENT_TRACKS: u64 = 400;
const MIN_UPDRAFT_RIBBON_COHERENT_TRACKS: u64 = 4000;
const MIN_CROSSWIND_GUIDE_COHERENT_TRACKS: u64 = 450;
const MIN_CROSSWIND_RIBBON_COHERENT_TRACKS: u64 = 125;
const MIN_LIFT_ROUTE_STAGE_COUNT: u64 = LiftRouteStage::ALL.len() as u64;
const MIN_LIFT_ROUTE_PURPOSE_COUNT: u64 = LiftRoutePurpose::ALL.len() as u64;
const MIN_CRITICAL_ROUTE_LIFT_NODES: u64 = 10;
const MIN_RECOVERY_LIFT_NODES: u64 = 5;
const MIN_OPTIONAL_DETOUR_LIFT_NODES: u64 = 3;
const MAX_STATIC_TRACK_RATIO: f64 = 0.02;
const MAX_OFF_FIELD_TRACK_RATIO: f64 = 0.001;
const MAX_LOW_ALIGNMENT_TRACK_RATIO: f64 = 0.09;
const MAX_UPDRAFT_RIBBON_LOW_ALIGNMENT_TRACK_RATIO: f64 = 0.68;
const MIN_FAMILY_MAX_DISPLACEMENT_M: f64 = 0.25;
const MIN_UPDRAFT_RIBBON_MAX_DISPLACEMENT_M: f64 = 0.16;
const MAX_UPDRAFT_GUIDE_QUALITY_VISIBLE_SPEED_MPS: f64 = 25.0;
const MAX_UPDRAFT_RIBBON_QUALITY_VISIBLE_SPEED_MPS: f64 = 18.0;
const MAX_CROSSWIND_GUIDE_QUALITY_VISIBLE_SPEED_MPS: f64 = 15.0;
const MAX_CROSSWIND_RIBBON_QUALITY_VISIBLE_SPEED_MPS: f64 = 17.0;
const MAX_UPDRAFT_GUIDE_QUALITY_VISIBLE_ACCELERATION_MPS2: f64 = 225.0;
const MAX_UPDRAFT_RIBBON_QUALITY_VISIBLE_ACCELERATION_MPS2: f64 = 500.0;
const MAX_CROSSWIND_GUIDE_QUALITY_VISIBLE_ACCELERATION_MPS2: f64 = 65.0;
const MAX_CROSSWIND_RIBBON_QUALITY_VISIBLE_ACCELERATION_MPS2: f64 = 60.0;
const MAX_TRACK_CONTINUITY_GAP_M: f64 = 0.01;
const MOTION_METRIC_TOLERANCE: f64 = 0.05;
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
    max_speed_mps: f64,
    quality_visible_track_count: u64,
    max_quality_visible_speed_mps: f64,
    max_quality_visible_acceleration_mps2: f64,
    max_acceleration_mps2: f64,
    max_continuity_gap_m: f64,
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
    visual_index: usize,
    sample_index: usize,
    interval_index: usize,
    elapsed_secs: f32,
    next_elapsed_secs: f32,
    current: Vec3,
    next: Vec3,
    manifest_displacement_m: f64,
    manifest_current_quality_visible: bool,
    manifest_next_quality_visible: bool,
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
    let authored_route = manifest.get("authored_route").unwrap_or(&Value::Null);
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
    checks.push(check_at_least_u64(
        "lift_route_stage_count",
        value_u64(authored_route, "stage_count"),
        MIN_LIFT_ROUTE_STAGE_COUNT,
        "stages",
    ));
    checks.push(check_at_least_u64(
        "lift_route_purpose_count",
        value_u64(authored_route, "purpose_count"),
        MIN_LIFT_ROUTE_PURPOSE_COUNT,
        "purposes",
    ));
    checks.push(check_eq_u64(
        "authored_lift_node_count",
        array_len(authored_route, "nodes"),
        value_u64(counts, "updraft_field_count"),
        "nodes",
    ));
    for stage in LiftRouteStage::ALL {
        checks.push(check_at_least_u64(
            &format!("lift_stage_{}", stage.label()),
            authored_route_node_count_by_label(authored_route, "stage", stage.label()),
            1,
            "nodes",
        ));
    }
    for (purpose, threshold, name) in [
        (
            LiftRoutePurpose::CriticalRoute,
            MIN_CRITICAL_ROUTE_LIFT_NODES,
            "critical_route_lift_nodes",
        ),
        (
            LiftRoutePurpose::Recovery,
            MIN_RECOVERY_LIFT_NODES,
            "recovery_lift_nodes",
        ),
        (
            LiftRoutePurpose::OptionalDetour,
            MIN_OPTIONAL_DETOUR_LIFT_NODES,
            "optional_detour_lift_nodes",
        ),
    ] {
        checks.push(check_at_least_u64(
            name,
            authored_route_node_count_by_label(authored_route, "purpose", purpose.label()),
            threshold,
            "nodes",
        ));
    }

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
    push_motion_quality_checks(&mut checks, track_metrics);

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
    checks.push(check_at_most_f64(
        "static_track_ratio",
        track_ratio(
            track_metrics.total.static_track_count,
            track_metrics.total.track_count,
        ),
        MAX_STATIC_TRACK_RATIO,
        "ratio",
    ));
    checks.push(check_at_most_f64(
        "off_field_track_ratio",
        track_ratio(
            track_metrics.total.off_field_track_count,
            track_metrics.total.track_count,
        ),
        MAX_OFF_FIELD_TRACK_RATIO,
        "ratio",
    ));
    checks.push(check_at_most_f64(
        "low_alignment_track_ratio",
        track_ratio(
            track_metrics.total.low_alignment_track_count,
            track_metrics.total.track_count,
        ),
        MAX_LOW_ALIGNMENT_TRACK_RATIO,
        "ratio",
    ));
    checks.push(check_at_most_f64(
        "updraft_ribbon_low_alignment_track_ratio",
        track_ratio(
            track_metrics.updraft_ribbon.low_alignment_track_count,
            track_metrics.updraft_ribbon.track_count,
        ),
        MAX_UPDRAFT_RIBBON_LOW_ALIGNMENT_TRACK_RATIO,
        "ratio",
    ));
    for (name, metrics, min_displacement_m) in [
        (
            "updraft_guide",
            track_metrics.updraft_guide,
            MIN_FAMILY_MAX_DISPLACEMENT_M,
        ),
        (
            "updraft_ribbon",
            track_metrics.updraft_ribbon,
            MIN_UPDRAFT_RIBBON_MAX_DISPLACEMENT_M,
        ),
        (
            "crosswind_guide",
            track_metrics.crosswind_guide,
            MIN_FAMILY_MAX_DISPLACEMENT_M,
        ),
        (
            "crosswind_ribbon",
            track_metrics.crosswind_ribbon,
            MIN_FAMILY_MAX_DISPLACEMENT_M,
        ),
    ] {
        checks.push(check_at_least_f64(
            &format!("{name}_max_displacement"),
            metrics.max_displacement_m,
            min_displacement_m,
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
    let mut observed_tracks = Vec::new();
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
        if let Some(observed) = observe_track(&mut metrics, track) {
            observed_tracks.push(observed);
        }
    }
    observe_track_pairs(&mut metrics, &observed_tracks);

    Ok((metrics, line_count))
}

fn parse_track(value: &Value) -> Option<ParsedTrack> {
    let family = intern_family(value.get("family")?.as_str()?)?;
    let sample_kind = intern_sample_kind(value.get("sample_kind")?.as_str()?)?;
    Some(ParsedTrack {
        family,
        sample_kind,
        field_index: value.get("field_index")?.as_u64()? as usize,
        visual_index: value.get("visual_index")?.as_u64()? as usize,
        sample_index: value.get("sample_index")?.as_u64()? as usize,
        interval_index: value.get("interval_index")?.as_u64()? as usize,
        elapsed_secs: value.get("elapsed_secs")?.as_f64()? as f32,
        next_elapsed_secs: value.get("next_elapsed_secs")?.as_f64()? as f32,
        current: value_vec3(value.get("current")?)?,
        next: value_vec3(value.get("next")?)?,
        manifest_displacement_m: value.get("displacement_m")?.as_f64()?,
        manifest_current_quality_visible: value.get("current_quality_visible")?.as_bool()?,
        manifest_next_quality_visible: value.get("next_quality_visible")?.as_bool()?,
        manifest_current_inside_field: value.get("current_inside_field")?.as_bool()?,
        manifest_next_inside_field: value.get("next_inside_field")?.as_bool()?,
        manifest_coherent: value.get("coherent")?.as_bool()?,
    })
}

fn observe_track(metrics: &mut TrackMetrics, track: ParsedTrack) -> Option<ObservedTrackRecord> {
    let Some((field, include_vertical, allow_center_fallback)) = track_field(track) else {
        metrics.missing_field_count += 1;
        return None;
    };
    let displacement_m = track.current.distance(track.next) as f64;
    let dt_secs = (track.next_elapsed_secs - track.elapsed_secs).max(f32::EPSILON) as f64;
    let speed_mps = displacement_m / dt_secs;
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
        speed_mps,
        quality_visible_motion: track.manifest_current_quality_visible
            && track.manifest_next_quality_visible,
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

    Some(ObservedTrackRecord {
        family: track.family,
        sample_kind: track.sample_kind,
        field_index: track.field_index,
        visual_index: track.visual_index,
        sample_index: track.sample_index,
        interval_index: track.interval_index,
        elapsed_secs: track.elapsed_secs,
        next_elapsed_secs: track.next_elapsed_secs,
        current: track.current,
        next: track.next,
        quality_visible_motion: track.manifest_current_quality_visible
            && track.manifest_next_quality_visible,
        velocity_mps: (track.next - track.current) / dt_secs as f32,
        dt_secs,
    })
}

#[derive(Clone, Copy)]
struct ObservedTrack {
    displacement_m: f64,
    speed_mps: f64,
    quality_visible_motion: bool,
    current_inside: bool,
    next_inside: bool,
    coherent: bool,
}

#[derive(Clone, Copy)]
struct ObservedTrackRecord {
    family: &'static str,
    sample_kind: &'static str,
    field_index: usize,
    visual_index: usize,
    sample_index: usize,
    interval_index: usize,
    elapsed_secs: f32,
    next_elapsed_secs: f32,
    current: Vec3,
    next: Vec3,
    quality_visible_motion: bool,
    velocity_mps: Vec3,
    dt_secs: f64,
}

impl FamilyMetrics {
    fn observe(&mut self, track: ObservedTrack) {
        self.track_count += 1;
        self.max_displacement_m = self.max_displacement_m.max(track.displacement_m);
        self.max_speed_mps = self.max_speed_mps.max(track.speed_mps);
        if track.quality_visible_motion {
            self.quality_visible_track_count += 1;
            self.max_quality_visible_speed_mps =
                self.max_quality_visible_speed_mps.max(track.speed_mps);
        }
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

    fn observe_pair(
        &mut self,
        continuity_gap_m: f64,
        acceleration_mps2: f64,
        quality_visible_pair: bool,
    ) {
        self.max_continuity_gap_m = self.max_continuity_gap_m.max(continuity_gap_m);
        self.max_acceleration_mps2 = self.max_acceleration_mps2.max(acceleration_mps2);
        if quality_visible_pair {
            self.max_quality_visible_acceleration_mps2 = self
                .max_quality_visible_acceleration_mps2
                .max(acceleration_mps2);
        }
    }
}

fn observe_track_pairs(metrics: &mut TrackMetrics, tracks: &[ObservedTrackRecord]) {
    let mut ordered = tracks.to_vec();
    ordered.sort_by(compare_observed_track_key);
    for pair in ordered.windows(2) {
        let previous = pair[0];
        let current = pair[1];
        if !same_observed_sample(previous, current)
            || current.interval_index != previous.interval_index + 1
            || (current.elapsed_secs - previous.next_elapsed_secs).abs() > f32::EPSILON
        {
            continue;
        }

        let continuity_gap_m = previous.next.distance(current.current) as f64;
        let average_dt = ((previous.dt_secs + current.dt_secs) * 0.5).max(f64::EPSILON);
        let acceleration_mps2 =
            (current.velocity_mps - previous.velocity_mps).length() as f64 / average_dt;
        let quality_visible_pair =
            previous.quality_visible_motion && current.quality_visible_motion;
        metrics
            .total
            .observe_pair(continuity_gap_m, acceleration_mps2, quality_visible_pair);
        match current.family {
            "updraft_guide" => metrics.updraft_guide.observe_pair(
                continuity_gap_m,
                acceleration_mps2,
                quality_visible_pair,
            ),
            "updraft_ribbon" => metrics.updraft_ribbon.observe_pair(
                continuity_gap_m,
                acceleration_mps2,
                quality_visible_pair,
            ),
            "crosswind_guide" => metrics.crosswind_guide.observe_pair(
                continuity_gap_m,
                acceleration_mps2,
                quality_visible_pair,
            ),
            "crosswind_ribbon" => metrics.crosswind_ribbon.observe_pair(
                continuity_gap_m,
                acceleration_mps2,
                quality_visible_pair,
            ),
            _ => metrics.unknown_family_count += 1,
        }
    }
}

fn compare_observed_track_key(
    a: &ObservedTrackRecord,
    b: &ObservedTrackRecord,
) -> std::cmp::Ordering {
    a.family
        .cmp(b.family)
        .then(a.sample_kind.cmp(b.sample_kind))
        .then(a.field_index.cmp(&b.field_index))
        .then(a.visual_index.cmp(&b.visual_index))
        .then(a.sample_index.cmp(&b.sample_index))
        .then(a.interval_index.cmp(&b.interval_index))
}

fn same_observed_sample(a: ObservedTrackRecord, b: ObservedTrackRecord) -> bool {
    a.family == b.family
        && a.sample_kind == b.sample_kind
        && a.field_index == b.field_index
        && a.visual_index == b.visual_index
        && a.sample_index == b.sample_index
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
    checks.push(check_close_f64(
        &format!("{family}_max_speed_manifest_matches"),
        value_f64(family_motion, "max_speed_mps"),
        recomputed.max_speed_mps,
        MOTION_METRIC_TOLERANCE,
        "m/s",
    ));
    checks.push(check_eq_u64(
        &format!("{family}_quality_visible_count_manifest_matches"),
        value_u64(family_motion, "quality_visible_track_count"),
        recomputed.quality_visible_track_count,
        "tracks",
    ));
    checks.push(check_close_f64(
        &format!("{family}_max_quality_visible_speed_manifest_matches"),
        value_f64(family_motion, "max_quality_visible_speed_mps"),
        recomputed.max_quality_visible_speed_mps,
        MOTION_METRIC_TOLERANCE,
        "m/s",
    ));
    checks.push(check_close_f64(
        &format!("{family}_max_quality_visible_acceleration_manifest_matches"),
        value_f64(family_motion, "max_quality_visible_acceleration_mps2"),
        recomputed.max_quality_visible_acceleration_mps2,
        MOTION_METRIC_TOLERANCE,
        "m/s^2",
    ));
    checks.push(check_close_f64(
        &format!("{family}_max_acceleration_manifest_matches"),
        value_f64(family_motion, "max_acceleration_mps2"),
        recomputed.max_acceleration_mps2,
        MOTION_METRIC_TOLERANCE,
        "m/s^2",
    ));
    checks.push(check_close_f64(
        &format!("{family}_max_continuity_gap_manifest_matches"),
        value_f64(family_motion, "max_continuity_gap_m"),
        recomputed.max_continuity_gap_m,
        MOTION_METRIC_TOLERANCE,
        "m",
    ));
}

fn push_motion_quality_checks(checks: &mut Vec<Value>, metrics: TrackMetrics) {
    for (name, family, max_quality_visible_speed_mps, max_quality_visible_acceleration_mps2) in [
        (
            "updraft_guide",
            metrics.updraft_guide,
            MAX_UPDRAFT_GUIDE_QUALITY_VISIBLE_SPEED_MPS,
            MAX_UPDRAFT_GUIDE_QUALITY_VISIBLE_ACCELERATION_MPS2,
        ),
        (
            "updraft_ribbon",
            metrics.updraft_ribbon,
            MAX_UPDRAFT_RIBBON_QUALITY_VISIBLE_SPEED_MPS,
            MAX_UPDRAFT_RIBBON_QUALITY_VISIBLE_ACCELERATION_MPS2,
        ),
        (
            "crosswind_guide",
            metrics.crosswind_guide,
            MAX_CROSSWIND_GUIDE_QUALITY_VISIBLE_SPEED_MPS,
            MAX_CROSSWIND_GUIDE_QUALITY_VISIBLE_ACCELERATION_MPS2,
        ),
        (
            "crosswind_ribbon",
            metrics.crosswind_ribbon,
            MAX_CROSSWIND_RIBBON_QUALITY_VISIBLE_SPEED_MPS,
            MAX_CROSSWIND_RIBBON_QUALITY_VISIBLE_ACCELERATION_MPS2,
        ),
    ] {
        checks.push(check_at_most_f64(
            &format!("{name}_max_quality_visible_speed"),
            family.max_quality_visible_speed_mps,
            max_quality_visible_speed_mps,
            "m/s",
        ));
        checks.push(check_at_most_f64(
            &format!("{name}_max_quality_visible_acceleration"),
            family.max_quality_visible_acceleration_mps2,
            max_quality_visible_acceleration_mps2,
            "m/s^2",
        ));
        checks.push(check_at_most_f64(
            &format!("{name}_max_continuity_gap"),
            family.max_continuity_gap_m,
            MAX_TRACK_CONTINUITY_GAP_M,
            "m",
        ));
    }
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

fn authored_route_node_count_by_label(parent: &Value, label_key: &str, label: &str) -> u64 {
    parent
        .get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter(|node| {
                    node.get(label_key)
                        .and_then(Value::as_str)
                        .is_some_and(|value| value == label)
                })
                .count() as u64
        })
        .unwrap_or(0)
}

fn array_len(parent: &Value, key: &str) -> u64 {
    parent
        .get(key)
        .and_then(Value::as_array)
        .map(|entries| entries.len() as u64)
        .unwrap_or(0)
}

fn value_u64(parent: &Value, key: &str) -> u64 {
    parent.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn value_f64(parent: &Value, key: &str) -> f64 {
    parent.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn track_ratio(count: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }

    count as f64 / total as f64
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
        "max_speed_mps": metrics.max_speed_mps,
        "quality_visible_track_count": metrics.quality_visible_track_count,
        "max_quality_visible_speed_mps": metrics.max_quality_visible_speed_mps,
        "max_quality_visible_acceleration_mps2": metrics.max_quality_visible_acceleration_mps2,
        "max_acceleration_mps2": metrics.max_acceleration_mps2,
        "max_continuity_gap_m": metrics.max_continuity_gap_m,
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

fn check_at_most_f64(name: &str, value: f64, threshold: f64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value <= threshold,
        "value": value,
        "comparator": "<=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_close_f64(name: &str, value: f64, expected: f64, tolerance: f64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": (value - expected).abs() <= tolerance,
        "value": value,
        "comparator": "~=",
        "threshold": expected,
        "tolerance": tolerance,
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

    #[test]
    fn audit_rejects_unstaged_lift_route_manifest() {
        let manifest = json!({
            "schema": EXPECTED_SCHEMA,
            "artifacts": {},
            "counts": {
                "updraft_field_count": MIN_UPDRAFT_FIELDS,
                "crosswind_field_count": MIN_CROSSWIND_FIELDS,
                "updraft_guide_count": MIN_UPDRAFT_GUIDES,
                "updraft_ribbon_count": MIN_UPDRAFT_RIBBONS,
                "crosswind_guide_count": MIN_CROSSWIND_GUIDES,
                "crosswind_ribbon_count": MIN_CROSSWIND_RIBBONS,
                "track_count": MIN_TOTAL_TRACKS,
                "track_vertex_count": MIN_TOTAL_TRACKS * 2,
                "track_segment_count": MIN_TOTAL_TRACKS
            },
            "coverage": {
                "updraft_fields_with_guides_and_ribbons_count": MIN_UPDRAFT_FIELDS,
                "crosswind_fields_with_guides_and_ribbons_count": MIN_CROSSWIND_FIELDS
            },
            "authored_route": {
                "stage_count": 1,
                "purpose_count": 1,
                "stages": [{"stage": "launch", "node_count": 1}],
                "purposes": [{"purpose": "critical_route", "node_count": 1}],
                "nodes": []
            },
            "motion": {}
        });

        let report = audit_manifest(&manifest, Path::new("."), "unstaged.json");
        let checks = report.get("checks").and_then(Value::as_array).unwrap();

        assert!(!named_check_passed(checks, "lift_route_stage_count"));
        assert!(!named_check_passed(checks, "lift_stage_under_route"));
        assert!(!named_check_passed(checks, "recovery_lift_nodes"));
        assert!(!named_check_passed(checks, "authored_lift_node_count"));
    }

    #[test]
    fn normal_motion_quality_tracks_pass_audit_gate() {
        let mut tracks = Vec::new();
        push_normal_pair(&mut tracks, "updraft_guide", "center", 0, 0, 0);
        push_normal_pair(&mut tracks, "updraft_ribbon", "center", 0, 1, 0);
        push_normal_pair(&mut tracks, "crosswind_guide", "center", 0, 0, 0);
        push_normal_pair(&mut tracks, "crosswind_ribbon", "mesh", 0, 1, 0);

        let metrics = metrics_for_tracks(&tracks);
        let checks = motion_quality_checks(metrics);

        assert!(checks.iter().all(check_passed));
        assert_eq!(metrics.total.max_continuity_gap_m, 0.0);
    }

    #[test]
    fn artificial_jump_speed_and_acceleration_regression_fails_motion_quality_gate() {
        let field = crosswind_field(0).expect("test crosswind field should exist");
        let axis = field.direction;
        let start = field.center - axis * 4.0;
        let id = TestTrackId {
            family: "crosswind_ribbon",
            sample_kind: "mesh",
            field_index: 0,
            visual_index: 0,
            sample_index: 0,
        };
        let tracks = [
            parsed_track(id, 0, 0.0, start, start + axis),
            parsed_track(id, 1, 0.2, start + axis * 3.0, start + axis * 8.0),
        ];

        let metrics = metrics_for_tracks(&tracks);
        let checks = motion_quality_checks(metrics);

        assert!(!named_check_passed(&checks, "crosswind_ribbon_max_speed"));
        assert!(!named_check_passed(
            &checks,
            "crosswind_ribbon_max_acceleration"
        ));
        assert!(!named_check_passed(
            &checks,
            "crosswind_ribbon_max_continuity_gap"
        ));
    }

    fn push_normal_pair(
        tracks: &mut Vec<ParsedTrack>,
        family: &'static str,
        sample_kind: &'static str,
        field_index: usize,
        visual_index: usize,
        sample_index: usize,
    ) {
        let field = if family.starts_with("updraft") {
            updraft_field(field_index).expect("test updraft field should exist")
        } else {
            crosswind_field(field_index).expect("test crosswind field should exist")
        };
        let axis = if family.starts_with("updraft") {
            Vec3::Y
        } else {
            field.direction
        };
        let id = TestTrackId {
            family,
            sample_kind,
            field_index,
            visual_index,
            sample_index,
        };
        let start = field.center - axis * 0.5;
        tracks.push(parsed_track(id, 0, 0.0, start, start + axis));
        tracks.push(parsed_track(id, 1, 0.2, start + axis, start + axis * 2.0));
    }

    #[derive(Clone, Copy)]
    struct TestTrackId {
        family: &'static str,
        sample_kind: &'static str,
        field_index: usize,
        visual_index: usize,
        sample_index: usize,
    }

    fn parsed_track(
        id: TestTrackId,
        interval_index: usize,
        elapsed_secs: f32,
        current: Vec3,
        next: Vec3,
    ) -> ParsedTrack {
        let mut track = ParsedTrack {
            family: id.family,
            sample_kind: id.sample_kind,
            field_index: id.field_index,
            visual_index: id.visual_index,
            sample_index: id.sample_index,
            interval_index,
            elapsed_secs,
            next_elapsed_secs: elapsed_secs + 0.2,
            current,
            next,
            manifest_displacement_m: current.distance(next) as f64,
            manifest_current_quality_visible: true,
            manifest_next_quality_visible: true,
            manifest_current_inside_field: false,
            manifest_next_inside_field: false,
            manifest_coherent: false,
        };
        let (field, include_vertical, allow_center_fallback) =
            track_field(track).expect("test track should map to a field");
        track.manifest_current_inside_field = field.contains(current);
        track.manifest_next_inside_field = field.contains(next);
        track.manifest_coherent = visual_flow_alignment(
            field,
            current,
            next,
            elapsed_secs,
            include_vertical,
            allow_center_fallback,
        )
        .is_some_and(|value| value >= WIND_VISUAL_ALIGNMENT_MIN_DOT);
        track
    }

    fn metrics_for_tracks(tracks: &[ParsedTrack]) -> TrackMetrics {
        let mut metrics = TrackMetrics::default();
        let mut observed = Vec::new();
        for track in tracks {
            if let Some(observed_track) = observe_track(&mut metrics, *track) {
                observed.push(observed_track);
            }
        }
        observe_track_pairs(&mut metrics, &observed);
        metrics
    }

    fn motion_quality_checks(metrics: TrackMetrics) -> Vec<Value> {
        let mut checks = Vec::new();
        push_motion_quality_checks(&mut checks, metrics);
        checks
    }

    fn named_check_passed(checks: &[Value], name: &str) -> bool {
        checks
            .iter()
            .find(|check| check.get("name").and_then(Value::as_str) == Some(name))
            .and_then(|check| check.get("passed").and_then(Value::as_bool))
            .unwrap_or(false)
    }

    fn check_passed(check: &Value) -> bool {
        check
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
}
