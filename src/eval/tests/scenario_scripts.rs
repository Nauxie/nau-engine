use super::*;
use crate::eval::scenarios::{
    ISLAND_HERO_GALLERY_FRAMES_PER_VIEW, ISLAND_HERO_GALLERY_HOLD_FRAMES,
    ISLAND_HERO_GALLERY_SETTLE_FRAMES,
};
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn shell_array<'a>(script: &'a str, name: &str) -> Vec<&'a str> {
    let marker = format!("{name}=(");
    let body = script
        .split_once(&marker)
        .unwrap_or_else(|| panic!("{name} shell array exists"))
        .1;

    body.lines()
        .take_while(|line| line.trim() != ")")
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_matches('"'))
        .collect()
}

#[test]
fn eval_sim_suite_covers_every_simulation_scenario() {
    let script = include_str!("../../../tools/eval_sim_suite.sh");
    let actual = shell_array(script, "scenarios");
    let expected = SCENARIO_NAMES
        .iter()
        .copied()
        .filter(|name| !APP_ONLY_SCENARIO_NAMES.contains(name))
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn camera_continuity_gate_covers_the_required_surface_and_timing_contract() {
    let script = include_str!("../../../tools/camera_continuity_gate.sh");
    let actual = shell_array(script, "simulation_scenarios")
        .into_iter()
        .chain(shell_array(script, "app_scenarios"))
        .collect::<BTreeSet<_>>();
    let expected = [
        CAMERA_MOUSE_CONTROL,
        CAMERA_YAW_STABILITY,
        CAMERA_TURN_STABILITY,
        CAMERA_STRAFE_STABILITY,
        AIR_CONTROL_RESPONSE,
        GREAT_SKY_PLATEAU_ROUTE,
        PLATEAU_ARRIVAL_CAMERA,
        UNDERBRIDGE_UNDER_ROUTE,
        PLAYTEST_RESET,
        WORLD_COLLISION_CONTACT,
        TERRAIN_RIM_COLLISION_CONTACT,
        TERRAIN_BODY_COLLISION_CONTACT,
        TERRAIN_EDGE_WALKOFF,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(actual, expected);
    assert!(script.contains("requested_refresh_rates=(30 60 120 144)"));
    assert!(script.contains("requested_hitches_ms=(50 100)"));
    assert!(script.contains("inactive_look_resets_capture_history_before_resume"));
    assert!(script.contains(".metrics.sample_count == $expected_samples"));
    assert!(script.contains(".metrics.max_world_collision_push_m <= $max_push"));
    assert!(script.contains(".metrics.max_terrain_rim_collision_push_m <= $max_push"));
    assert!(script.contains(".metrics.max_terrain_body_collision_push_m <= $max_push"));
    assert!(
        script.contains("NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ANGULAR_VELOCITY_DEGREES_PER_SEC:-180")
    );
    assert!(script.contains("NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ACCELERATION_MPS2:-1300"));
    assert!(script.contains(
        "NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ANGULAR_ACCELERATION_DEGREES_PER_SEC2:-6000"
    ));
    assert!(
        script.contains(".metrics.max_camera_player_relative_angular_velocity_degrees_per_sec")
    );
    assert!(
        script
            .contains(".metrics.max_camera_player_relative_angular_acceleration_degrees_per_sec2")
    );
    assert!(script.contains("fault_injection_proof_json=\"$(jq -cn"));
    assert!(script.contains("ran: ($ran == 1)"));
    assert!(script.contains("tests: (if $ran == 1 then ["));
    assert!(script.contains("] else [] end"));
    assert!(script.contains("--argjson fault_injection_proof"));
}

#[test]
fn continuity_and_performance_gates_fail_closed_on_incomplete_or_failed_evidence() {
    let continuity = include_str!("../../../tools/camera_continuity_gate.sh");
    assert!(continuity.contains("if (( eval_status != 0 )); then"));
    assert!(continuity.contains("exit \"${eval_status}\""));
    assert!(continuity.contains("exit 1"));

    let performance = include_str!("../../../tools/dev_play_performance_gate.sh");
    assert!(performance.contains("$debug_eval_status == 0"));
    assert!(performance.contains("and $release_eval_status == 0"));
    assert!(performance.contains("and $debug[0].passed == true"));
    assert!(performance.contains("and $release[0].passed == true"));
    assert!(performance.contains("if ! jq -e '.passed == true'"));
}

#[test]
fn development_performance_gate_keeps_local_and_ci_budgets_explicit() {
    let performance = include_str!("../../../tools/dev_play_performance_gate.sh");
    assert!(performance.contains("NAU_DEV_PLAY_PERF_VISIBLE_WINDOW:-0"));
    assert!(performance.contains("NAU_DEV_PLAY_PERF_MAX_AVG_FRAME_TIME_MS:-12"));
    assert!(performance.contains("NAU_DEV_PLAY_PERF_MAX_P95_FRAME_TIME_MS:-18"));
    assert!(performance.contains("NAU_DEV_PLAY_PERF_MAX_FRAME_TIME_MS:-35"));
    assert!(performance.contains("NAU_DEV_PLAY_PERF_MAX_FRAMES_OVER_16_67MS:-24"));
    assert!(performance.contains("NAU_DEV_PLAY_PERF_MAX_MATERIAL_COUNT:-128"));
    assert!(performance.contains("NAU_DEV_PLAY_PERF_MAX_DEBUG_RELEASE_AVG_RATIO:-1.25"));
    assert!(performance.contains("NAU_DEV_PLAY_PERF_RUN_WARMUP:-1"));
    assert!(performance.contains("NAU_DEV_PLAY_PERF_RUN_HOST_PREFLIGHT:-1"));

    let workflow = include_str!("../../../.github/workflows/camera-continuity.yml");
    assert!(workflow.contains("NAU_DEV_PLAY_PERF_MAX_AVG_FRAME_TIME_MS: \"70\""));
    assert!(workflow.contains("NAU_DEV_PLAY_PERF_MAX_P95_FRAME_TIME_MS: \"90\""));
    assert!(workflow.contains("NAU_DEV_PLAY_PERF_MAX_MATERIAL_COUNT: \"116\""));
    assert!(workflow.contains("NAU_DEV_PLAY_PERF_RUN_HOST_PREFLIGHT: \"0\""));
    assert!(workflow.contains(
        "NAU_PERF_MAX_AVG_FRAME_TIME_MS: \"500\"\n          \
         NAU_PERF_MAX_P95_FRAME_TIME_MS: \"500\"\n          \
         NAU_PERF_MAX_P99_FRAME_TIME_MS: \"500\""
    ));
    assert!(
        workflow.contains(
            "NAU_PERF_SUMMARY_CAMERA_MOUSE_MAX_AVG_FRAME_TIME_REGRESSION_RATIO: \"1.15\""
        )
    );
    assert!(workflow.contains("NAU_PERF_SUMMARY_MAX_GATING_HITCH_FRACTION: \"0.05\""));
    assert!(!workflow.contains("NAU_PERF_SUMMARY_CAMERA_MOUSE_MAX_HITCH_COUNT_REGRESSION_RATIO"));
    assert!(!workflow.contains("NAU_PERF_SUMMARY_MAX_COUNT_REGRESSION_RATIO: \"2.0\""));
    assert!(workflow.contains("Compare PR performance with base"));
    assert!(workflow.contains("github.event.pull_request.base.sha"));
    assert!(workflow.contains("./tools/perf_baseline.sh"));
    assert!(workflow.contains("camera_mouse_control"));
    assert!(workflow.contains("./tools/compare_perf_summaries.sh"));
    assert!(workflow.contains(") || baseline_status=$?"));
    assert!(workflow.contains("camera_mouse_control || candidate_status=$?"));
    assert!(workflow.contains("\"${candidate_output}/perf_summary.json\" || comparison_status=$?"));
    assert!(workflow.contains(
        "if (( baseline_status == 0 && candidate_status == 0 && comparison_status != 0 )); then"
    ));
    assert!(workflow.contains("retry_attempted=1"));
    assert!(workflow.contains(
        "if (( retry_candidate_status != 0 || retry_baseline_status != 0 || retry_comparison_status != 0 )); then"
    ));
    assert!(workflow.contains(
        "if (( baseline_status != 0 || candidate_status != 0 || final_comparison_status != 0 )); then"
    ));
    let retry_candidate = workflow
        .find("NAU_PERF_OUTPUT_DIR=\"${retry_candidate_output}\"")
        .expect("candidate-first retry");
    let retry_baseline = workflow
        .find("NAU_PERF_OUTPUT_DIR=\"${retry_baseline_output}\"")
        .expect("base-second retry");
    assert!(retry_candidate < retry_baseline);
    assert!(workflow.contains("branches:\n      - main"));

    let comparison = include_str!("../../../tools/compare_perf_summaries.sh");
    assert!(comparison.contains(
        "NAU_PERF_SUMMARY_CAMERA_MOUSE_MAX_AVG_FRAME_TIME_REGRESSION_RATIO:-${max_frame_time_ratio}"
    ));
    assert!(comparison.contains("NAU_PERF_SUMMARY_MAX_GATING_HITCH_FRACTION:-0.05"));
    assert!(
        comparison
            .contains("NAU_PERF_SUMMARY_MAX_GATING_HITCH_FRACTION must be numeric between 0 and 1")
    );
    assert!(comparison.contains("if [[ \"${scenario}\" == \"camera_mouse_control\" ]]; then"));
    assert!(comparison.contains(
        "scenario_runtime_frame_time_sample_count \"${baseline_summary}\" \"${scenario}\""
    ));
    assert!(comparison.contains("printf 'null\\n'"));
    assert!(comparison.contains("fraction <= max_fraction ? \"true\" : \"false\""));
    assert!(comparison.contains(
        "compare_metric \"${scenario}\" \"avg_frame_time_ms\" \\\n    \"${scenario_avg_frame_time_ratio}\""
    ));
    assert!(comparison.contains(
        "compare_metric \"${scenario}\" \"p95_frame_time_ms\" \\\n    \"${max_frame_time_ratio}\""
    ));
    assert!(comparison.contains(
        "compare_optional_hitch_metric \"${scenario}\" \"runtime_frames_over_50ms\" \\\n    \"${max_count_ratio}\""
    ));
    assert!(comparison.contains(
        "compare_metric \"${scenario}\" \"max_entity_count\" \\\n    \"${max_count_ratio}\""
    ));
    let baseline = include_str!("../../../tools/perf_baseline.sh");
    assert!(baseline.contains(
        "runtime_frame_time_sample_count: (.metrics.runtime_frame_time_sample_count // null)"
    ));
}

fn performance_comparison_scenario(
    name: &str,
    runtime_samples: u64,
    avg_frame_time_ms: f64,
    p95_frame_time_ms: f64,
    runtime_hitch_counts: [u64; 3],
    max_visible_island_detail_count: u64,
) -> serde_json::Value {
    serde_json::json!({
        "scenario": name,
        "eval_status": 0,
        "passed": true,
        "frame_count": runtime_samples + 4,
        "avg_frame_time_ms": avg_frame_time_ms,
        "p95_frame_time_ms": p95_frame_time_ms,
        "p99_frame_time_ms": 90.0,
        "max_frame_time_ms": 120.0,
        "runtime_frame_time_sample_count": runtime_samples,
        "runtime_frames_over_33_34ms": runtime_hitch_counts[0],
        "runtime_frames_over_50ms": runtime_hitch_counts[1],
        "runtime_frames_over_100ms": runtime_hitch_counts[2],
        "max_entity_count": 5_005,
        "max_mesh_count": 2_105,
        "max_material_count": 116,
        "max_loaded_mesh_vertices": 1_182_842,
        "max_loaded_mesh_triangles": 1_251_872,
        "max_visible_island_terrain_count": 48,
        "max_visible_island_detail_count": max_visible_island_detail_count,
        "max_resident_island_visual_count": 368,
        "max_stream_spawned_visuals_per_frame": 32,
        "max_stream_despawned_visuals_per_frame": 32
    })
}

fn write_performance_comparison_summary(
    root: &Path,
    mut scenarios: Vec<serde_json::Value>,
    aggregate_runtime_sample_count: bool,
) {
    fs::create_dir_all(root).expect("create performance comparison fixture directory");
    for scenario in &mut scenarios {
        let name = scenario["scenario"]
            .as_str()
            .expect("fixture scenario name")
            .to_string();
        let runtime_samples = scenario["runtime_frame_time_sample_count"]
            .as_u64()
            .expect("fixture runtime sample count");
        let raw_summary_dir = root.join(&name);
        fs::create_dir_all(&raw_summary_dir).expect("create raw scenario fixture directory");
        fs::write(
            raw_summary_dir.join("summary.json"),
            serde_json::json!({
                "metrics": {
                    "runtime_frame_time_sample_count": runtime_samples
                }
            })
            .to_string(),
        )
        .expect("write raw scenario fixture");
        if !aggregate_runtime_sample_count {
            scenario
                .as_object_mut()
                .expect("fixture scenario object")
                .remove("runtime_frame_time_sample_count");
        }
    }

    fs::write(
        root.join("perf_summary.json"),
        serde_json::json!({
            "mode": "release",
            "visible_window": true,
            "capture_screenshot": false,
            "scenarios": scenarios
        })
        .to_string(),
    )
    .expect("write performance comparison fixture");
}

fn run_performance_comparison(
    baseline: &Path,
    candidate: &Path,
    gating_hitch_fraction: Option<&str>,
) -> Output {
    let mut command = Command::new("bash");
    command
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tools/compare_perf_summaries.sh"))
        .arg(baseline.join("perf_summary.json"))
        .arg(candidate.join("perf_summary.json"))
        .env("NAU_PERF_ALLOW_MISSING_HOST_SNAPSHOTS", "1")
        .env("NAU_PERF_REQUIRE_QUIET_HOST_AFTER", "0")
        .env("NAU_PERF_MAX_AVG_FRAME_TIME_MS", "500")
        .env("NAU_PERF_MAX_P95_FRAME_TIME_MS", "500")
        .env("NAU_PERF_MAX_P99_FRAME_TIME_MS", "500");
    if let Some(fraction) = gating_hitch_fraction {
        command.env("NAU_PERF_SUMMARY_MAX_GATING_HITCH_FRACTION", fraction);
    }
    command
        .output()
        .expect("run performance summary comparison")
}

#[test]
fn performance_comparison_distinguishes_bulk_cadence_from_tail_and_structure() {
    let fixture_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let fixture_root = std::env::temp_dir().join(format!(
        "nau-perf-comparison-{}-{fixture_id}",
        std::process::id()
    ));
    let baseline_root = fixture_root.join("baseline");
    let candidate_root = fixture_root.join("candidate");
    let baseline_scenarios = vec![
        performance_comparison_scenario(
            "baseline_route",
            436,
            49.2948,
            67.4392,
            [390, 186, 5],
            202,
        ),
        performance_comparison_scenario(
            "long_glide_visibility",
            716,
            57.3681,
            84.2747,
            [695, 470, 13],
            251,
        ),
    ];
    let candidate_scenarios = vec![
        performance_comparison_scenario(
            "baseline_route",
            436,
            53.9088,
            73.0728,
            [424, 276, 1],
            202,
        ),
        performance_comparison_scenario(
            "long_glide_visibility",
            716,
            45.5954,
            61.0184,
            [636, 237, 0],
            251,
        ),
    ];
    write_performance_comparison_summary(&baseline_root, baseline_scenarios, false);
    write_performance_comparison_summary(&candidate_root, candidate_scenarios.clone(), true);

    let noisy_bucket = run_performance_comparison(&baseline_root, &candidate_root, None);
    let noisy_bucket_stdout = String::from_utf8_lossy(&noisy_bucket.stdout);
    assert!(
        noisy_bucket.status.success(),
        "bulk cadence must not fail the comparator:\n{noisy_bucket_stdout}\n{}",
        String::from_utf8_lossy(&noisy_bucket.stderr)
    );
    assert!(noisy_bucket_stdout.contains("runtime_frames_over_50ms_advisory"));
    let noisy_tail_line = noisy_bucket_stdout
        .lines()
        .find(|line| line.starts_with("baseline_route\truntime_frames_over_100ms\t"))
        .expect("baseline route severe-hitch result");
    assert!(noisy_tail_line.contains("baseline=5/436"));
    assert!(noisy_tail_line.contains("candidate=1/436"));
    assert!(noisy_tail_line.ends_with("gating=true\tpassed=true"));

    let invalid_fraction =
        run_performance_comparison(&baseline_root, &candidate_root, Some("-0.1"));
    assert_eq!(invalid_fraction.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&invalid_fraction.stderr)
            .contains("must be numeric between 0 and 1")
    );

    let mut severe_hitch_scenarios = candidate_scenarios.clone();
    severe_hitch_scenarios[0]["runtime_frames_over_100ms"] = serde_json::json!(11);
    write_performance_comparison_summary(&candidate_root, severe_hitch_scenarios, true);
    let severe_hitch = run_performance_comparison(&baseline_root, &candidate_root, None);
    assert!(!severe_hitch.status.success());
    let severe_hitch_stdout = String::from_utf8_lossy(&severe_hitch.stdout);
    let severe_hitch_line = severe_hitch_stdout
        .lines()
        .find(|line| line.starts_with("baseline_route\truntime_frames_over_100ms\t"))
        .expect("severe-hitch regression result");
    assert!(severe_hitch_line.contains("baseline=5/436"));
    assert!(severe_hitch_line.contains("candidate=11/436"));
    assert!(severe_hitch_line.ends_with("gating=true\tpassed=false"));

    let mut structural_regression_scenarios = candidate_scenarios;
    structural_regression_scenarios[0]["max_visible_island_detail_count"] = serde_json::json!(223);
    write_performance_comparison_summary(&candidate_root, structural_regression_scenarios, true);
    let structural_regression = run_performance_comparison(&baseline_root, &candidate_root, None);
    assert!(!structural_regression.status.success());
    let structural_regression_stdout = String::from_utf8_lossy(&structural_regression.stdout);
    let structural_regression_line = structural_regression_stdout
        .lines()
        .find(|line| line.starts_with("baseline_route\tmax_visible_island_detail_count\t"))
        .expect("structural regression result");
    assert!(structural_regression_line.contains("baseline=202"));
    assert!(structural_regression_line.contains("candidate=223"));
    assert!(structural_regression_line.ends_with("passed=false"));

    fs::remove_dir_all(fixture_root).expect("remove performance comparison fixture");
}

#[test]
fn development_performance_gate_enforces_camera_feel_budgets_for_both_profiles() {
    let performance = include_str!("../../../tools/dev_play_performance_gate.sh");

    assert!(performance.contains("max_frame_time_ms \\\n  max_debug_release_avg_ratio"));
    assert!(
        performance
            .contains("if ! [[ \"${max_frames_over_16_67ms}\" =~ ^(0|[1-9][0-9]*)$ ]]; then")
    );
    assert!(performance.contains("if ! [[ \"${max_material_count}\" =~ ^[1-9][0-9]*$ ]]; then"));
    assert!(performance.contains("and (.metrics.max_frame_time_ms | type) == \"number\""));
    assert!(performance.contains("and (.metrics.frames_over_16_67ms | type) == \"number\""));
    assert!(performance.contains("and (.metrics.max_material_count | type) == \"number\""));
    assert!(performance.contains("local warmup_dir=\"${output_root}/${profile}_warmup\""));
    assert!(performance.contains("./tools/perf_host_preflight.sh"));
    assert!(performance.contains("warmup_run: ($run_warmup == 1)"));
    assert!(performance.contains("host_preflight: ($run_host_preflight == 1)"));

    for (profile_field, metric, threshold) in [
        (
            "avg_frame_time_ms",
            "avg_frame_time_ms",
            "max_avg_frame_time_ms",
        ),
        (
            "p95_frame_time_ms",
            "p95_frame_time_ms",
            "max_p95_frame_time_ms",
        ),
        (
            "max_frame_time_ms",
            "max_frame_time_ms",
            "max_frame_time_ms",
        ),
        (
            "frames_over_16_67ms",
            "frames_over_16_67ms",
            "max_frames_over_16_67ms",
        ),
        (
            "max_material_count",
            "max_material_count",
            "max_material_count",
        ),
    ] {
        assert!(performance.contains(&format!("{threshold}: ${threshold}")));
        for profile in ["debug", "release"] {
            assert!(performance.contains(&format!(
                "and ${profile}[0].metrics.{metric} <= ${threshold}"
            )));
            assert!(
                performance.contains(&format!("{profile_field}: ${profile}[0].metrics.{metric}"))
            );
        }
    }

    assert!(performance.contains("and $avg_ratio <= $max_debug_release_avg_ratio"));
    assert!(performance.contains(
        "and $debug[0].metrics.max_entity_count == $release[0].metrics.max_entity_count"
    ));
    assert!(
        performance
            .contains("and $debug[0].metrics.max_mesh_count == $release[0].metrics.max_mesh_count")
    );
    assert!(performance.contains(
        "and $debug[0].metrics.max_loaded_mesh_triangles\n          == $release[0].metrics.max_loaded_mesh_triangles"
    ));
}

#[test]
fn scenarios_require_exact_expected_sample_coverage() {
    for name in SCENARIO_NAMES {
        let scenario = scenario_named(name).expect("scenario exists");
        let observed_schedule_count = (0..=scenario.frame_count)
            .filter(|frame| scenario.should_sample(*frame))
            .count() as u32;

        assert_eq!(observed_schedule_count, scenario.expected_sample_count());
        assert_eq!(
            scenario.thresholds.min_samples, observed_schedule_count,
            "{name} should fail if any deterministic sample is omitted"
        );
    }
}

#[test]
fn baseline_route_has_scripted_launch_and_glide() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 2).launch);
    assert!(scripted_input(scenario, 60).glide);
    assert!(scripted_input(scenario, 390).glide);
    assert!(scripted_input(scenario, 390).dive);
}

#[test]
fn release_traversal_content_budgets_cover_authored_island_density() {
    for name in [BASELINE_ROUTE, LONG_GLIDE_VISIBILITY] {
        let scenario = scenario_named(name).expect("release traversal route exists");

        assert_eq!(scenario.thresholds.max_visible_island_detail_count, 260);
        assert_eq!(scenario.thresholds.max_resident_island_visual_count, 430);
        assert_eq!(scenario.thresholds.max_entity_count, 5_500);
    }
}

#[test]
fn ground_taxi_script_exercises_wasd_without_launching() {
    let scenario = scenario_named(GROUND_TAXI_CONTROL).expect("ground taxi route exists");

    assert!(scripted_input(scenario, 20).forward);
    assert!(scripted_input(scenario, 60).right);
    assert!(scripted_input(scenario, 135).backward);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 60).glide);
    assert_eq!(scenario.sample_stride, 1);
    assert!(scenario.frame_count >= MIN_SUSTAINED_WIND_VISUAL_FLOW_SAMPLES);
    assert!(scenario.frame_count >= MIN_SUSTAINED_UPDRAFT_VISUAL_FLOW_SAMPLES);
    assert!(scenario.frame_count >= MIN_SUSTAINED_CROSSWIND_VISUAL_FLOW_SAMPLES);
}

#[test]
fn playtest_reset_script_triggers_the_central_reset_command() {
    let scenario = scenario_named(PLAYTEST_RESET).expect("playtest reset route exists");
    let alias = scenario_named("central_reset").expect("central reset alias exists");

    assert_eq!(alias.name, PLAYTEST_RESET);
    assert!(APP_ONLY_SCENARIO_NAMES.contains(&PLAYTEST_RESET));
    assert_eq!(scenario.target_island_name, Some("great sky plateau"));
    assert!(scripted_input(scenario, 12).forward);
    assert!(scripted_input(scenario, 20).right);
    assert!(!scripted_input(scenario, 36).forward);
    assert!(!scripted_input(scenario, 36).launch);
    assert!(!scripted_playtest_reset_requested(scenario, 29));
    assert!(scripted_playtest_reset_requested(scenario, 30));
    assert!(!scripted_playtest_reset_requested(scenario, 31));
    assert!(
        scenario
            .checkpoint_at(96)
            .is_some_and(|checkpoint| { checkpoint.name == "plateau_central_close_review" })
    );
    assert_eq!(scenario.frame_count, 180);
    assert_eq!(scenario.thresholds.min_samples, 181);
    assert!(scenario.thresholds.min_horizontal_distance_m >= 2_000.0);
    assert!(scenario.thresholds.min_max_speed_mps >= 8.0);
    assert!(scenario.thresholds.min_grounded_samples >= 170);
    assert!(!scenario.thresholds.require_target_landing);
    assert!(scenario.thresholds.min_target_landing_samples >= 140);
    assert!(scenario.thresholds.max_final_target_distance_m <= 0.05);
}

#[test]
fn world_collision_contact_script_taxis_into_launch_tree() {
    let scenario = scenario_named(WORLD_COLLISION_CONTACT).expect("collision route exists");

    assert!(scripted_input(scenario, 60).backward);
    assert!(scripted_input(scenario, 150).backward);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 60).glide);
    assert_eq!(scenario.thresholds.max_abs_camera_view_yaw_degrees, 32.0);
    assert!(scenario.thresholds.max_camera_rotation_delta_degrees <= 1.5);
    assert_eq!(scenario.thresholds.min_camera_obstructed_distance_m, 3.4);
    assert_eq!(scenario.thresholds.max_camera_obstruction_snap_count, 0);
}

#[test]
fn terrain_rim_collision_contact_script_presses_into_visible_launch_rim() {
    let scenario = scenario_named(TERRAIN_RIM_COLLISION_CONTACT).expect("rim route exists");

    assert!(scripted_input(scenario, 15).forward);
    assert!(scripted_input(scenario, 52).forward);
    assert!(scripted_input(scenario, 52).left);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 52).glide);
    assert!(!scripted_input(scenario, 52).backward);
    assert_eq!(scenario.thresholds.min_grounded_samples, 0);
    assert_eq!(scenario.frame_count, 56);
}

#[test]
fn terrain_body_collision_contact_script_presses_into_visible_launch_cliff_body() {
    let scenario = scenario_named(TERRAIN_BODY_COLLISION_CONTACT).expect("body route exists");

    assert!(scripted_input(scenario, 60).left);
    assert!(scripted_input(scenario, 120).left);
    assert!(!scripted_input(scenario, 120).forward);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 60).glide);
    assert!(!scripted_input(scenario, 120).backward);
    assert_eq!(scenario.thresholds.min_grounded_samples, 0);
    assert_eq!(scenario.frame_count, 121);
}

#[test]
fn terrain_edge_walkoff_script_walks_off_then_skim_glides() {
    let scenario = scenario_named(TERRAIN_EDGE_WALKOFF).expect("edge walkoff route exists");

    assert!(scripted_input(scenario, 60).right);
    assert!(scripted_input(scenario, 180).right);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 20).glide);
    assert!(scripted_input(scenario, 30).glide);
    assert!(scripted_input(scenario, 180).glide);
    assert_eq!(scenario.frame_count, 300);
    assert!(scenario.thresholds.min_gliding_samples >= 18);
    assert!(scenario.thresholds.min_grounded_samples >= 12);
}

#[test]
fn updraft_route_steers_toward_lift_without_diving() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 90).right);
    assert!(scripted_input(scenario, 180).glide);
    assert!(!scripted_input(scenario, 180).dive);
    assert_eq!(scenario.thresholds.min_completed_objective_count, 1);
}

#[test]
fn island_launch_script_releases_forward_after_touchdown() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");

    assert!(scripted_input(scenario, 360).forward);
    assert!(scripted_input(scenario, 360).glide);
    assert!(scripted_input(scenario, 360).dive);
    assert!(scripted_input(scenario, 423).forward);
    assert!(!scripted_input(scenario, 475).forward);
    assert!(scenario.thresholds.require_target_landing);
}

#[test]
fn branch_recovery_route_targets_named_recovery_island() {
    let scenario = scenario_named(BRANCH_RECOVERY_ROUTE).expect("branch route exists");

    assert_eq!(scenario.target_island_name, Some("sunlit terrace"));
    assert!(scenario.thresholds.require_target_landing);
    assert_eq!(scenario.thresholds.min_objective_total_count, 3);
    assert_eq!(scenario.thresholds.min_completed_objective_count, 3);
    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 540).glide);
    assert!(scripted_input(scenario, 540).dive);
    assert!(!scripted_input(scenario, 590).dive);
    assert!(!scripted_input(scenario, 630).forward);
    assert!(scripted_input(scenario, 630).backward);
    assert!(!scripted_input(scenario, 650).forward);
    assert!(scripted_input(scenario, 650).backward);
    assert!(scripted_input(scenario, 660).backward);
    assert!(scripted_input(scenario, 675).backward);
    assert!(!scripted_input(scenario, 690).backward);
    assert!(!scripted_input(scenario, 750).forward);
}

#[test]
fn camera_mouse_script_exercises_x_and_y_axes() {
    let scenario = scenario_named(CAMERA_MOUSE_CONTROL).expect("camera route exists");

    assert!(scripted_camera_input(scenario, 30).mouse_delta.x > 0.0);
    assert!(scripted_camera_input(scenario, 70).mouse_delta.y < 0.0);
    assert!(scripted_camera_input(scenario, 105).mouse_delta.y > 0.0);
    assert!(scripted_input(scenario, 30).forward);
    assert!(!scripted_input(scenario, 70).forward);
    assert!(scenario.thresholds.min_horizontal_distance_m >= 3.0);
    assert_eq!(
        scenario.thresholds.min_samples,
        scenario.expected_sample_count()
    );
    assert!(scenario.thresholds.max_camera_step_distance_m <= 0.7);
    assert!(scenario.thresholds.max_camera_rotation_delta_degrees <= 1.75);
}

#[test]
fn camera_yaw_stability_script_applies_small_yaw_then_settles() {
    let scenario = scenario_named(CAMERA_YAW_STABILITY).expect("camera yaw route exists");

    assert!(scripted_camera_input(scenario, 18).mouse_delta.x > 0.0);
    assert_eq!(scripted_camera_input(scenario, 80), CameraInput::default());
    assert_eq!(
        scripted_input(scenario, 18),
        FlightInput::default(),
        "yaw stability eval should isolate mouse drift from movement"
    );
}

#[test]
fn camera_turn_script_exercises_air_turns_and_air_brake() {
    let scenario = scenario_named(CAMERA_TURN_STABILITY).expect("turn route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 80).glide);
    assert!(scripted_input(scenario, 85).left);
    assert!(scripted_input(scenario, 115).right);
    assert!(scripted_input(scenario, 180).forward);
    assert!(scripted_input(scenario, 255).backward);
}

#[test]
fn camera_strafe_script_exercises_lateral_input_without_mouse() {
    let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");

    assert!(scripted_input(scenario, 30).right);
    assert!(scripted_input(scenario, 130).left);
    assert_eq!(scripted_camera_input(scenario, 30), CameraInput::default());
    assert_eq!(scripted_camera_input(scenario, 130), CameraInput::default());
}

#[test]
fn air_control_response_script_exercises_lateral_brake_and_recovery_without_mouse() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 90).forward);
    assert!(scripted_input(scenario, 90).right);
    assert!(scripted_input(scenario, 140).right);
    assert!(!scripted_input(scenario, 140).forward);
    assert!(scripted_input(scenario, 210).left);
    assert!(scripted_input(scenario, 250).backward);
    assert!(scripted_input(scenario, 250).right);
    assert!(scripted_input(scenario, 310).backward);
    assert!(scripted_input(scenario, 310).left);
    assert!(scripted_input(scenario, 350).glide);
    assert!(scripted_input(scenario, 350).dive);
    assert!(scripted_input(scenario, 370).forward);
    assert_eq!(scripted_camera_input(scenario, 90), CameraInput::default());
    assert_eq!(scripted_camera_input(scenario, 210), CameraInput::default());
    assert_eq!(scripted_camera_input(scenario, 310), CameraInput::default());
    assert!(scenario.thresholds.min_gliding_samples >= 45);
}

#[test]
fn pose_state_coverage_script_exercises_full_traversal_pose_chain() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 20).forward);
    assert!(!scripted_input(scenario, 30).glide);
    assert!(scripted_input(scenario, 75).glide);
    assert!(scripted_input(scenario, 200).left);
    assert!(!scripted_input(scenario, 200).forward);
    assert!(!scripted_input(scenario, 200).backward);
    assert!(!scripted_input(scenario, 260).dive);
    assert!(scripted_input(scenario, 265).dive);
    assert!(scripted_input(scenario, 290).backward);
    assert!(scripted_input(scenario, 290).left);
    assert!(!scripted_input(scenario, 290).forward);
    assert!(scripted_input(scenario, 315).backward);
    assert!(scripted_input(scenario, 315).right);
    assert!(!scripted_input(scenario, 315).forward);
    assert!(scripted_input(scenario, 333).right);
    assert!(!scripted_input(scenario, 333).forward);
    assert!(!scripted_input(scenario, 333).backward);
    assert!(scripted_input(scenario, 345).right);
    assert!(!scripted_input(scenario, 345).backward);
    assert!(!scripted_input(scenario, 360).dive);
    assert!(scripted_input(scenario, 450).forward);
    assert!(!scripted_input(scenario, 500).forward);
    assert!(!scripted_input(scenario, 510).forward);
    assert!(!scripted_input(scenario, 540).forward);
    assert!(!scripted_input(scenario, 650).forward);
    assert!(!scripted_input(scenario, 700).forward);
    assert!(scripted_input(scenario, 720).forward);
    assert!(!scripted_input(scenario, 760).forward);
    assert_eq!(scripted_camera_input(scenario, 360), CameraInput::default());
    assert!(scenario.frame_count >= 840);
    assert!(scenario.thresholds.min_samples >= 140);
    assert!(scenario.thresholds.min_grounded_samples >= 12);
    assert!(scenario.thresholds.min_gliding_samples >= 55);
}

#[test]
fn long_glide_visibility_script_crosses_archipelago() {
    let scenario = scenario_named(LONG_GLIDE_VISIBILITY).expect("long glide route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 120).right);
    assert!(scripted_input(scenario, 160).left);
    assert!(scripted_input(scenario, 620).glide);
    assert!(!scripted_input(scenario, 620).dive);
    assert!(scenario.thresholds.min_sky_island_count >= MIN_SKY_ISLAND_COUNT);
    assert_eq!(
        scenario.thresholds.min_power_up_count,
        AERIAL_POWER_UP_ROUTE.len()
    );
    assert_eq!(scenario.thresholds.min_collected_power_up_count, 3);
    assert!(scenario.thresholds.min_power_up_effect_samples >= 3);
}

#[test]
fn great_sky_plateau_route_targets_long_vertical_chain() {
    let scenario = scenario_named(GREAT_SKY_PLATEAU_ROUTE).expect("plateau route exists");
    let alias = scenario_named("plateau_route").expect("plateau alias exists");
    let checkpoint_names = scenario
        .checkpoints
        .iter()
        .map(|checkpoint| checkpoint.name)
        .collect::<Vec<_>>();

    assert_eq!(alias.name, GREAT_SKY_PLATEAU_ROUTE);
    assert_eq!(alias.target_island_name, scenario.target_island_name);
    assert_eq!(scenario.target_island_name, Some("great sky plateau"));
    assert_eq!(
        checkpoint_names,
        [
            "launch_review",
            "upper_thermal_chain",
            "high_crown_tease",
            "waterfall_vista",
            "plateau_arrival_reveal",
        ]
    );
    assert!(
        scenario.frame_count
            > scenario_named(LONG_GLIDE_VISIBILITY)
                .expect("long glide route exists")
                .frame_count
    );
    assert_eq!(scenario.thresholds.min_objective_total_count, 10);
    assert!(scenario.thresholds.min_completed_objective_count >= 3);
    assert!(scenario.thresholds.min_lifted_samples >= 8);
    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 420).glide);
    assert!(scripted_input(scenario, 480).dive);
    assert!(scripted_input(scenario, 1020).right);
    assert!(scripted_input(scenario, 1240).left);
    assert!(scripted_input(scenario, 1980).glide);
    assert!(scripted_input(scenario, 1980).forward);
    assert!(!scripted_input(scenario, 1980).backward);
}

#[test]
fn great_sky_plateau_vistas_is_a_grounded_pixel_review() {
    let scenario =
        scenario_named(GREAT_SKY_PLATEAU_VISTAS).expect("plateau vistas scenario exists");
    let alias = scenario_named("plateau_showcase").expect("plateau vistas alias exists");

    assert_eq!(alias.name, GREAT_SKY_PLATEAU_VISTAS);
    assert!(APP_ONLY_SCENARIO_NAMES.contains(&GREAT_SKY_PLATEAU_VISTAS));
    assert_eq!(scenario.target_island_name, Some("great sky plateau"));
    assert_eq!(
        scenario
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.name)
            .collect::<Vec<_>>(),
        ["plateau_arrival_reveal", "waterfall_vista"]
    );
    assert!(scenario.thresholds.min_grounded_samples >= 280);
    assert!(scenario.thresholds.max_final_target_distance_m <= 220.0);
    assert_eq!(scripted_input(scenario, 120), FlightInput::default());
    assert_eq!(
        scripted_camera_input(scenario, 120).mouse_delta,
        bevy::prelude::Vec2::ZERO
    );
    assert_eq!(
        scripted_camera_input(scenario, 240).mouse_delta,
        bevy::prelude::Vec2::ZERO
    );
}

#[test]
fn island_surface_review_is_registered_as_app_only_with_aliases() {
    let scenario =
        scenario_named(ISLAND_SURFACE_REVIEW).expect("island surface review scenario exists");

    assert_eq!(SCENARIO_NAMES.len(), 24);
    assert_eq!(APP_ONLY_SCENARIO_NAMES.len(), 9);
    assert!(APP_ONLY_SCENARIO_NAMES.contains(&ISLAND_SURFACE_REVIEW));
    assert_eq!(
        scenario_named("surface_review")
            .expect("surface review alias exists")
            .name,
        ISLAND_SURFACE_REVIEW
    );
    assert_eq!(
        scenario_named("island_details")
            .expect("island details alias exists")
            .name,
        ISLAND_SURFACE_REVIEW
    );
    assert_eq!(scenario.target_island_name, Some("great sky plateau"));
}

#[test]
fn island_hero_gallery_is_catalog_driven_app_only_and_transition_lenient() {
    use crate::world::{ISLAND_REVIEW_VIEWS_PER_ISLAND, IslandReviewPlan, SkyRoute};

    let scenario = scenario_named("island_hero_gallery").expect("hero gallery scenario exists");
    let alias = scenario_named("all_islands").expect("hero gallery alias exists");
    let plan = IslandReviewPlan::from_route(&SkyRoute::default());
    let expected_capture_count = plan.islands.len() * ISLAND_REVIEW_VIEWS_PER_ISLAND;

    assert_eq!(alias.name, scenario.name);
    assert!(APP_ONLY_SCENARIO_NAMES.contains(&scenario.name));
    assert_eq!(plan.islands.len(), 41);
    assert_eq!(expected_capture_count, 123);
    assert_eq!(ISLAND_HERO_GALLERY_SETTLE_FRAMES, 32);
    assert_eq!(ISLAND_HERO_GALLERY_HOLD_FRAMES, 4);
    assert_eq!(ISLAND_HERO_GALLERY_FRAMES_PER_VIEW, 36);
    assert_eq!(
        scenario.frame_count,
        expected_capture_count as u32 * ISLAND_HERO_GALLERY_FRAMES_PER_VIEW - 1
    );
    assert_eq!(scenario.thresholds.min_samples, scenario.frame_count + 1);
    assert!(scenario.checkpoints.is_empty());
    assert_eq!(scenario.target_island_name, None);
    assert_eq!(scenario.thresholds.min_horizontal_distance_m, 0.0);
    assert_eq!(scenario.thresholds.max_camera_step_distance_m, 20_000.0);
    assert_eq!(scenario.thresholds.max_camera_rotation_delta_degrees, 180.0);
    assert_eq!(scenario.thresholds.max_camera_player_angle_degrees, 180.0);
    assert_eq!(
        scenario.thresholds.max_camera_orbit_alignment_degrees,
        180.0
    );
    assert_eq!(scenario.thresholds.max_visible_island_detail_count, 260);
    assert_eq!(scenario.thresholds.max_resident_island_visual_count, 430);
    assert_eq!(scenario.thresholds.max_entity_count, 5_500);
}

#[test]
fn island_surface_review_has_grounded_detail_checkpoints_and_zero_input() {
    let scenario =
        scenario_named(ISLAND_SURFACE_REVIEW).expect("island surface review scenario exists");

    assert_eq!(scenario.frame_count, 360);
    assert_eq!(scenario.sample_stride, 1);
    assert_eq!(scenario.thresholds.min_samples, 361);
    assert_eq!(
        scenario
            .checkpoints
            .iter()
            .map(|checkpoint| (checkpoint.frame, checkpoint.name))
            .collect::<Vec<_>>(),
        [
            (60, "ruins_and_rock_detail"),
            (180, "dense_flora_detail"),
            (300, "lake_river_waterfall_detail"),
        ]
    );
    assert!(scenario.thresholds.min_grounded_samples >= 340);
    assert_eq!(scenario.thresholds.min_horizontal_distance_m, 0.0);
    assert_eq!(scenario.thresholds.min_max_speed_mps, 0.0);
    assert_eq!(scenario.thresholds.min_gliding_samples, 0);
    assert_eq!(scenario.thresholds.min_lifted_samples, 0);
    assert_eq!(scenario.thresholds.max_camera_distance_m, 360.0);
    assert_eq!(scenario.thresholds.max_camera_step_distance_m, 6.0);
    assert_eq!(scenario.thresholds.max_camera_rotation_delta_degrees, 3.5);
    assert_eq!(scenario.thresholds.max_camera_player_angle_degrees, 90.0);
    assert!(!scenario.thresholds.require_target_landing);
    assert_eq!(scenario.thresholds.min_target_landing_samples, 0);
    assert_eq!(scenario.thresholds.max_final_target_distance_m, 8.0);

    for frame in [0, 60, 120, 180, 240, 300, 360] {
        assert_eq!(scripted_input(scenario, frame), FlightInput::default());
        assert_eq!(
            scripted_camera_input(scenario, frame),
            CameraInput::default()
        );
    }
}

#[test]
fn underbridge_under_route_targets_low_cave_camera_pass() {
    let scenario = scenario_named(UNDERBRIDGE_UNDER_ROUTE).expect("underbridge route exists");
    let alias = scenario_named("under_route").expect("under-route alias exists");

    assert_eq!(alias.name, UNDERBRIDGE_UNDER_ROUTE);
    assert_eq!(scenario.target_island_name, Some("underbridge cay"));
    assert_eq!(
        scenario.checkpoints[1].name,
        "under_route_camera_obstruction"
    );
    assert_eq!(
        scenario.thresholds.min_camera_obstruction_adjustment_m,
        0.25
    );
    assert!(scenario.thresholds.max_camera_step_distance_m <= 1.0);
    assert_eq!(scenario.thresholds.max_camera_obstruction_snap_count, 0);
    assert!(scenario.thresholds.min_lifted_samples >= 2);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 30).left);
    assert!(scripted_input(scenario, 120).glide);
    assert!(!scripted_input(scenario, 120).dive);
    assert!(scripted_input(scenario, 220).glide);
    assert!(scripted_input(scenario, 220).dive);
    assert!(scripted_input(scenario, 220).right);
    assert!(scripted_input(scenario, 220).backward);
    assert!(!scripted_input(scenario, 220).left);
    assert!(!scripted_input(scenario, 220).forward);
    assert!(scripted_input(scenario, 240).right);
    assert!(scripted_input(scenario, 240).backward);
    assert!(!scripted_input(scenario, 240).dive);
    assert!(scripted_input(scenario, 300).right);
    assert!(scripted_input(scenario, 300).backward);
    assert!(!scripted_input(scenario, 330).left);
    assert!(!scripted_input(scenario, 330).forward);
    assert!(!scripted_input(scenario, 390).right);
}

#[test]
fn scenario_camera_thresholds_guard_follow_distance_and_jitter() {
    for name in SCENARIO_NAMES {
        let scenario = scenario_named(name).expect("scenario exists");
        let mouse_camera = *name == CAMERA_MOUSE_CONTROL;
        let gallery = *name == "island_hero_gallery";
        let cinematic_vista = matches!(*name, GREAT_SKY_PLATEAU_VISTAS | ISLAND_SURFACE_REVIEW);
        let max_camera_distance_m = match *name {
            "island_hero_gallery" => 1_000.0,
            ISLAND_SURFACE_REVIEW => 360.0,
            GREAT_SKY_PLATEAU_VISTAS => 220.0,
            _ => 16.5,
        };

        assert!(
            scenario.thresholds.max_camera_distance_m <= max_camera_distance_m,
            "{name} should fail if its camera exceeds the intended framing distance"
        );
        assert!(
            scenario.thresholds.max_camera_step_distance_m
                <= if gallery {
                    20_000.0
                } else if cinematic_vista {
                    6.0
                } else {
                    1.15
                },
            "{name} should fail large per-frame camera jumps"
        );
        assert!(
            scenario.thresholds.max_camera_player_angle_degrees
                <= if mouse_camera {
                    6.0
                } else if gallery {
                    180.0
                } else if cinematic_vista {
                    90.0
                } else {
                    3.0
                },
            "{name} should keep the player focus centered"
        );
        assert!(
            scenario.thresholds.max_camera_rotation_delta_degrees
                <= if mouse_camera {
                    1.75
                } else if gallery {
                    180.0
                } else if cinematic_vista {
                    3.5
                } else {
                    1.5
                },
            "{name} should fail camera rotation jitter"
        );
        assert!(
            scenario.thresholds.max_camera_orbit_alignment_degrees
                <= if gallery { 180.0 } else { 5.0 },
            "{name} should fail broad orbit misalignment"
        );
    }
}

#[test]
fn scenarios_define_non_final_camera_checkpoints() {
    for name in SCENARIO_NAMES {
        let scenario = scenario_named(name).expect("scenario exists");

        if *name == "island_hero_gallery" {
            assert!(scenario.checkpoints.is_empty());
            continue;
        }
        assert!(!scenario.checkpoints.is_empty());
        assert!(
            scenario
                .checkpoints
                .iter()
                .all(|checkpoint| checkpoint.frame < scenario.frame_count)
        );
        assert!(
            scenario
                .checkpoints
                .windows(2)
                .all(|pair| pair[0].frame < pair[1].frame),
            "{name} checkpoint frames should be strictly increasing"
        );
        assert_eq!(
            scenario.checkpoint_at(scenario.checkpoints[0].frame),
            Some(scenario.checkpoints[0])
        );
    }
}

#[test]
fn camera_continuity_fault_injection_rejects_nominally_unsampled_one_frame_snap() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let fault_frame = 1_u32;

    assert!(!fault_frame.is_multiple_of(scenario.sample_stride));
    assert!(!scenario.should_sample(fault_frame));

    let summary_for = |injected_fault_frame: Option<u32>| {
        let mut accumulator = EvalAccumulator::default();
        for frame in 0..=scenario.frame_count {
            let camera_step_distance_m = if injected_fault_frame == Some(frame) {
                scenario.thresholds.max_camera_step_distance_m + 0.25
            } else {
                0.1
            };
            accumulator.observe_continuity(
                frame,
                12.0,
                camera_step_distance_m,
                camera_step_distance_m,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0,
                "follow",
                false,
                false,
                0.0,
                0.0,
                0.0,
                None,
            );
            if !scenario.should_sample(frame) {
                continue;
            }
            let mut sample = content_metric_sample(scenario, frame, 12, 0, 64);
            sample.movement_camera_heading_error_degrees = 0.0;
            accumulator.observe(sample);
        }
        accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        )
    };

    let passing = summary_for(None);
    assert_eq!(
        passing.metrics.sample_count,
        scenario.expected_sample_count()
    );
    assert!(named_check(&passing, "sample_count").passed);
    assert!(named_check(&passing, "max_camera_step_distance").passed);

    let faulted = summary_for(Some(fault_frame));
    assert_eq!(
        faulted.metrics.max_camera_step_distance_m,
        scenario.thresholds.max_camera_step_distance_m + 0.25
    );
    assert!(named_check(&faulted, "sample_count").passed);
    assert!(!named_check(&faulted, "max_camera_step_distance").passed);
}

#[test]
fn camera_continuity_rotation_fault_injection_rejects_one_frame_snap() {
    const MAX_ANGULAR_VELOCITY_DEGREES_PER_SEC: f32 = 180.0;
    const MAX_ANGULAR_ACCELERATION_DEGREES_PER_SEC2: f32 = 15_000.0;

    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let fault_frame = 1_u32;

    assert!(!scenario.should_sample(fault_frame));

    let summary_for = |injected_fault_frame: Option<u32>| {
        let mut accumulator = EvalAccumulator::default();
        for frame in 0..=scenario.frame_count {
            let (angular_velocity, angular_acceleration) = if injected_fault_frame == Some(frame) {
                (720.0, 36_900.0)
            } else {
                (105.0, 6_300.0)
            };
            accumulator.observe_continuity(
                frame,
                12.0,
                0.1,
                0.1,
                1.0,
                0.0,
                0.0,
                angular_velocity,
                angular_acceleration,
                0.0,
                0,
                "follow",
                false,
                false,
                0.0,
                0.0,
                0.0,
                None,
            );
            if !scenario.should_sample(frame) {
                continue;
            }
            let mut sample = content_metric_sample(scenario, frame, 12, 0, 64);
            sample.movement_camera_heading_error_degrees = 0.0;
            accumulator.observe(sample);
        }
        accumulator.summary(
            scenario,
            EvalArtifacts {
                summary_json: "summary.json".to_string(),
                samples_ndjson: "samples.ndjson".to_string(),
                screenshot_png: None,
                checkpoint_screenshots: Vec::new(),
                checkpoint_marker_metadata: Vec::new(),
            },
        )
    };
    let angular_gate_passes = |summary: &EvalSummary| {
        summary
            .metrics
            .max_camera_player_relative_angular_velocity_degrees_per_sec
            <= MAX_ANGULAR_VELOCITY_DEGREES_PER_SEC
            && summary
                .metrics
                .max_camera_player_relative_angular_acceleration_degrees_per_sec2
                <= MAX_ANGULAR_ACCELERATION_DEGREES_PER_SEC2
    };

    let passing = summary_for(None);
    assert!(angular_gate_passes(&passing));
    assert!(named_check(&passing, "max_camera_step_distance").passed);
    assert!(named_check(&passing, "air_control_camera_rotation_delta").passed);

    let faulted = summary_for(Some(fault_frame));
    assert_eq!(
        faulted
            .metrics
            .max_camera_player_relative_angular_velocity_degrees_per_sec,
        720.0
    );
    assert_eq!(
        faulted
            .metrics
            .max_camera_player_relative_angular_acceleration_degrees_per_sec2,
        36_900.0
    );
    assert!(!angular_gate_passes(&faulted));
    assert!(named_check(&faulted, "max_camera_step_distance").passed);
    assert!(named_check(&faulted, "air_control_camera_rotation_delta").passed);
}
