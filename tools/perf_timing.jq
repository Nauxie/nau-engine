# Shared by capture production, standalone comparison, and CI capture validity.
def perf_phase_valid:
  .mode == "release"
  and .frame_time_clock == "real"
  and .frame_time_scope == "runtime_after_startup_warmup"
  and .startup_warmup_frames == 30;

def perf_scenario_timing_valid:
  (.frame_count | type == "number" and . > 0)
  and all(.avg_frame_time_ms, .p95_frame_time_ms, .p99_frame_time_ms,
          .max_frame_time_ms, .runtime_frame_time_sample_count,
          .eval_artifact_frame_time_sample_count,
          .runtime_frames_over_33_34ms, .runtime_frames_over_50ms,
          .runtime_frames_over_100ms;
          type == "number" and . >= 0)
  and all(.avg_runtime_frame_time_ms, .p95_runtime_frame_time_ms,
          .p99_runtime_frame_time_ms, .max_runtime_frame_time_ms;
          type == "number" and . > 0)
  and .runtime_frame_time_sample_count > 0
  and (.runtime_frame_time_sample_count + .eval_artifact_frame_time_sample_count)
    == (.frame_count + 1 - 30);

def perf_timing_valid:
  perf_phase_valid
  and (.scenarios | type == "array" and length > 0)
  and all(.scenarios[]; perf_scenario_timing_valid);
