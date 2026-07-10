#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

baseline_perf="${NAU_WORLD_FLOOR_BASELINE_PERF:-target/eval/perf_baseline/main_visible_no_screenshot/perf_summary.json}"
candidate_perf="${NAU_WORLD_FLOOR_CANDIDATE_PERF:-target/eval/perf_baseline/candidate_visible_no_screenshot/perf_summary.json}"
baseline_freeflight_profile="${NAU_WORLD_FLOOR_BASELINE_FREEFLIGHT_PROFILE:-${NAU_WORLD_FLOOR_BASELINE_PROFILE:-target/eval/play_profile/main_scripted_freeflight.json}}"
candidate_freeflight_profile="${NAU_WORLD_FLOOR_CANDIDATE_FREEFLIGHT_PROFILE:-${NAU_WORLD_FLOOR_CANDIDATE_PROFILE:-target/eval/play_profile/candidate_scripted_freeflight.json}}"
baseline_ground_profile="${NAU_WORLD_FLOOR_BASELINE_GROUND_TRAVERSAL_PROFILE:-${NAU_WORLD_FLOOR_BASELINE_GROUND_PROFILE:-target/eval/play_profile/main_scripted_ground_traversal.json}}"
candidate_ground_profile="${NAU_WORLD_FLOOR_CANDIDATE_GROUND_TRAVERSAL_PROFILE:-${NAU_WORLD_FLOOR_CANDIDATE_GROUND_PROFILE:-target/eval/play_profile/candidate_scripted_ground_traversal.json}}"
run_rust_gates="${NAU_WORLD_FLOOR_RUN_RUST_GATES:-1}"
run_candidate_perf="${NAU_WORLD_FLOOR_RUN_CANDIDATE_PERF:-1}"
run_candidate_profile="${NAU_WORLD_FLOOR_RUN_CANDIDATE_PROFILE:-1}"
candidate_profile_attempts="${NAU_WORLD_FLOOR_CANDIDATE_PROFILE_ATTEMPTS:-2}"
world_floor_host_wait_secs="${NAU_WORLD_FLOOR_HOST_WAIT_SECS:-900}"
require_quiet_host_after="${NAU_WORLD_FLOOR_REQUIRE_QUIET_HOST_AFTER:-0}"

case "${run_rust_gates}" in
  0 | 1) ;;
  *)
    echo "NAU_WORLD_FLOOR_RUN_RUST_GATES must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${run_candidate_perf}" in
  0 | 1) ;;
  *)
    echo "NAU_WORLD_FLOOR_RUN_CANDIDATE_PERF must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${run_candidate_profile}" in
  0 | 1) ;;
  *)
    echo "NAU_WORLD_FLOOR_RUN_CANDIDATE_PROFILE must be 0 or 1" >&2
    exit 2
    ;;
esac

if ! [[ "${world_floor_host_wait_secs}" =~ ^[0-9]+$ ]]; then
  echo "NAU_WORLD_FLOOR_HOST_WAIT_SECS must be a non-negative integer" >&2
  exit 2
fi
case "${require_quiet_host_after}" in
  0 | 1) ;;
  *)
    echo "NAU_WORLD_FLOOR_REQUIRE_QUIET_HOST_AFTER must be 0 or 1" >&2
    exit 2
    ;;
esac
if ! [[ "${candidate_profile_attempts}" =~ ^[1-9][0-9]*$ ]]; then
  echo "NAU_WORLD_FLOOR_CANDIDATE_PROFILE_ATTEMPTS must be a positive integer" >&2
  exit 2
fi

for baseline_artifact in "${baseline_perf}" "${baseline_freeflight_profile}" "${baseline_ground_profile}"; do
  if [[ ! -s "${baseline_artifact}" ]]; then
    cat >&2 <<EOF
missing clean-main baseline artifact: ${baseline_artifact}

This candidate gate intentionally does not regenerate main baselines from a dirty
world-floor feature branch. Refresh the clean-main artifacts before using this
as gating evidence.
EOF
    exit 1
  fi
done

candidate_perf_dir="$(dirname "${candidate_perf}")"
if [[ "$(basename "${candidate_perf}")" != "perf_summary.json" ]]; then
  echo "candidate perf path must end in perf_summary.json: ${candidate_perf}" >&2
  exit 2
fi

if [[ "${run_rust_gates}" == "1" ]]; then
  cargo fmt --check
  cargo check --all-targets
  cargo test --quiet
  cargo clippy --all-targets --all-features -- -D warnings
fi

if [[ "${run_candidate_perf}" == "1" ]]; then
  NAU_PERF_VISIBLE_WINDOW="${NAU_PERF_VISIBLE_WINDOW:-1}" \
    NAU_PERF_CAPTURE_SCREENSHOT="${NAU_PERF_CAPTURE_SCREENSHOT:-0}" \
    NAU_PERF_HOST_WAIT_SECS="${NAU_PERF_HOST_WAIT_SECS:-${world_floor_host_wait_secs}}" \
    NAU_PERF_REQUIRE_QUIET_HOST_AFTER="${NAU_PERF_REQUIRE_QUIET_HOST_AFTER:-${require_quiet_host_after}}" \
    NAU_PERF_OUTPUT_DIR="${candidate_perf_dir}" \
    ./tools/perf_baseline.sh baseline_route long_glide_visibility
fi

capture_candidate_profile() {
  local script_name="$1"
  local profile_path="$2"
  local attempt

  for ((attempt = 1; attempt <= candidate_profile_attempts; attempt += 1)); do
    echo "Candidate scripted ${script_name} profile attempt ${attempt}/${candidate_profile_attempts}..."
    NAU_MANUAL_PROFILE_HOST_WAIT_SECS="${NAU_MANUAL_PROFILE_HOST_WAIT_SECS:-${world_floor_host_wait_secs}}" \
      NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER="${NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER:-${require_quiet_host_after}}" \
      NAU_PLAY_PROFILE_ALLOW_FAILED_CHECKS=1 \
      ./tools/scripted_play_profile.sh "${profile_path}" "${script_name}"

    if jq -e '.passed == true' "${profile_path}" >/dev/null; then
      break
    fi

    if (( attempt < candidate_profile_attempts )); then
      echo "Candidate scripted ${script_name} profile failed; retrying for host/frame-scheduling noise." >&2
    fi
  done
}

if [[ "${run_candidate_profile}" == "1" ]]; then
  capture_candidate_profile "freeflight" "${candidate_freeflight_profile}"
  capture_candidate_profile "ground_traversal" "${candidate_ground_profile}"
fi

./tools/validate_world_floor_evidence_identity.sh \
  "${baseline_perf}" \
  "${candidate_perf}" \
  "${baseline_freeflight_profile}" \
  "${candidate_freeflight_profile}" \
  "${baseline_ground_profile}" \
  "${candidate_ground_profile}"

NAU_PERF_REQUIRE_QUIET_HOST_AFTER="${NAU_PERF_REQUIRE_QUIET_HOST_AFTER:-${require_quiet_host_after}}" \
  NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER="${NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER:-${require_quiet_host_after}}" \
  ./tools/world_floor_readiness.sh \
  "${baseline_perf}" \
  "${candidate_perf}" \
  "${baseline_freeflight_profile}" \
  "${candidate_freeflight_profile}" \
  "${baseline_ground_profile}" \
  "${candidate_ground_profile}"

cat <<EOF

Automated world-floor candidate gate passed.
Acceptance remains pending until an explicit human manual playtest.

Human manual playtest command, only after the automated gate passes:
  ./tools/manual_play_profile.sh target/eval/play_profile/candidate_manual_playtest.json
EOF
