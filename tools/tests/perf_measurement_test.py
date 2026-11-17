"""Exercise the actual baseline instrumenter, validator and CI retry shell."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[2]


def run(*args, cwd=None, env=None):
    return subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True, check=False)


def executable(path, source):
    path.write_text(source)
    path.chmod(0o755)


class PerfMeasurementTests(unittest.TestCase):
    def test_real_collector_records_instrumented_baseline_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            worktree = root / "worktree"
            source = worktree / "src/eval_app_runtime/metrics.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub(super) const EVAL_FRAME_TIME_WARMUP_FRAMES: u32 = 30;\npub(crate) fn collect_eval_frame_time(time: Res<Time>, mut run: ResMut<EvalRun>) {\n}\n")
            run("git", "init", "-q", cwd=worktree)
            run("git", "add", ".", cwd=worktree)
            self.assertEqual(run("git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture", cwd=worktree).returncode, 0)
            binaries = root / "bin"
            binaries.mkdir()
            executable(binaries / "cargo", RAW_CAPTURE_FIXTURE)
            output = root / "output"
            env = dict(os.environ, PATH=f"{binaries}:{os.environ['PATH']}",
                       NAU_PERF_OUTPUT_DIR=str(output), NAU_PERF_PREPARE_BASELINE_CLOCK="1",
                       NAU_PERF_ALLOW_BUSY_HOST="1", NAU_PERF_REQUIRE_QUIET_HOST_AFTER="0",
                       NAU_PERF_MAX_AVG_FRAME_TIME_MS="500", NAU_PERF_MAX_P95_FRAME_TIME_MS="500",
                       NAU_PERF_MAX_P99_FRAME_TIME_MS="500")
            scenarios = ("baseline_route", "long_glide_visibility", "camera_mouse_control")
            result = run("bash", str(ROOT / "tools/perf_baseline.sh"), *scenarios, cwd=worktree, env=env)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            summary = json.loads((output / "perf_summary.json").read_text())
            self.assertEqual(summary["source_state"], "dirty")
            self.assertEqual(summary["frame_time_clock"], "real")
            self.assertEqual(summary["frame_time_scope"], "runtime_after_startup_warmup")
            self.assertEqual(summary["startup_warmup_frames"], 30)
            self.assertTrue(all(row["p99_frame_time_ms"] == 900 and row["p99_runtime_frame_time_ms"] == 100 for row in summary["scenarios"]))
            self.assertEqual(summary["frame_time_scope"], "runtime_after_startup_warmup")
            self.assertEqual(summary["startup_warmup_frames"], 30)
            self.assertEqual(summary["scenarios"][0]["p99_frame_time_ms"], 900)
            self.assertEqual(summary["scenarios"][0]["p99_runtime_frame_time_ms"], 100)
            self.assertEqual(summary["clock_instrumentation"]["measurement_patch"], "eval_real_clock_v1")
            self.assertNotEqual(summary["clock_instrumentation"]["source_head_sha256"], summary["clock_instrumentation"]["source_measured_sha256"])
            self.assertTrue(all(scenario["eval_passed"] for scenario in summary["scenarios"]))
            valid = run("bash", str(ROOT / "tools/validate_perf_capture.sh"), str(output / "perf_summary.json"), *scenarios)
            self.assertEqual(valid.returncode, 0, valid.stderr)
            # Exercise both standalone entrypoints, without CI's outer validator.
            for failure in ("valid", "missing_max", "wrong_count"):
                with self.subTest(standalone=failure):
                    capture_env = dict(env, FIXTURE_RAW_FAILURE=failure)
                    produced = run("bash", str(ROOT / "tools/perf_baseline.sh"), *scenarios, cwd=worktree, env=capture_env)
                    self.assertEqual(produced.returncode == 0, failure == "valid", produced.stdout + produced.stderr)
                    aggregate = json.loads((output / "perf_summary.json").read_text())
                    self.assertTrue(all(row["passed"] == (failure == "valid") for row in aggregate["scenarios"]))
                    # Explicitly preserve standalone comparison's missing-host opt-in.
                    aggregate.pop("host_snapshot_before")
                    aggregate.pop("host_snapshot_after")
                    # Corruption must fail even if an external writer marks it passed.
                    for row in aggregate["scenarios"]:
                        row["passed"] = True
                    comparison_input = output / "comparison_summary.json"
                    comparison_input.write_text(json.dumps(aggregate))
                    compared = run("bash", str(ROOT / "tools/compare_perf_summaries.sh"),
                                   str(comparison_input), str(comparison_input),
                                   env=dict(env, NAU_PERF_ALLOW_MISSING_HOST_SNAPSHOTS="1"))
                    self.assertEqual(compared.returncode == 0, failure == "valid", compared.stdout + compared.stderr)

    def test_clock_patch_is_exact_idempotent_and_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "src/eval_app_runtime/metrics.rs"
            source.parent.mkdir(parents=True)
            legacy = "pub(crate) fn collect_eval_frame_time(time: Res<Time>, mut run: ResMut<EvalRun>) {"
            original = f"pub(super) const EVAL_FRAME_TIME_WARMUP_FRAMES: u32 = 30;\nconst GAMEPLAY: f32 = 42.0;\n{legacy}\n    run.record(time.delta_secs());\n}}\n"
            source.write_text(original)
            self.assertEqual(run("git", "init", "-q", cwd=root).returncode, 0)
            run("git", "add", ".", cwd=root)
            self.assertEqual(run("git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture", cwd=root).returncode, 0)
            command = ("python3", str(ROOT / "tools/perf_clock.py"), "--prepare-baseline")
            first = run(*command, cwd=root)
            self.assertEqual(first.returncode, 0, first.stderr)
            metadata = json.loads(first.stdout)
            self.assertEqual(metadata["measurement_patch"], "eval_real_clock_v1")
            self.assertEqual(metadata["frame_time_clock"], "real")
            self.assertNotEqual(metadata["source_head_sha256"], metadata["source_measured_sha256"])
            self.assertEqual(source.read_text(), original.replace("Res<Time>", "Res<Time<bevy::time::Real>>"))
            self.assertEqual(run(*command, cwd=root).stdout, first.stdout)
            (root / "unrelated").write_text("do not overwrite")
            self.assertNotEqual(run(*command, cwd=root).returncode, 0)
            (root / "unrelated").unlink()
            source.write_text(source.read_text() + "// unrelated edit\n")
            self.assertNotEqual(run(*command, cwd=root).returncode, 0)
            source.write_text(original.replace("Res<Time>", "Res<Time<bevy::time::Real>>"))
            run("git", "add", ".", cwd=root)
            run("git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "real clock", cwd=root)
            already_real = run(*command, cwd=root)
            self.assertEqual(already_real.returncode, 0, already_real.stderr)
            self.assertIsNone(json.loads(already_real.stdout)["measurement_patch"])
            self.assertEqual(run("git", "status", "--porcelain", cwd=root).stdout, "")
            source.write_text(original.replace("Res<Time>", "Res<Time<Unknown>>"))
            self.assertNotEqual(run(*command, cwd=root).returncode, 0)
            source.write_text(original.replace("= 30;", "= 31;"))
            self.assertNotEqual(run(*command, cwd=root).returncode, 0)

    def test_actual_ci_retry_requires_complete_capture_and_passing_retry(self):
        workflow = (ROOT / ".github/workflows/camera-continuity.yml").read_text()
        script = workflow.split("      - name: Compare PR performance with base", 1)[1]
        script = script.split("        run: |\n", 1)[1].split("      - name: Upload performance comparison evidence", 1)[0]
        script = textwrap.dedent(script).replace("${{ github.event.pull_request.base.sha }}", "fixture-base")
        for failure, should_pass, should_retry in [
            ("budget_once", True, True),
            ("startup_only", True, False),
            ("unknown_phase", False, False),
            ("missing_runtime", False, False),
            ("missing_runtime_max", False, False),
            ("missing_phase", False, False),
            ("wrong_runtime_count", False, False),
            ("budget_always", False, True),
            ("gameplay", False, False),
            ("missing_metric", False, False),
            ("missing_clock", False, False),
            ("missing_scenario", False, False),
            ("missing_snapshot", False, False),
        ]:
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                tools = root / "tools"
                tools.mkdir()
                for name in ("validate_perf_capture.sh", "compare_perf_summaries.sh", "perf_timing.jq"):
                    shutil.copy2(ROOT / "tools" / name, tools / name)
                executable(tools / "perf_baseline.sh", COLLECTOR_FIXTURE)
                binaries = root / "bin"
                binaries.mkdir()
                executable(binaries / "git", '#!/usr/bin/env bash\nif [[ "$1 $2" == "worktree add" ]]; then mkdir -p "$4"; fi\n')
                env = dict(os.environ, GITHUB_WORKSPACE=str(root), RUNNER_TEMP=str(root / "runner"),
                           PATH=f"{binaries}:{os.environ['PATH']}", FIXTURE_FAILURE=failure,
                           NAU_PERF_MAX_AVG_FRAME_TIME_MS="500", NAU_PERF_MAX_P95_FRAME_TIME_MS="500",
                           NAU_PERF_MAX_P99_FRAME_TIME_MS="500", NAU_PERF_REQUIRE_QUIET_HOST_AFTER="0")
                result = run("bash", "-euo", "pipefail", "-c", script, cwd=root, env=env)
                self.assertEqual(result.returncode == 0, should_pass, result.stdout + result.stderr)
                self.assertIn(f"retry attempted: {int(should_retry)}", result.stdout)
                if failure == "budget_always":
                    self.assertEqual(result.stdout.count("frame_time_phase\t"), 2)
                    self.assertIn("absolute_p99_runtime_frame_time_ms\tvalue=600", result.stdout)
                self.assertIn(f"candidate collection status: {0 if failure == 'startup_only' else 1}", result.stdout)
                if should_pass:
                    self.assertIn("retry performance comparison status: 0", result.stdout)
                    if failure == "startup_only":
                        self.assertIn("p99_frame_time_ms_advisory", result.stdout)
                        self.assertIn("candidate=1200", result.stdout)
                        self.assertIn("gating=false", result.stdout)
                    base = json.loads((root / "target/eval/perf_compare/base/perf_summary.json").read_text())
                    self.assertTrue(base["baseline_patch_requested"])


# Replace only the expensive app capture. The production workflow, comparison,
# capture validator and their thresholds run unchanged around these fixtures.
COLLECTOR_FIXTURE = '''#!/usr/bin/env python3
import json, os, pathlib, sys
root = pathlib.Path(os.environ["NAU_PERF_OUTPUT_DIR"])
root.mkdir(parents=True, exist_ok=True)
phase = root.name
failure = os.environ["FIXTURE_FAILURE"]
failed_candidate = phase == "candidate" or (phase == "retry_candidate" and failure == "budget_always")
snapshot = root / "host.txt"
snapshot.write_text("top_cpu_processes\\n123 0.0 0.0 fixture\\n")
scenarios = []
for name in sys.argv[1:]:
    scenario = dict(scenario=name, eval_status=0, eval_passed=True, passed=True, frame_count=229,
        avg_frame_time_ms=100, p95_frame_time_ms=400, p99_frame_time_ms=900, max_frame_time_ms=1500,
        avg_runtime_frame_time_ms=10, p95_runtime_frame_time_ms=12, p99_runtime_frame_time_ms=100, max_runtime_frame_time_ms=120,
        runtime_frame_time_sample_count=200, eval_artifact_frame_time_sample_count=0, runtime_frames_over_33_34ms=0,
        runtime_frames_over_50ms=0, runtime_frames_over_100ms=0, max_entity_count=5000,
        max_mesh_count=2000, max_material_count=116, max_loaded_mesh_vertices=1000000,
        max_loaded_mesh_triangles=1000000, max_visible_island_terrain_count=48,
        max_visible_island_detail_count=202, max_resident_island_visual_count=368,
        max_stream_spawned_visuals_per_frame=32, max_stream_despawned_visuals_per_frame=32)
    if failed_candidate:
        scenario.update(passed=False, p99_runtime_frame_time_ms=600)
        if failure == "startup_only": scenario.update(passed=True, p99_runtime_frame_time_ms=100, p99_frame_time_ms=1200)
        if failure == "missing_runtime": scenario.pop("avg_runtime_frame_time_ms")
        if failure == "missing_runtime_max": scenario.pop("max_runtime_frame_time_ms")
        if failure == "wrong_runtime_count": scenario["runtime_frame_time_sample_count"] = 199
        if failure == "gameplay": scenario.update(eval_status=1, eval_passed=False)
        if failure == "missing_metric": scenario.pop("avg_frame_time_ms")
    scenarios.append(scenario)
summary = dict(mode="release", frame_time_clock="real", frame_time_scope="runtime_after_startup_warmup", startup_warmup_frames=30, source_fingerprint="fixture",
    visible_window=True, capture_screenshot=False, scenarios=scenarios,
    host_snapshot_before=str(snapshot), host_snapshot_after=str(snapshot),
    baseline_patch_requested=os.environ.get("NAU_PERF_PREPARE_BASELINE_CLOCK") == "1")
if failed_candidate:
    if failure == "unknown_phase": summary["startup_warmup_frames"] = 31
    if failure == "missing_phase": summary.pop("frame_time_scope")
    if failure == "missing_clock": summary.pop("frame_time_clock")
    if failure == "missing_scenario": summary["scenarios"].pop()
    if failure == "missing_snapshot": snapshot.unlink()
(root / "perf_summary.json").write_text(json.dumps(summary))
sys.exit(1 if failed_candidate and failure != "startup_only" else 0)
'''

RAW_CAPTURE_FIXTURE = '''#!/usr/bin/env python3
import json, os, pathlib, sys
if sys.argv[1] == "build": sys.exit(0)
args = sys.argv[2:]
name = args[args.index("--eval") + 1]
root = pathlib.Path(args[args.index("--eval-output") + 1])
root.mkdir(parents=True, exist_ok=True)
metrics = dict(avg_frame_time_ms=100, p95_frame_time_ms=400, p99_frame_time_ms=900, max_frame_time_ms=1500,
        avg_runtime_frame_time_ms=10, p95_runtime_frame_time_ms=12, p99_runtime_frame_time_ms=100, max_runtime_frame_time_ms=120,
    runtime_frame_time_sample_count=200, eval_artifact_frame_time_sample_count=0, runtime_frames_over_33_34ms=0, runtime_frames_over_50ms=0,
    runtime_frames_over_100ms=0, max_entity_count=5000, max_mesh_count=2000, max_material_count=116,
    max_loaded_mesh_vertices=1000000, max_loaded_mesh_triangles=1000000,
    max_visible_island_terrain_count=48, max_visible_island_detail_count=202,
    max_resident_island_visual_count=368, max_stream_spawned_visuals_per_frame=32,
    max_stream_despawned_visuals_per_frame=32)
failure = os.environ.get("FIXTURE_RAW_FAILURE")
if failure == "missing_max": metrics.pop("max_runtime_frame_time_ms")
if failure == "wrong_count": metrics["runtime_frame_time_sample_count"] = 199
(root / "summary.json").write_text(json.dumps(dict(scenario=name, passed=True,
    frame_count=229, duration_secs=3.4, metrics=metrics, checks=[])))
'''


if __name__ == "__main__":
    unittest.main()
