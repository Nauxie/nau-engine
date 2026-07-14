#!/usr/bin/env bash
set -euo pipefail

max_host_process_cpu_percent="${NAU_PERF_MAX_HOST_PROCESS_CPU_PERCENT:-80}"
max_host_total_cpu_percent="${NAU_PERF_MAX_HOST_TOTAL_CPU_PERCENT:-160}"
wait_secs="${NAU_PERF_HOST_PREFLIGHT_WAIT_SECS:-0}"
poll_secs="${NAU_PERF_HOST_PREFLIGHT_POLL_SECS:-15}"
output_path="${NAU_PERF_HOST_PREFLIGHT_OUTPUT:-target/eval/perf_host_preflight/latest.txt}"
default_ignore_process_pattern="${NAU_PERF_DEFAULT_IGNORE_PROCESS_PATTERN-}"
ignore_process_pattern="${NAU_PERF_IGNORE_PROCESS_PATTERN-${default_ignore_process_pattern}}"

for value_name in max_host_process_cpu_percent max_host_total_cpu_percent wait_secs poll_secs; do
  value="${!value_name}"
  if ! [[ "${value}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "${value_name} must be numeric, got: ${value}" >&2
    exit 2
  fi
done

write_host_snapshot() {
  local snapshot="$1"
  mkdir -p "$(dirname "${snapshot}")"
  {
    printf 'generated_at\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    if [[ -n "${ignore_process_pattern}" ]]; then
      printf 'ignored_process_pattern\t%s\n' "${ignore_process_pattern}"
    fi
    printf 'thermal_status\n'
    pmset -g therm 2>/dev/null || true
    printf '\ntop_cpu_processes\n'
    ps -Ao pid,pcpu,pmem,comm | sort -k2 -nr | sed -n '1,20p'
  } > "${snapshot}"
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

start_secs="${SECONDS}"
while true; do
  write_host_snapshot "${output_path}"
  max_cpu="$(host_snapshot_max_cpu "${output_path}")"
  total_cpu="$(host_snapshot_total_cpu "${output_path}")"
  top_process="$(host_snapshot_top_process "${output_path}")"

  if [[ "$(host_is_quiet "${output_path}")" == "true" ]]; then
    printf 'host_quiet\ttrue\n'
    printf 'max_process_cpu_percent\t%s\n' "${max_cpu}"
    printf 'max_allowed_cpu_percent\t%s\n' "${max_host_process_cpu_percent}"
    printf 'total_top_process_cpu_percent\t%s\n' "${total_cpu}"
    printf 'max_allowed_total_cpu_percent\t%s\n' "${max_host_total_cpu_percent}"
    if [[ -n "${ignore_process_pattern}" ]]; then
      printf 'ignored_process_pattern\t%s\n' "${ignore_process_pattern}"
    fi
    printf 'top_process\t%s\n' "${top_process}"
    printf 'snapshot\t%s\n' "${output_path}"
    exit 0
  fi

  elapsed_secs=$((SECONDS - start_secs))
  if awk -v elapsed="${elapsed_secs}" -v wait="${wait_secs}" 'BEGIN { exit !(elapsed >= wait) }'; then
    printf 'host_quiet\tfalse\n' >&2
    printf 'max_process_cpu_percent\t%s\n' "${max_cpu}" >&2
    printf 'max_allowed_cpu_percent\t%s\n' "${max_host_process_cpu_percent}" >&2
    printf 'total_top_process_cpu_percent\t%s\n' "${total_cpu}" >&2
    printf 'max_allowed_total_cpu_percent\t%s\n' "${max_host_total_cpu_percent}" >&2
    if [[ -n "${ignore_process_pattern}" ]]; then
      printf 'ignored_process_pattern\t%s\n' "${ignore_process_pattern}" >&2
    fi
    printf 'top_process\t%s\n' "${top_process}" >&2
    printf 'snapshot\t%s\n' "${output_path}" >&2
    printf 'Set NAU_PERF_HOST_PREFLIGHT_WAIT_SECS to wait for a quiet host.\n' >&2
    exit 1
  fi

  printf 'host still busy: max_process_cpu_percent=%s max_allowed=%s total_top_process_cpu_percent=%s total_allowed=%s top_process=%s\n' \
    "${max_cpu}" "${max_host_process_cpu_percent}" "${total_cpu}" "${max_host_total_cpu_percent}" "${top_process}" >&2
  sleep "${poll_secs}"
done
