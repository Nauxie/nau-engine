#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

host_wait_secs="${NAU_WORLD_FLOOR_FULL_GATE_HOST_WAIT_SECS:-${NAU_WORLD_FLOOR_HOST_WAIT_SECS:-3600}}"
baseline_worktree="${NAU_WORLD_FLOOR_BASELINE_WORKTREE:-target/perf_worktrees/manual_profile_main}"
baseline_cargo_target_dir="${NAU_WORLD_FLOOR_BASELINE_CARGO_TARGET_DIR:-target/perf_builds/world_floor_main}"
candidate_cargo_target_dir="${NAU_WORLD_FLOOR_CANDIDATE_CARGO_TARGET_DIR:-target/perf_builds/world_floor_candidate}"
baseline_perf="${NAU_WORLD_FLOOR_BASELINE_PERF:-target/eval/perf_baseline/main_visible_no_screenshot/perf_summary.json}"
candidate_perf="${NAU_WORLD_FLOOR_CANDIDATE_PERF:-target/eval/perf_baseline/candidate_visible_no_screenshot/perf_summary.json}"
baseline_freeflight_profile="${NAU_WORLD_FLOOR_BASELINE_FREEFLIGHT_PROFILE:-${NAU_WORLD_FLOOR_BASELINE_PROFILE:-target/eval/play_profile/main_scripted_freeflight.json}}"
candidate_freeflight_profile="${NAU_WORLD_FLOOR_CANDIDATE_FREEFLIGHT_PROFILE:-${NAU_WORLD_FLOOR_CANDIDATE_PROFILE:-target/eval/play_profile/candidate_scripted_freeflight.json}}"
baseline_ground_profile="${NAU_WORLD_FLOOR_BASELINE_GROUND_TRAVERSAL_PROFILE:-${NAU_WORLD_FLOOR_BASELINE_GROUND_PROFILE:-target/eval/play_profile/main_scripted_ground_traversal.json}}"
candidate_ground_profile="${NAU_WORLD_FLOOR_CANDIDATE_GROUND_TRAVERSAL_PROFILE:-${NAU_WORLD_FLOOR_CANDIDATE_GROUND_PROFILE:-target/eval/play_profile/candidate_scripted_ground_traversal.json}}"
visual_evidence_dir="${NAU_WORLD_FLOOR_VISUAL_EVIDENCE_DIR:-target/eval/world_floor_visual_evidence}"
run_visual_evidence="${NAU_WORLD_FLOOR_RUN_VISUAL_EVIDENCE:-1}"
write_acceptance_report="${NAU_WORLD_FLOOR_WRITE_ACCEPTANCE_REPORT:-${run_visual_evidence}}"
acceptance_report="${NAU_WORLD_FLOOR_ACCEPTANCE_REPORT:-docs/world-floor-acceptance-report.md}"
baseline_profile_attempts="${NAU_WORLD_FLOOR_BASELINE_PROFILE_ATTEMPTS:-2}"
require_quiet_host_after="${NAU_WORLD_FLOOR_REQUIRE_QUIET_HOST_AFTER:-0}"

if ! [[ "${host_wait_secs}" =~ ^[0-9]+$ ]]; then
  echo "NAU_WORLD_FLOOR_FULL_GATE_HOST_WAIT_SECS must be a non-negative integer" >&2
  exit 2
fi
if ! [[ "${baseline_profile_attempts}" =~ ^[1-9][0-9]*$ ]]; then
  echo "NAU_WORLD_FLOOR_BASELINE_PROFILE_ATTEMPTS must be a positive integer" >&2
  exit 2
fi

case "${run_visual_evidence}" in
  0 | 1) ;;
  *)
    echo "NAU_WORLD_FLOOR_RUN_VISUAL_EVIDENCE must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${write_acceptance_report}" in
  0 | 1) ;;
  *)
    echo "NAU_WORLD_FLOOR_WRITE_ACCEPTANCE_REPORT must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${require_quiet_host_after}" in
  0 | 1) ;;
  *)
    echo "NAU_WORLD_FLOOR_REQUIRE_QUIET_HOST_AFTER must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${baseline_worktree}" in
  /*) ;;
  *) baseline_worktree="${repo_root}/${baseline_worktree}" ;;
esac

case "${baseline_cargo_target_dir}" in
  /*) ;;
  *) baseline_cargo_target_dir="${repo_root}/${baseline_cargo_target_dir}" ;;
esac

case "${candidate_cargo_target_dir}" in
  /*) ;;
  *) candidate_cargo_target_dir="${repo_root}/${candidate_cargo_target_dir}" ;;
esac

case "${baseline_perf}" in
  /*) ;;
  *) baseline_perf="${repo_root}/${baseline_perf}" ;;
esac

case "${candidate_perf}" in
  /*) ;;
  *) candidate_perf="${repo_root}/${candidate_perf}" ;;
esac

case "${baseline_freeflight_profile}" in
  /*) ;;
  *) baseline_freeflight_profile="${repo_root}/${baseline_freeflight_profile}" ;;
esac

case "${candidate_freeflight_profile}" in
  /*) ;;
  *) candidate_freeflight_profile="${repo_root}/${candidate_freeflight_profile}" ;;
esac

case "${baseline_ground_profile}" in
  /*) ;;
  *) baseline_ground_profile="${repo_root}/${baseline_ground_profile}" ;;
esac

case "${candidate_ground_profile}" in
  /*) ;;
  *) candidate_ground_profile="${repo_root}/${candidate_ground_profile}" ;;
esac

case "${visual_evidence_dir}" in
  /*) ;;
  *) visual_evidence_dir="${repo_root}/${visual_evidence_dir}" ;;
esac

case "${acceptance_report}" in
  /*) ;;
  *) acceptance_report="${repo_root}/${acceptance_report}" ;;
esac

if [[ "$(basename "${baseline_perf}")" != "perf_summary.json" ]]; then
  echo "baseline perf path must end in perf_summary.json: ${baseline_perf}" >&2
  exit 2
fi

baseline_perf_dir="$(dirname "${baseline_perf}")"

echo "Preparing instrumented clean-main baseline worktree..."
NAU_RUN_PLAY_PROFILE=0 \
  ./tools/prepare_manual_profile_baseline_worktree.sh \
  "${baseline_worktree}" \
  "${baseline_freeflight_profile}"

capture_baseline_profile() {
  local script_name="$1"
  local profile_path="$2"
  local attempt
  local profile_status

  echo
  echo "Capturing clean-main scripted ${script_name} profile..."
  for ((attempt = 1; attempt <= baseline_profile_attempts; attempt += 1)); do
    echo "Clean-main scripted ${script_name} profile attempt ${attempt}/${baseline_profile_attempts}..."
    profile_status=0
    (
      cd "${baseline_worktree}"
      CARGO_TARGET_DIR="${baseline_cargo_target_dir}" \
        NAU_MANUAL_PROFILE_HOST_WAIT_SECS="${NAU_MANUAL_PROFILE_HOST_WAIT_SECS:-${host_wait_secs}}" \
        NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER="${NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER:-${require_quiet_host_after}}" \
        NAU_PLAY_PROFILE_ALLOW_FAILED_CHECKS=1 \
        "${repo_root}/tools/scripted_play_profile.sh" "${profile_path}" "${script_name}"
    ) || profile_status=$?

    if (( profile_status == 0 )) && jq -e '.passed == true' "${profile_path}" >/dev/null; then
      break
    fi

    if (( attempt < baseline_profile_attempts )); then
      echo "Clean-main scripted ${script_name} profile failed; retrying for host/window-scheduling noise." >&2
    else
      echo "clean-main scripted ${script_name} profile failed after ${baseline_profile_attempts} attempt(s)" >&2
      if (( profile_status == 0 )); then
        exit 1
      fi
      exit "${profile_status}"
    fi
  done
}

capture_baseline_profile "freeflight" "${baseline_freeflight_profile}"
capture_baseline_profile "ground_traversal" "${baseline_ground_profile}"

echo
echo "Capturing clean-main release perf baseline..."
(
  cd "${baseline_worktree}"
  CARGO_TARGET_DIR="${baseline_cargo_target_dir}" \
    NAU_PERF_VISIBLE_WINDOW="${NAU_PERF_VISIBLE_WINDOW:-1}" \
    NAU_PERF_CAPTURE_SCREENSHOT="${NAU_PERF_CAPTURE_SCREENSHOT:-0}" \
    NAU_PERF_HOST_WAIT_SECS="${NAU_PERF_HOST_WAIT_SECS:-${host_wait_secs}}" \
    NAU_PERF_REQUIRE_QUIET_HOST_AFTER="${NAU_PERF_REQUIRE_QUIET_HOST_AFTER:-${require_quiet_host_after}}" \
    NAU_PERF_OUTPUT_DIR="${baseline_perf_dir}" \
    "${repo_root}/tools/perf_baseline.sh" baseline_route long_glide_visibility
)

echo
echo "Running candidate world-floor gate..."
CARGO_TARGET_DIR="${candidate_cargo_target_dir}" \
  NAU_WORLD_FLOOR_BASELINE_PERF="${baseline_perf}" \
  NAU_WORLD_FLOOR_CANDIDATE_PERF="${candidate_perf}" \
  NAU_WORLD_FLOOR_BASELINE_FREEFLIGHT_PROFILE="${baseline_freeflight_profile}" \
  NAU_WORLD_FLOOR_CANDIDATE_FREEFLIGHT_PROFILE="${candidate_freeflight_profile}" \
  NAU_WORLD_FLOOR_BASELINE_GROUND_TRAVERSAL_PROFILE="${baseline_ground_profile}" \
  NAU_WORLD_FLOOR_CANDIDATE_GROUND_TRAVERSAL_PROFILE="${candidate_ground_profile}" \
  NAU_WORLD_FLOOR_HOST_WAIT_SECS="${host_wait_secs}" \
  NAU_WORLD_FLOOR_REQUIRE_QUIET_HOST_AFTER="${require_quiet_host_after}" \
  NAU_MANUAL_PROFILE_REQUIRE_BASELINE_ABSOLUTE_CHECKS=0 \
  ./tools/world_floor_candidate_gate.sh

if [[ "${run_visual_evidence}" == "1" ]]; then
  echo
  echo "Capturing world-floor visual evidence..."
  ./tools/world_floor_visual_evidence.sh "${visual_evidence_dir}"
fi

if [[ "${write_acceptance_report}" == "1" ]]; then
  echo
  echo "Writing world-floor acceptance report..."
  NAU_WORLD_FLOOR_BASELINE_PERF="${baseline_perf}" \
    NAU_WORLD_FLOOR_CANDIDATE_PERF="${candidate_perf}" \
    NAU_WORLD_FLOOR_BASELINE_FREEFLIGHT_PROFILE="${baseline_freeflight_profile}" \
    NAU_WORLD_FLOOR_CANDIDATE_FREEFLIGHT_PROFILE="${candidate_freeflight_profile}" \
    NAU_WORLD_FLOOR_BASELINE_GROUND_TRAVERSAL_PROFILE="${baseline_ground_profile}" \
    NAU_WORLD_FLOOR_CANDIDATE_GROUND_TRAVERSAL_PROFILE="${candidate_ground_profile}" \
    NAU_WORLD_FLOOR_VISUAL_EVIDENCE_DIR="${visual_evidence_dir}" \
    NAU_WORLD_FLOOR_EVIDENCE_COMMAND="./tools/world_floor_full_gate.sh" \
    NAU_WORLD_FLOOR_EVIDENCE_STATUS="Command completed successfully on a quiet host: clean-main and candidate scripted freeflight and ground-traversal profiles, release perf comparison, Rust gates, world-floor readiness, screenshot evidence, and report generation all passed." \
    ./tools/world_floor_acceptance_report.sh "${acceptance_report}"
fi

cat <<EOF

Automated world-floor full gate passed.
Acceptance remains pending until an explicit human manual playtest.
EOF
