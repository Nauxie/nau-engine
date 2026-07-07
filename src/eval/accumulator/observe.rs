use super::{
    EvalAccumulator, EvalSample, assets, camera, content, initial_sample, movement, objectives,
    world,
};
use crate::eval::{
    EvalScenario, MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS, min_updraft_swirl_force_delta_mps_for_scenario,
};

impl EvalAccumulator {
    pub fn observe_frame_time_ms(&mut self, frame_time_ms: f32) {
        self.observe_frame_time_sample_ms(frame_time_ms, true);
    }

    pub fn observe_eval_artifact_frame_time_ms(&mut self, frame_time_ms: f32) {
        self.observe_frame_time_sample_ms(frame_time_ms, false);
    }

    pub fn observe_startup_frame_time_ms(&mut self, frame_time_ms: f32) {
        if frame_time_ms.is_finite() && frame_time_ms >= 0.0 {
            self.frame_times_ms.push(frame_time_ms);
        }
    }

    fn observe_frame_time_sample_ms(&mut self, frame_time_ms: f32, runtime_sample: bool) {
        if frame_time_ms.is_finite() && frame_time_ms >= 0.0 {
            self.frame_times_ms.push(frame_time_ms);
            if runtime_sample {
                self.runtime_frame_times_ms.push(frame_time_ms);
            } else {
                self.eval_artifact_frame_times_ms.push(frame_time_ms);
            }
        }
    }

    pub fn observe(&mut self, sample: EvalSample) {
        self.observe_with_updraft_swirl_force_delta_mps(sample, MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS);
    }

    pub fn observe_for_scenario(&mut self, sample: EvalSample, scenario: EvalScenario) {
        self.observe_with_updraft_swirl_force_delta_mps(
            sample,
            min_updraft_swirl_force_delta_mps_for_scenario(scenario.name),
        );
    }

    fn observe_with_updraft_swirl_force_delta_mps(
        &mut self,
        sample: EvalSample,
        min_updraft_swirl_force_delta_mps: f32,
    ) {
        if self.first_sample.is_none() {
            initial_sample::observe(self, &sample);
        }

        self.sample_count += 1;
        movement::observe(self, &sample);
        camera::observe(self, &sample);
        world::observe(self, &sample, min_updraft_swirl_force_delta_mps);
        content::observe(self, &sample);
        objectives::observe(self, &sample);
        assets::observe(self, &sample);

        self.final_sample = Some(sample);
    }
}
