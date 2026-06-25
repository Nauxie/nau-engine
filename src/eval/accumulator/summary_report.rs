#[path = "summary_report/checks.rs"]
mod checks;
#[path = "summary_report/derived.rs"]
mod derived;
#[path = "summary_report/metrics_summary.rs"]
mod metrics_summary;

use super::EvalAccumulator;
use crate::eval::{
    scenarios::EvalScenario,
    summary::{EvalArtifacts, EvalSummary},
};

use checks::build_checks;
use derived::SummaryDerivedMetrics;
use metrics_summary::build_metrics_summary;

impl EvalAccumulator {
    pub fn summary(&self, scenario: EvalScenario, artifacts: EvalArtifacts) -> EvalSummary {
        let derived = SummaryDerivedMetrics::from_accumulator(self, scenario);
        let checks = build_checks(self, scenario, &derived);
        let passed = checks.iter().all(|check| check.passed);

        EvalSummary {
            scenario_name: scenario.name,
            target_island_name: scenario.target_island_name,
            passed,
            frame_count: scenario.frame_count,
            duration_secs: scenario.duration_secs(),
            thresholds: scenario.thresholds,
            metrics: build_metrics_summary(self, &derived),
            checks,
            artifacts,
            final_sample: self.final_sample.clone(),
        }
    }
}
