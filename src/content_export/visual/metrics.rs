use super::types::VisualMeshSummary;
use crate::content_export::shared::{mesh_index_values, mesh_positions};
use bevy::prelude::*;
use std::path::PathBuf;

pub(super) fn visual_content_mesh_summary(obj_path: PathBuf, mesh: &Mesh) -> VisualMeshSummary {
    let (horizontal_span_m, vertical_span_m, depth_span_m) = mesh_bounds(mesh)
        .map_or((0.0, 0.0, 0.0), |(min, max)| {
            (max.x - min.x, max.y - min.y, max.z - min.z)
        });

    VisualMeshSummary {
        obj_path,
        vertex_count: mesh.count_vertices(),
        triangle_count: mesh_index_values(mesh).len() / 3,
        horizontal_span_m,
        vertical_span_m,
        depth_span_m,
    }
}

fn mesh_bounds(mesh: &Mesh) -> Option<(Vec3, Vec3)> {
    let positions = mesh_positions(mesh);
    let first = positions.first()?;
    let mut min = Vec3::from_array(*first);
    let mut max = min;

    for position in positions.iter().skip(1) {
        let position = Vec3::from_array(*position);
        min = min.min(position);
        max = max.max(position);
    }

    Some((min, max))
}
pub(super) fn min_finite_f32(values: impl Iterator<Item = f32>) -> f32 {
    values
        .filter(|value| value.is_finite())
        .min_by(f32::total_cmp)
        .unwrap_or(0.0)
}

pub(super) fn finite_ratio(numerator: f32, denominator: f32) -> f32 {
    if denominator.abs() <= f32::EPSILON {
        return 0.0;
    }
    let ratio = numerator / denominator;
    if ratio.is_finite() { ratio } else { 0.0 }
}
