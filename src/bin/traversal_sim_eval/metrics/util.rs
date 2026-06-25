use bevy::prelude::{Vec2, Vec3};

use super::super::SimSample;
use super::SimMetrics;

pub(super) fn avg_body_heading_error_degrees(metrics: &SimMetrics) -> f32 {
    if metrics.desired_body_heading_samples == 0 {
        0.0
    } else {
        metrics.desired_body_heading_error_sum_degrees / metrics.desired_body_heading_samples as f32
    }
}

pub(super) fn p95_body_heading_error_degrees(metrics: &SimMetrics) -> f32 {
    percentile(&metrics.desired_body_heading_error_values_degrees, 0.95)
}

pub(super) fn backward_diagonal_rear_response_mps(sample: &SimSample) -> Option<f32> {
    if sample.movement_input_forward_axis >= 0.0
        || sample.movement_input_lateral_axis.abs() <= f32::EPSILON
        || !sample.desired_heading_alignment_mps.is_finite()
        || !sample.lateral_response_mps.is_finite()
    {
        return None;
    }

    Some(
        sample.desired_heading_alignment_mps * std::f32::consts::SQRT_2
            - sample.lateral_response_mps,
    )
}

pub(super) fn response_latency_secs(
    input_time_secs: Option<f32>,
    response_time_secs: Option<f32>,
) -> f32 {
    match (input_time_secs, response_time_secs) {
        (Some(input_time), Some(response_time)) => (response_time - input_time).max(0.0),
        (Some(_), None) => 999.0,
        _ => 0.0,
    }
}

pub(super) fn percentile(values: &[f32], percentile: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let index =
        ((sorted.len().saturating_sub(1)) as f32 * percentile.clamp(0.0, 1.0)).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

pub(super) fn horizontal_distance(left: Vec3, right: Vec3) -> f32 {
    Vec2::new(left.x - right.x, left.z - right.z).length()
}
