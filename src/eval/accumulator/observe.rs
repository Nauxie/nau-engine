use super::{
    EvalAccumulator, EvalSample, assets, camera, content, initial_sample, movement, objectives,
    world,
};

impl EvalAccumulator {
    pub fn observe_frame_time_ms(&mut self, frame_time_ms: f32) {
        if frame_time_ms.is_finite() && frame_time_ms >= 0.0 {
            self.frame_times_ms.push(frame_time_ms);
        }
    }

    pub fn observe(&mut self, sample: EvalSample) {
        if self.first_sample.is_none() {
            initial_sample::observe(self, &sample);
        }

        self.sample_count += 1;
        movement::observe(self, &sample);
        camera::observe(self, &sample);
        world::observe(self, &sample);
        content::observe(self, &sample);
        objectives::observe(self, &sample);
        assets::observe(self, &sample);

        self.final_sample = Some(sample);
    }
}
