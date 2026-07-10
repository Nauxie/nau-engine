#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 6 ]]; then
  echo "Usage: $0 <baseline_perf> <candidate_perf> <baseline_freeflight> <candidate_freeflight> <baseline_ground> <candidate_ground>" >&2
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${script_dir}/source_identity.sh"

baseline_perf="$1"
candidate_perf="$2"
baseline_freeflight="$3"
candidate_freeflight="$4"
baseline_ground="$5"
candidate_ground="$6"

for artifact in "$@"; do
  if [[ ! -s "${artifact}" ]]; then
    echo "missing evidence artifact: ${artifact}" >&2
    exit 1
  fi
done

current_commit="$(nau_source_commit)"
current_state="$(nau_source_state)"
current_fingerprint="$(nau_source_fingerprint)"
main_commit="$(git rev-parse origin/main)"

if [[ "${current_state}" != "clean" ]]; then
  echo "candidate source must be clean before evidence can be accepted" >&2
  exit 1
fi

require_identity() {
  local artifact="$1"
  local expected_commit="$2"
  local expected_fingerprint="$3"
  local expected_state="${4:-}"
  local actual_commit
  local actual_fingerprint
  local actual_state

  actual_commit="$(jq -er '.repo_commit' "${artifact}")"
  actual_fingerprint="$(jq -er '.source_fingerprint' "${artifact}")"
  actual_state="$(jq -er '.source_state' "${artifact}")"

  if [[ "${actual_commit}" != "${expected_commit}" ]]; then
    echo "artifact commit mismatch: ${artifact}: ${actual_commit} != ${expected_commit}" >&2
    exit 1
  fi
  if [[ "${actual_fingerprint}" != "${expected_fingerprint}" ]]; then
    echo "artifact source fingerprint mismatch: ${artifact}" >&2
    exit 1
  fi
  if [[ -n "${expected_state}" && "${actual_state}" != "${expected_state}" ]]; then
    echo "artifact source state mismatch: ${artifact}: ${actual_state} != ${expected_state}" >&2
    exit 1
  fi
}

baseline_fingerprint="$(jq -er '.source_fingerprint' "${baseline_perf}")"
baseline_state="$(jq -er '.source_state' "${baseline_perf}")"
require_identity "${baseline_perf}" "${main_commit}" "${baseline_fingerprint}" "${baseline_state}"
require_identity "${baseline_freeflight}" "${main_commit}" "${baseline_fingerprint}" "${baseline_state}"
require_identity "${baseline_ground}" "${main_commit}" "${baseline_fingerprint}" "${baseline_state}"

require_identity "${candidate_perf}" "${current_commit}" "${current_fingerprint}" "clean"
require_identity "${candidate_freeflight}" "${current_commit}" "${current_fingerprint}" "clean"
require_identity "${candidate_ground}" "${current_commit}" "${current_fingerprint}" "clean"

echo "World-floor evidence source identity checks passed."
