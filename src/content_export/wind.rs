use crate::{
    environment_visuals::{
        CROSSWIND_GUIDES_PER_FIELD, CROSSWIND_RIBBONS_PER_FIELD, CrosswindGuide, CrosswindRibbon,
        UPDRAFT_GUIDE_RING_LEVELS, UPDRAFT_GUIDES_PER_RING, UPDRAFT_RIBBONS_PER_FIELD,
        UpdraftGuide, UpdraftRibbon, WIND_VISUAL_ALIGNMENT_MIN_DOT, WIND_VISUAL_COHERENCE_DT,
        crosswind_guide_position, crosswind_ribbon_scene_sample_positions,
        crosswind_ribbon_transform, updraft_guide_position, updraft_ribbon_scene_sample_positions,
        updraft_ribbon_transform, visual_flow_alignment,
    },
    eval_runtime::{path_string, remove_existing_dir},
};
use bevy::prelude::*;
use nau_engine::environment::{
    GAMEPLAY_LIFT_ROUTE, VISUAL_CROSSWIND_FIELD_COUNT, WindField, visual_crosswind_fields,
};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

const TRACK_TIMES_SECS: [f32; 3] = [
    0.0,
    WIND_VISUAL_COHERENCE_DT,
    WIND_VISUAL_COHERENCE_DT * 2.0,
];
const MIN_TRACK_DISPLACEMENT_M: f32 = 0.01;

#[derive(Debug)]
pub(crate) struct WindVisualExportReport {
    pub(crate) manifest_path: PathBuf,
    pub(crate) track_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct WindVisualTrack {
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
    displacement_m: f32,
    alignment: Option<f32>,
    current_inside_field: bool,
    next_inside_field: bool,
    coherent: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct FamilyMetrics {
    track_count: usize,
    coherent_track_count: usize,
    static_track_count: usize,
    off_field_track_count: usize,
    low_alignment_track_count: usize,
    max_displacement_m: f32,
    min_alignment: f32,
    max_alignment: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct MotionSummary {
    total: FamilyMetrics,
    updraft_guide: FamilyMetrics,
    updraft_ribbon: FamilyMetrics,
    crosswind_guide: FamilyMetrics,
    crosswind_ribbon: FamilyMetrics,
}

#[derive(Clone, Copy, Debug)]
struct TrackSpec {
    family: &'static str,
    sample_kind: &'static str,
    field_index: usize,
    visual_index: usize,
    field: WindField,
    include_vertical: bool,
    allow_center_fallback: bool,
}

#[derive(Clone, Copy, Debug)]
struct WindVisualCounts {
    updraft_field_count: usize,
    crosswind_field_count: usize,
    updraft_guide_count: usize,
    updraft_ribbon_count: usize,
    crosswind_guide_count: usize,
    crosswind_ribbon_count: usize,
    track_count: usize,
}

struct WindVisualManifestInput<'a> {
    track_obj_relative: &'a Path,
    track_ndjson_relative: &'a Path,
    counts: WindVisualCounts,
    summary: MotionSummary,
}

pub(crate) fn export_wind_visual_inspection(
    output_dir: &Path,
) -> std::io::Result<WindVisualExportReport> {
    fs::create_dir_all(output_dir)?;
    let tracks_dir = output_dir.join("wind_tracks");
    remove_existing_dir(&tracks_dir)?;
    fs::create_dir_all(&tracks_dir)?;

    let tracks = wind_visual_tracks();
    let summary = summarize_tracks(&tracks);
    let track_obj_relative = PathBuf::from("wind_tracks").join("wind_visual_tracks.obj");
    let track_ndjson_relative = PathBuf::from("wind_tracks").join("wind_visual_tracks.ndjson");
    let track_obj_path = output_dir.join(&track_obj_relative);
    let track_ndjson_path = output_dir.join(&track_ndjson_relative);

    write_track_obj(&track_obj_path, &tracks)?;
    write_track_ndjson(&track_ndjson_path, &tracks)?;

    let updraft_field_count = GAMEPLAY_LIFT_ROUTE.len();
    let crosswind_field_count = VISUAL_CROSSWIND_FIELD_COUNT;
    let updraft_guide_count =
        updraft_field_count * UPDRAFT_GUIDE_RING_LEVELS.len() * UPDRAFT_GUIDES_PER_RING;
    let updraft_ribbon_count = updraft_field_count * UPDRAFT_RIBBONS_PER_FIELD;
    let crosswind_guide_count = crosswind_field_count * CROSSWIND_GUIDES_PER_FIELD;
    let crosswind_ribbon_count = crosswind_field_count * CROSSWIND_RIBBONS_PER_FIELD;
    let manifest_path = output_dir.join("manifest.json");
    let manifest = wind_visual_manifest(WindVisualManifestInput {
        track_obj_relative: &track_obj_relative,
        track_ndjson_relative: &track_ndjson_relative,
        counts: WindVisualCounts {
            updraft_field_count,
            crosswind_field_count,
            updraft_guide_count,
            updraft_ribbon_count,
            crosswind_guide_count,
            crosswind_ribbon_count,
            track_count: tracks.len(),
        },
        summary,
    });
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("wind visual manifest should serialize"),
    )?;

    Ok(WindVisualExportReport {
        manifest_path,
        track_count: tracks.len(),
    })
}

fn wind_visual_tracks() -> Vec<WindVisualTrack> {
    let mut tracks = Vec::new();

    for (field_index, node) in GAMEPLAY_LIFT_ROUTE.iter().copied().enumerate() {
        let field = node.visual_field();
        let ring_radius = field.half_extents.x.min(field.half_extents.z) * 0.5;
        for ribbon_index in 0..UPDRAFT_RIBBONS_PER_FIELD {
            let phase = ribbon_index as f32 / UPDRAFT_RIBBONS_PER_FIELD as f32;
            let mesh_phase = phase * std::f32::consts::TAU;
            let ribbon = UpdraftRibbon {
                field,
                spin_speed: 0.072 + ribbon_index as f32 * 0.018,
                base_translation: field.center,
                base_rotation: Quat::from_rotation_y(mesh_phase * 0.35),
                phase,
            };
            push_transform_tracks(
                &mut tracks,
                TrackSpec {
                    family: "updraft_ribbon",
                    sample_kind: "center",
                    field_index,
                    visual_index: ribbon_index,
                    field,
                    include_vertical: true,
                    allow_center_fallback: true,
                },
                |elapsed| updraft_ribbon_transform(&ribbon, elapsed).translation,
            );
            push_sampled_transform_tracks(
                &mut tracks,
                TrackSpec {
                    family: "updraft_ribbon",
                    sample_kind: "mesh",
                    field_index,
                    visual_index: ribbon_index,
                    field,
                    include_vertical: true,
                    allow_center_fallback: false,
                },
                |elapsed| {
                    let transform = updraft_ribbon_transform(&ribbon, elapsed);
                    updraft_ribbon_scene_sample_positions(&ribbon, &transform)
                },
            );
        }

        for (level_index, level) in UPDRAFT_GUIDE_RING_LEVELS.into_iter().enumerate() {
            for marker_index in 0..UPDRAFT_GUIDES_PER_RING {
                let phase = marker_index as f32 / UPDRAFT_GUIDES_PER_RING as f32
                    * std::f32::consts::TAU
                    + level_index as f32 * 0.46;
                let guide = UpdraftGuide {
                    field,
                    center: field.center,
                    radius: ring_radius,
                    height_offset: level * field.half_extents.y,
                    phase,
                    angular_speed: 0.26 + level_index as f32 * 0.035,
                };
                let visual_index = level_index * UPDRAFT_GUIDES_PER_RING + marker_index;
                push_transform_tracks(
                    &mut tracks,
                    TrackSpec {
                        family: "updraft_guide",
                        sample_kind: "center",
                        field_index,
                        visual_index,
                        field,
                        include_vertical: true,
                        allow_center_fallback: true,
                    },
                    |elapsed| updraft_guide_position(&guide, elapsed),
                );
            }
        }
    }

    for (field_index, field) in visual_crosswind_fields().into_iter().enumerate() {
        for ribbon_index in 0..CROSSWIND_RIBBONS_PER_FIELD {
            let phase = ribbon_index as f32 / CROSSWIND_RIBBONS_PER_FIELD as f32;
            let origin = field.stream_origin(ribbon_index, CROSSWIND_RIBBONS_PER_FIELD);
            let ribbon = CrosswindRibbon {
                field,
                base_translation: origin + field.direction * (field.half_extents.x * 0.62),
                phase,
            };
            push_sampled_transform_tracks(
                &mut tracks,
                TrackSpec {
                    family: "crosswind_ribbon",
                    sample_kind: "mesh",
                    field_index,
                    visual_index: ribbon_index,
                    field,
                    include_vertical: false,
                    allow_center_fallback: false,
                },
                |elapsed| {
                    let transform = crosswind_ribbon_transform(&ribbon, elapsed);
                    crosswind_ribbon_scene_sample_positions(&ribbon, &transform)
                },
            );
        }

        for stream_index in 0..CROSSWIND_GUIDES_PER_FIELD {
            let guide = CrosswindGuide {
                field,
                stream_index,
                stream_count: CROSSWIND_GUIDES_PER_FIELD,
                phase: (stream_index as f32 * 0.381_966).fract(),
            };
            push_transform_tracks(
                &mut tracks,
                TrackSpec {
                    family: "crosswind_guide",
                    sample_kind: "center",
                    field_index,
                    visual_index: stream_index,
                    field,
                    include_vertical: false,
                    allow_center_fallback: true,
                },
                |elapsed| crosswind_guide_position(&guide, elapsed),
            );
        }
    }

    tracks
}

fn push_transform_tracks(
    tracks: &mut Vec<WindVisualTrack>,
    spec: TrackSpec,
    position_at: impl Fn(f32) -> Vec3,
) {
    for interval_index in 0..TRACK_TIMES_SECS.len() - 1 {
        let elapsed_secs = TRACK_TIMES_SECS[interval_index];
        let next_elapsed_secs = TRACK_TIMES_SECS[interval_index + 1];
        tracks.push(make_track(
            spec,
            0,
            interval_index,
            elapsed_secs,
            next_elapsed_secs,
            position_at(elapsed_secs),
            position_at(next_elapsed_secs),
        ));
    }
}

fn push_sampled_transform_tracks(
    tracks: &mut Vec<WindVisualTrack>,
    spec: TrackSpec,
    positions_at: impl Fn(f32) -> [Vec3; 3],
) {
    for interval_index in 0..TRACK_TIMES_SECS.len() - 1 {
        let elapsed_secs = TRACK_TIMES_SECS[interval_index];
        let next_elapsed_secs = TRACK_TIMES_SECS[interval_index + 1];
        let current_positions = positions_at(elapsed_secs);
        let next_positions = positions_at(next_elapsed_secs);
        for (sample_index, (current, next)) in current_positions
            .into_iter()
            .zip(next_positions)
            .enumerate()
        {
            tracks.push(make_track(
                spec,
                sample_index,
                interval_index,
                elapsed_secs,
                next_elapsed_secs,
                current,
                next,
            ));
        }
    }
}

fn make_track(
    spec: TrackSpec,
    sample_index: usize,
    interval_index: usize,
    elapsed_secs: f32,
    next_elapsed_secs: f32,
    current: Vec3,
    next: Vec3,
) -> WindVisualTrack {
    let alignment = visual_flow_alignment(
        spec.field,
        current,
        next,
        elapsed_secs,
        spec.include_vertical,
        spec.allow_center_fallback,
    );
    WindVisualTrack {
        family: spec.family,
        sample_kind: spec.sample_kind,
        field_index: spec.field_index,
        visual_index: spec.visual_index,
        sample_index,
        interval_index,
        elapsed_secs,
        next_elapsed_secs,
        current,
        next,
        displacement_m: current.distance(next),
        alignment,
        current_inside_field: spec.field.contains(current),
        next_inside_field: spec.field.contains(next),
        coherent: alignment.is_some_and(|value| value >= WIND_VISUAL_ALIGNMENT_MIN_DOT),
    }
}

fn summarize_tracks(tracks: &[WindVisualTrack]) -> MotionSummary {
    let mut summary = MotionSummary::default();
    for track in tracks {
        summary.total.observe(track);
        match track.family {
            "updraft_guide" => summary.updraft_guide.observe(track),
            "updraft_ribbon" => summary.updraft_ribbon.observe(track),
            "crosswind_guide" => summary.crosswind_guide.observe(track),
            "crosswind_ribbon" => summary.crosswind_ribbon.observe(track),
            _ => {}
        }
    }
    summary
}

impl FamilyMetrics {
    fn observe(&mut self, track: &WindVisualTrack) {
        self.track_count += 1;
        self.max_displacement_m = self.max_displacement_m.max(track.displacement_m);
        if track.displacement_m < MIN_TRACK_DISPLACEMENT_M {
            self.static_track_count += 1;
        }
        if !track.current_inside_field || !track.next_inside_field {
            self.off_field_track_count += 1;
        }
        if track.coherent {
            self.coherent_track_count += 1;
        } else {
            self.low_alignment_track_count += 1;
        }
        if let Some(alignment) = track.alignment {
            if self.track_count == 1 {
                self.min_alignment = alignment;
                self.max_alignment = alignment;
            } else {
                self.min_alignment = self.min_alignment.min(alignment);
                self.max_alignment = self.max_alignment.max(alignment);
            }
        }
    }
}

fn wind_visual_manifest(input: WindVisualManifestInput<'_>) -> Value {
    let counts = input.counts;
    let summary = input.summary;
    json!({
        "schema": "nau_wind_visual_export.v1",
        "artifacts": {
            "track_obj": path_string(input.track_obj_relative),
            "track_ndjson": path_string(input.track_ndjson_relative),
        },
        "sample_times_secs": TRACK_TIMES_SECS,
        "thresholds": {
            "coherence_dt_secs": WIND_VISUAL_COHERENCE_DT,
            "alignment_min_dot": WIND_VISUAL_ALIGNMENT_MIN_DOT,
            "min_track_displacement_m": MIN_TRACK_DISPLACEMENT_M,
        },
        "counts": {
            "updraft_field_count": counts.updraft_field_count,
            "crosswind_field_count": counts.crosswind_field_count,
            "total_field_count": counts.updraft_field_count + counts.crosswind_field_count,
            "updraft_guide_count": counts.updraft_guide_count,
            "updraft_ribbon_count": counts.updraft_ribbon_count,
            "crosswind_guide_count": counts.crosswind_guide_count,
            "crosswind_ribbon_count": counts.crosswind_ribbon_count,
            "total_visual_count": counts.updraft_guide_count + counts.updraft_ribbon_count + counts.crosswind_guide_count + counts.crosswind_ribbon_count,
            "track_count": counts.track_count,
            "track_vertex_count": counts.track_count * 2,
            "track_segment_count": counts.track_count,
        },
        "coverage": {
            "updraft_fields_with_guides_count": counts.updraft_field_count,
            "updraft_fields_with_ribbons_count": counts.updraft_field_count,
            "updraft_fields_with_guides_and_ribbons_count": counts.updraft_field_count,
            "crosswind_fields_with_guides_count": counts.crosswind_field_count,
            "crosswind_fields_with_ribbons_count": counts.crosswind_field_count,
            "crosswind_fields_with_guides_and_ribbons_count": counts.crosswind_field_count,
        },
        "motion": {
            "total": family_metrics_json(summary.total),
            "updraft_guide": family_metrics_json(summary.updraft_guide),
            "updraft_ribbon": family_metrics_json(summary.updraft_ribbon),
            "crosswind_guide": family_metrics_json(summary.crosswind_guide),
            "crosswind_ribbon": family_metrics_json(summary.crosswind_ribbon),
        },
    })
}

fn family_metrics_json(metrics: FamilyMetrics) -> Value {
    json!({
        "track_count": metrics.track_count,
        "coherent_track_count": metrics.coherent_track_count,
        "static_track_count": metrics.static_track_count,
        "off_field_track_count": metrics.off_field_track_count,
        "low_alignment_track_count": metrics.low_alignment_track_count,
        "max_displacement_m": metrics.max_displacement_m,
        "min_alignment": metrics.min_alignment,
        "max_alignment": metrics.max_alignment,
    })
}

fn write_track_obj(path: &Path, tracks: &[WindVisualTrack]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# NAU wind visual motion tracks")?;
    writeln!(file, "# each line segment is one t->t+dt visual sample")?;
    for track in tracks {
        writeln!(
            file,
            "v {:.4} {:.4} {:.4}",
            track.current.x, track.current.y, track.current.z
        )?;
        writeln!(
            file,
            "v {:.4} {:.4} {:.4}",
            track.next.x, track.next.y, track.next.z
        )?;
    }
    for index in 0..tracks.len() {
        let start = index * 2 + 1;
        writeln!(file, "l {} {}", start, start + 1)?;
    }
    Ok(())
}

fn write_track_ndjson(path: &Path, tracks: &[WindVisualTrack]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    for track in tracks {
        writeln!(
            file,
            "{}",
            serde_json::to_string(&track.to_json()).expect("wind visual track should serialize")
        )?;
    }
    Ok(())
}

impl WindVisualTrack {
    fn to_json(self) -> Value {
        json!({
            "family": self.family,
            "sample_kind": self.sample_kind,
            "field_index": self.field_index,
            "visual_index": self.visual_index,
            "sample_index": self.sample_index,
            "interval_index": self.interval_index,
            "elapsed_secs": self.elapsed_secs,
            "next_elapsed_secs": self.next_elapsed_secs,
            "current": vec3_json(self.current),
            "next": vec3_json(self.next),
            "displacement_m": self.displacement_m,
            "alignment": self.alignment,
            "current_inside_field": self.current_inside_field,
            "next_inside_field": self.next_inside_field,
            "coherent": self.coherent,
        })
    }
}

fn vec3_json(value: Vec3) -> Value {
    json!([value.x, value.y, value.z])
}
