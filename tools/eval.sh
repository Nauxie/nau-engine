#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-baseline_route}"
output_dir="${2:-target/eval/${scenario}}"
extra_args=()

if [[ "${NAU_EVAL_NO_SCREENSHOT:-0}" == "1" ]]; then
  extra_args+=(--eval-no-screenshot)
fi

if [[ "${#extra_args[@]}" -gt 0 ]]; then
  cargo run -- --eval "${scenario}" --eval-output "${output_dir}" "${extra_args[@]}"
else
  cargo run -- --eval "${scenario}" --eval-output "${output_dir}"
fi

summary="${output_dir}/summary.json"
samples="${output_dir}/samples.ndjson"

if [[ ! -s "${summary}" ]]; then
  echo "missing eval summary: ${summary}" >&2
  exit 1
fi

if [[ ! -s "${samples}" ]]; then
  echo "missing eval samples: ${samples}" >&2
  exit 1
fi

if command -v jq >/dev/null 2>&1; then
  jq '{scenario, passed, metrics, checks, artifacts}' "${summary}"
else
  sed -n '1,220p' "${summary}"
fi
