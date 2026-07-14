#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to summarize the play profile" >&2
  exit 1
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${script_dir}/source_identity.sh"

profile_path="${1:-target/eval/manual_play/rollback_baseline.json}"
profile_dir="$(dirname "${profile_path}")"
profile_duration_secs="${NAU_PLAY_PROFILE_DURATION_SECS:-45}"
profile_script="${NAU_PLAY_PROFILE_SCRIPT:-}"
host_snapshot_before="${profile_path}.host_snapshot_before.txt"
host_snapshot_after="${profile_path}.host_snapshot_after.txt"
max_host_process_cpu_percent="${NAU_MANUAL_PROFILE_MAX_HOST_PROCESS_CPU_PERCENT:-80}"
max_host_total_cpu_percent="${NAU_MANUAL_PROFILE_MAX_HOST_TOTAL_CPU_PERCENT:-${NAU_PERF_MAX_HOST_TOTAL_CPU_PERCENT:-160}}"
allow_busy_host="${NAU_MANUAL_PROFILE_ALLOW_BUSY_HOST:-0}"
require_quiet_host_after="${NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER:-1}"
allow_failed_checks="${NAU_PLAY_PROFILE_ALLOW_FAILED_CHECKS:-0}"
default_ignore_process_pattern="${NAU_PERF_DEFAULT_IGNORE_PROCESS_PATTERN-}"
if [[ "${NAU_MANUAL_PROFILE_IGNORE_PROCESS_PATTERN+x}" == "x" ]]; then
  ignore_process_pattern="${NAU_MANUAL_PROFILE_IGNORE_PROCESS_PATTERN}"
elif [[ "${NAU_PERF_IGNORE_PROCESS_PATTERN+x}" == "x" ]]; then
  ignore_process_pattern="${NAU_PERF_IGNORE_PROCESS_PATTERN}"
else
  ignore_process_pattern="${default_ignore_process_pattern}"
fi
build_first="${NAU_PLAY_PROFILE_BUILD_FIRST:-1}"
host_wait_secs="${NAU_MANUAL_PROFILE_HOST_WAIT_SECS:-0}"
host_wait_interval_secs=5
source_commit="$(nau_source_commit)"
source_state="$(nau_source_state)"
source_fingerprint="$(nau_source_fingerprint)"

for value_name in max_host_process_cpu_percent max_host_total_cpu_percent; do
  value="${!value_name}"
  if ! [[ "${value}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "${value_name} must be numeric, got: ${value}" >&2
    exit 2
  fi
done

case "${allow_busy_host}" in
  0 | 1) ;;
  *)
    echo "NAU_MANUAL_PROFILE_ALLOW_BUSY_HOST must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${require_quiet_host_after}" in
  0 | 1) ;;
  *)
    echo "NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${allow_failed_checks}" in
  0 | 1) ;;
  *)
    echo "NAU_PLAY_PROFILE_ALLOW_FAILED_CHECKS must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${build_first}" in
  0 | 1) ;;
  *)
    echo "NAU_PLAY_PROFILE_BUILD_FIRST must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${host_wait_secs}" in
  '' | *[!0-9]*)
    echo "NAU_MANUAL_PROFILE_HOST_WAIT_SECS must be a non-negative integer" >&2
    exit 2
    ;;
esac

write_host_snapshot() {
  local output_path="$1"
  {
    printf 'generated_at\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    if [[ -n "${ignore_process_pattern}" ]]; then
      printf 'ignored_process_pattern\t%s\n' "${ignore_process_pattern}"
    fi
    printf 'thermal_status\n'
    pmset -g therm 2>/dev/null || true
    printf '\ntop_cpu_processes\n'
    ps -Ao pid,pcpu,pmem,comm | sort -k2 -nr | sed -n '1,20p'
  } > "${output_path}"
}

host_snapshot_max_cpu() {
  local snapshot="$1"
  awk -v ignore="${ignore_process_pattern}" '
    BEGIN { in_top = 0; max_cpu = 0 }
    /^top_cpu_processes$/ { in_top = 1; next }
    in_top && $1 ~ /^[0-9]+$/ {
      if (ignore != "" && $0 ~ ignore) {
        next
      }
      cpu = $2 + 0
      if (cpu > max_cpu) {
        max_cpu = cpu
      }
    }
    END { printf "%.1f", max_cpu }
  ' "${snapshot}"
}

host_snapshot_top_process() {
  local snapshot="$1"
  awk -v ignore="${ignore_process_pattern}" '
    /^top_cpu_processes$/ { in_top = 1; next }
    in_top && $1 ~ /^[0-9]+$/ {
      if (ignore != "" && $0 ~ ignore) {
        next
      }
      print $1 " " $2 "% " $4
      exit
    }
  ' "${snapshot}"
}

host_snapshot_total_cpu() {
  local snapshot="$1"
  awk -v ignore="${ignore_process_pattern}" '
    BEGIN { in_top = 0; total_cpu = 0 }
    /^top_cpu_processes$/ { in_top = 1; next }
    in_top && $1 ~ /^[0-9]+$/ {
      if (ignore != "" && $0 ~ ignore) {
        next
      }
      total_cpu += $2 + 0
    }
    END { printf "%.1f", total_cpu }
  ' "${snapshot}"
}

host_is_quiet() {
  local snapshot="$1"
  local max_cpu
  local total_cpu
  max_cpu="$(host_snapshot_max_cpu "${snapshot}")"
  total_cpu="$(host_snapshot_total_cpu "${snapshot}")"
  awk \
    -v max_cpu="${max_cpu}" \
    -v max_threshold="${max_host_process_cpu_percent}" \
    -v total_cpu="${total_cpu}" \
    -v total_threshold="${max_host_total_cpu_percent}" \
    'BEGIN { print (max_cpu <= max_threshold && total_cpu <= total_threshold ? "true" : "false") }'
}

capture_quiet_host_snapshot() {
  local snapshot="$1"
  local phase="${2:-before launch}"
  local waited_secs=0

  while true; do
    write_host_snapshot "${snapshot}"
    if [[ "$(host_is_quiet "${snapshot}")" == "true" ]]; then
      return 0
    fi

    if [[ "${host_wait_secs}" == "0" || "${waited_secs}" -ge "${host_wait_secs}" ]]; then
      return 1
    fi

    echo "host is busy; waiting ${host_wait_interval_secs}s ${phase}..." >&2
    echo "max process CPU: $(host_snapshot_max_cpu "${snapshot}")% (limit ${max_host_process_cpu_percent}%)" >&2
    echo "total top-process CPU: $(host_snapshot_total_cpu "${snapshot}")% (limit ${max_host_total_cpu_percent}%)" >&2
    echo "top process: $(host_snapshot_top_process "${snapshot}")" >&2
    sleep "${host_wait_interval_secs}"
    waited_secs=$((waited_secs + host_wait_interval_secs))
  done
}

mkdir -p "${profile_dir}"
rm -f "${profile_path}"
rm -f "${host_snapshot_before}" "${host_snapshot_after}"

cat <<EOF
Foreground play profile

This runs the release sandbox with profiling enabled.

EOF
if [[ -n "${profile_script}" ]]; then
  echo "Scripted profile: ${profile_script}"
else
  echo "Manual profile: play normally in the foreground until the profile duration completes."
fi
echo "Keep the game window foregrounded; measurement arms after focused movement."
echo

command=(cargo run --release -- --play --play-profile "${profile_path}")
if [[ "${profile_duration_secs}" != "0" ]]; then
  command+=(--play-profile-duration "${profile_duration_secs}")
fi
if [[ -n "${profile_script}" ]]; then
  command+=(--play-profile-script "${profile_script}")
fi

printf 'Command:'
printf ' %q' "${command[@]}"
printf '\n'
printf 'Profile: %s\n' "${profile_path}"
printf 'Host snapshot before: %s\n' "${host_snapshot_before}"
printf 'Host snapshot after: %s\n' "${host_snapshot_after}"
printf 'Duration: %ss\n' "${profile_duration_secs}"
printf 'Build first: %s\n' "${build_first}"
printf 'Host wait: %ss\n' "${host_wait_secs}"
printf 'Require quiet host after: %s\n' "${require_quiet_host_after}"
printf 'Allow failed checks: %s\n' "${allow_failed_checks}"
if [[ -n "${profile_script}" ]]; then
  printf 'Script: %s\n' "${profile_script}"
fi
if [[ -n "${ignore_process_pattern}" ]]; then
  printf 'Ignored process pattern: %s\n' "${ignore_process_pattern}"
fi

if [[ "${build_first}" == "1" ]]; then
  echo
  echo "Prebuilding release binary before host snapshot..."
  cargo build --release
fi

if [[ "${allow_busy_host}" == "1" ]]; then
  write_host_snapshot "${host_snapshot_before}"
elif ! capture_quiet_host_snapshot "${host_snapshot_before}" "before launch"; then
  echo "host is too busy for a gating play profile; refusing to launch" >&2
  echo "max process CPU: $(host_snapshot_max_cpu "${host_snapshot_before}")% (limit ${max_host_process_cpu_percent}%)" >&2
  echo "total top-process CPU: $(host_snapshot_total_cpu "${host_snapshot_before}")% (limit ${max_host_total_cpu_percent}%)" >&2
  echo "top process: $(host_snapshot_top_process "${host_snapshot_before}")" >&2
  if [[ -n "${ignore_process_pattern}" ]]; then
    echo "ignored process pattern: ${ignore_process_pattern}" >&2
  fi
  echo "snapshot: ${host_snapshot_before}" >&2
  echo "Set NAU_MANUAL_PROFILE_ALLOW_BUSY_HOST=1 only for non-gating investigation." >&2
  exit 1
fi
set +e
"${command[@]}"
run_status=$?
set -e
after_snapshot_status=0
if [[ "${require_quiet_host_after}" == "0" ]]; then
  write_host_snapshot "${host_snapshot_after}"
  if [[ "$(host_is_quiet "${host_snapshot_after}")" != "true" ]]; then
    echo "host is busy after the play profile; continuing because NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER=0" >&2
  fi
elif [[ "${allow_busy_host}" == "1" ]]; then
  write_host_snapshot "${host_snapshot_after}"
elif ! capture_quiet_host_snapshot "${host_snapshot_after}" "after play profile"; then
  after_snapshot_status=1
fi

if (( run_status != 0 )); then
  echo "play profile command failed with status ${run_status}" >&2
  exit "${run_status}"
fi

if (( after_snapshot_status != 0 )); then
  echo "host is too busy after the play profile; refusing to treat this as gating evidence" >&2
  echo "max process CPU: $(host_snapshot_max_cpu "${host_snapshot_after}")% (limit ${max_host_process_cpu_percent}%)" >&2
  echo "total top-process CPU: $(host_snapshot_total_cpu "${host_snapshot_after}")% (limit ${max_host_total_cpu_percent}%)" >&2
  echo "top process: $(host_snapshot_top_process "${host_snapshot_after}")" >&2
  if [[ -n "${ignore_process_pattern}" ]]; then
    echo "ignored process pattern: ${ignore_process_pattern}" >&2
  fi
  echo "snapshot: ${host_snapshot_after}" >&2
  echo "Set NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER=0 only when post-run host load should be advisory." >&2
  exit 1
fi

if [[ ! -s "${profile_path}" ]]; then
  echo "play profile was not written: ${profile_path}" >&2
  exit 1
fi

identity_tmp="${profile_path}.identity.tmp"
jq \
  --arg repo_commit "${source_commit}" \
  --arg source_state "${source_state}" \
  --arg source_fingerprint "${source_fingerprint}" \
  '. + {
    repo_commit: $repo_commit,
    source_state: $source_state,
    source_fingerprint: $source_fingerprint
  }' \
  "${profile_path}" > "${identity_tmp}"
mv "${identity_tmp}" "${profile_path}"

echo
echo "Play profile summary:"
jq '{
  repo_commit,
  source_state,
  source_fingerprint,
  profile_kind,
  control_source,
  script,
  passed,
  armed,
  arming,
  duration_secs,
  target_duration_secs,
  warmup_excluded_secs,
  checks,
  activity,
  window_focus,
  frame_time,
  steady_frame_time,
  hitch_event_threshold_ms,
  hitch_events: [
    .hitch_events[]? | {
      frame,
      elapsed_secs,
      frame_time_ms,
      severity,
      steady,
      player_position: .snapshot.player_position,
      window: .snapshot.window,
      entity_count: .snapshot.entity_count,
      streaming: .snapshot.streaming,
      streaming_lod: .snapshot.streaming_lod,
      island_visuals: .snapshot.island_visuals,
      world_floor: .snapshot.world_floor,
      runtime_assets: .snapshot.runtime_assets
    }
  ],
  max
}' \
  "${profile_path}"

if [[ "${allow_failed_checks}" != "1" ]] \
  && ! jq -e '.passed == true' "${profile_path}" >/dev/null; then
  echo "play profile failed checks; inspect ${profile_path}" >&2
  exit 1
fi
