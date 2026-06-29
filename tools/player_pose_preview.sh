#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-target/player_pose_preview}"

cargo run --quiet --bin asset_fixture_audit -- --export-player-pose-preview "${output_dir}" > "${output_dir}.json"

echo "player pose preview: ${output_dir}/player_pose_sheet.svg"
echo "player transition pose preview: ${output_dir}/player_transition_pose_sheet.svg"
echo "glider pose preview: ${output_dir}/glider_pose_sheet.svg"
echo "player pose preview manifest: ${output_dir}/manifest.json"
