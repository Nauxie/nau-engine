#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -gt 2 ]]; then
  echo "Usage: $0 [profile.json] [freeflight|ground_traversal]" >&2
  exit 2
fi

script_name="${2:-${NAU_PLAY_PROFILE_SCRIPT:-freeflight}}"
profile_path="${1:-target/eval/play_profile/candidate_scripted_${script_name}.json}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

case "${script_name}" in
  freeflight | ground_traversal) ;;
  *)
    echo "script must be freeflight or ground_traversal: ${script_name}" >&2
    exit 2
    ;;
esac

NAU_MANUAL_PROFILE_HOST_WAIT_SECS="${NAU_MANUAL_PROFILE_HOST_WAIT_SECS:-180}" \
  NAU_PLAY_PROFILE_SCRIPT="${script_name}" \
  "${script_dir}/manual_play_profile.sh" "${profile_path}"
