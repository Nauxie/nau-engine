use super::super::EvalAccumulator;
use crate::eval::scenarios::EvalScenario;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct EvalFrameTimeStats {
    pub(super) sample_count: u32,
    pub(super) avg_ms: f32,
    pub(super) p95_ms: f32,
    pub(super) p99_ms: f32,
    pub(super) max_ms: f32,
}

impl EvalFrameTimeStats {
    fn from_samples(samples: &[f32]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(f32::total_cmp);

        let sum: f32 = sorted.iter().sum();
        Self {
            sample_count: sorted.len() as u32,
            avg_ms: sum / sorted.len() as f32,
            p95_ms: percentile(&sorted, 0.95),
            p99_ms: percentile(&sorted, 0.99),
            max_ms: *sorted.last().unwrap_or(&0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SummaryDerivedMetrics {
    pub(super) horizontal_distance_m: f32,
    pub(super) final_target_distance_m: f32,
    pub(super) final_objective_completed_count: usize,
    pub(super) final_objective_distance_m: f32,
    pub(super) frame_time_stats: EvalFrameTimeStats,
    pub(super) runtime_frame_time_stats: EvalFrameTimeStats,
    pub(super) eval_artifact_frame_time_stats: EvalFrameTimeStats,
    pub(super) avg_desired_body_heading_error_degrees: f32,
    pub(super) p95_desired_body_heading_error_degrees: f32,
    pub(super) p95_lateral_body_travel_heading_error_degrees: f32,
    pub(super) p95_backward_diagonal_body_travel_heading_error_degrees: f32,
    pub(super) p95_desired_travel_heading_error_degrees: f32,
    pub(super) p95_pure_air_turn_sideways_body_travel_heading_error_degrees: f32,
    pub(super) p95_pure_air_turn_sideways_desired_travel_heading_error_degrees: f32,
    pub(super) avg_camera_follow_direction_error_degrees: f32,
    pub(super) p95_camera_follow_direction_error_degrees: f32,
    pub(super) lateral_response_latency_secs: f32,
    pub(super) right_lateral_response_latency_secs: f32,
    pub(super) left_lateral_response_latency_secs: f32,
    pub(super) backward_lateral_response_latency_secs: f32,
    pub(super) backward_right_lateral_response_latency_secs: f32,
    pub(super) backward_left_lateral_response_latency_secs: f32,
}

impl SummaryDerivedMetrics {
    pub(super) fn from_accumulator(acc: &EvalAccumulator, scenario: EvalScenario) -> Self {
        let horizontal_distance_m = match (&acc.first_sample, &acc.final_sample) {
            (Some(first), Some(final_sample)) => {
                horizontal_distance(first.position, final_sample.position)
            }
            _ => 0.0,
        };
        let final_target_distance_m = acc
            .final_sample
            .as_ref()
            .map_or(0.0, |sample| sample.target_distance_m);
        let final_objective_completed_count = acc
            .final_sample
            .as_ref()
            .map_or(0, |sample| sample.objective.completed_count);
        let final_objective_distance_m = acc
            .final_sample
            .as_ref()
            .map_or(0.0, |sample| sample.objective.current_distance_m);
        let frame_time_stats = EvalFrameTimeStats::from_samples(&acc.frame_times_ms);
        let runtime_frame_time_stats =
            EvalFrameTimeStats::from_samples(&acc.runtime_frame_times_ms);
        let eval_artifact_frame_time_stats =
            EvalFrameTimeStats::from_samples(&acc.eval_artifact_frame_times_ms);
        let avg_desired_body_heading_error_degrees = if acc.desired_body_heading_samples == 0 {
            0.0
        } else {
            acc.desired_body_heading_error_sum_degrees / acc.desired_body_heading_samples as f32
        };
        let mut desired_body_heading_error_values_degrees =
            acc.desired_body_heading_error_values_degrees.clone();
        desired_body_heading_error_values_degrees.sort_by(f32::total_cmp);
        let p95_desired_body_heading_error_degrees =
            percentile(&desired_body_heading_error_values_degrees, 0.95);
        let mut lateral_body_travel_heading_error_values_degrees =
            acc.lateral_body_travel_heading_error_values_degrees.clone();
        lateral_body_travel_heading_error_values_degrees.sort_by(f32::total_cmp);
        let p95_lateral_body_travel_heading_error_degrees =
            percentile(&lateral_body_travel_heading_error_values_degrees, 0.95);
        let mut backward_diagonal_body_travel_heading_error_values_degrees = acc
            .backward_diagonal_body_travel_heading_error_values_degrees
            .clone();
        backward_diagonal_body_travel_heading_error_values_degrees.sort_by(f32::total_cmp);
        let p95_backward_diagonal_body_travel_heading_error_degrees = percentile(
            &backward_diagonal_body_travel_heading_error_values_degrees,
            0.95,
        );
        let mut desired_travel_heading_error_values_degrees =
            acc.desired_travel_heading_error_values_degrees.clone();
        desired_travel_heading_error_values_degrees.sort_by(f32::total_cmp);
        let p95_desired_travel_heading_error_degrees =
            percentile(&desired_travel_heading_error_values_degrees, 0.95);
        let mut pure_air_turn_sideways_body_travel_heading_error_values_degrees = acc
            .pure_air_turn_sideways_body_travel_heading_error_values_degrees
            .clone();
        pure_air_turn_sideways_body_travel_heading_error_values_degrees.sort_by(f32::total_cmp);
        let p95_pure_air_turn_sideways_body_travel_heading_error_degrees = percentile(
            &pure_air_turn_sideways_body_travel_heading_error_values_degrees,
            0.95,
        );
        let mut pure_air_turn_sideways_desired_travel_heading_error_values_degrees = acc
            .pure_air_turn_sideways_desired_travel_heading_error_values_degrees
            .clone();
        pure_air_turn_sideways_desired_travel_heading_error_values_degrees.sort_by(f32::total_cmp);
        let p95_pure_air_turn_sideways_desired_travel_heading_error_degrees = percentile(
            &pure_air_turn_sideways_desired_travel_heading_error_values_degrees,
            0.95,
        );
        let avg_camera_follow_direction_error_degrees =
            if acc.camera_follow_direction_error_samples == 0 {
                0.0
            } else {
                acc.camera_follow_direction_error_sum_degrees
                    / acc.camera_follow_direction_error_samples as f32
            };
        let mut camera_follow_direction_error_values_degrees =
            acc.camera_follow_direction_error_values_degrees.clone();
        camera_follow_direction_error_values_degrees.sort_by(f32::total_cmp);
        let p95_camera_follow_direction_error_degrees =
            percentile(&camera_follow_direction_error_values_degrees, 0.95);

        Self {
            horizontal_distance_m,
            final_target_distance_m,
            final_objective_completed_count,
            final_objective_distance_m,
            frame_time_stats,
            runtime_frame_time_stats,
            eval_artifact_frame_time_stats,
            avg_desired_body_heading_error_degrees,
            p95_desired_body_heading_error_degrees,
            p95_lateral_body_travel_heading_error_degrees,
            p95_backward_diagonal_body_travel_heading_error_degrees,
            p95_desired_travel_heading_error_degrees,
            p95_pure_air_turn_sideways_body_travel_heading_error_degrees,
            p95_pure_air_turn_sideways_desired_travel_heading_error_degrees,
            avg_camera_follow_direction_error_degrees,
            p95_camera_follow_direction_error_degrees,
            lateral_response_latency_secs: response_latency_secs(
                acc.first_lateral_input_time_secs,
                acc.first_lateral_response_time_secs,
                scenario,
            ),
            right_lateral_response_latency_secs: response_latency_secs(
                acc.first_right_lateral_input_time_secs,
                acc.first_right_lateral_response_time_secs,
                scenario,
            ),
            left_lateral_response_latency_secs: response_latency_secs(
                acc.first_left_lateral_input_time_secs,
                acc.first_left_lateral_response_time_secs,
                scenario,
            ),
            backward_lateral_response_latency_secs: response_latency_secs(
                acc.first_backward_lateral_input_time_secs,
                acc.first_backward_lateral_response_time_secs,
                scenario,
            ),
            backward_right_lateral_response_latency_secs: response_latency_secs(
                acc.first_backward_right_lateral_input_time_secs,
                acc.first_backward_right_lateral_response_time_secs,
                scenario,
            ),
            backward_left_lateral_response_latency_secs: response_latency_secs(
                acc.first_backward_left_lateral_input_time_secs,
                acc.first_backward_left_lateral_response_time_secs,
                scenario,
            ),
        }
    }
}

fn response_latency_secs(
    input_time_secs: Option<f32>,
    response_time_secs: Option<f32>,
    scenario: EvalScenario,
) -> f32 {
    match (input_time_secs, response_time_secs) {
        (Some(input_time), Some(response_time)) => (response_time - input_time).max(0.0),
        (Some(_), None) => scenario.duration_secs(),
        _ => 0.0,
    }
}

fn horizontal_distance(start: [f32; 3], end: [f32; 3]) -> f32 {
    let dx = end[0] - start[0];
    let dz = end[2] - start[2];
    (dx * dx + dz * dz).sqrt()
}

fn percentile(sorted_values: &[f32], percentile: f32) -> f32 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = ((sorted_values.len() as f32 * percentile).ceil() as usize)
        .saturating_sub(1)
        .min(sorted_values.len() - 1);
    sorted_values[index]
}
