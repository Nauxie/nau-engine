use super::{EvalAccumulator, EvalSample};

pub(super) fn observe(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.min_target_distance_m = accumulator
        .min_target_distance_m
        .min(sample.target_distance_m);
    accumulator.max_objective_total_count = accumulator
        .max_objective_total_count
        .max(sample.objective.total_count);
    accumulator.max_completed_objective_count = accumulator
        .max_completed_objective_count
        .max(sample.objective.completed_count);
    accumulator.min_objective_distance_m = accumulator
        .min_objective_distance_m
        .min(sample.objective.current_distance_m);
    if sample.objective.complete {
        accumulator.objective_complete_samples += 1;
    }

    accumulator.max_power_up_count = accumulator.max_power_up_count.max(sample.power_up_count);
    accumulator.min_visible_power_up_count = accumulator
        .min_visible_power_up_count
        .min(sample.visible_power_up_count);
    accumulator.max_collected_power_up_count = accumulator
        .max_collected_power_up_count
        .max(sample.collected_power_up_count);
    accumulator.total_power_up_activations = accumulator
        .total_power_up_activations
        .max(sample.total_power_up_activations);
    if sample.active_power_up_effects > 0 {
        accumulator.power_up_effect_samples += 1;
    }
    if sample.on_landing_target {
        accumulator.target_landing_samples += 1;
    }
    if sample.active_lift_fields > 0 {
        accumulator.lifted_samples += 1;
        if sample.readable_lift_fields > 0 {
            accumulator.readable_lift_samples += 1;
        } else {
            accumulator.unreadable_lift_samples += 1;
        }
    }
}
