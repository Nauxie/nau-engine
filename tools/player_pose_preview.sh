#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-target/player_pose_preview}"

cargo run --quiet --bin asset_fixture_audit -- --export-player-pose-preview "${output_dir}" > "${output_dir}.json"

chrome_bin="${CHROME_BIN:-}"
if [[ -z "${chrome_bin}" && -x "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" ]]; then
  chrome_bin="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
fi

if [[ -n "${chrome_bin}" && -x "${chrome_bin}" ]]; then
  abs_output_dir="$(cd "${output_dir}" && pwd -P)"
  chrome_profile="$(mktemp -d)"
  trap 'rm -rf "${chrome_profile}"' EXIT

  render_sheet_png() {
    local sheet="$1"
    "${chrome_bin}" \
      --headless \
      --disable-gpu \
      --disable-background-networking \
      --disable-default-apps \
      --hide-scrollbars \
      --no-first-run \
      --user-data-dir="${chrome_profile}" \
      --window-size=1216,2176 \
      --screenshot="${abs_output_dir}/${sheet}.png" \
      "file://${abs_output_dir}/${sheet}.svg" > /dev/null 2>&1 &

    local render_pid="$!"
    local elapsed_secs=0
    while kill -0 "${render_pid}" 2> /dev/null; do
      if (( elapsed_secs >= 20 )); then
        kill "${render_pid}" 2> /dev/null || true
        wait "${render_pid}" 2> /dev/null || true
        return 1
      fi
      sleep 1
      elapsed_secs=$((elapsed_secs + 1))
    done
    wait "${render_pid}"
  }

  for sheet in player_pose_sheet player_transition_pose_sheet glider_pose_sheet player_glider_attachment_sheet; do
    render_sheet_png "${sheet}" || true
  done
fi

echo "player pose preview: ${output_dir}/player_pose_sheet.svg"
echo "player transition pose preview: ${output_dir}/player_transition_pose_sheet.svg"
echo "glider pose preview: ${output_dir}/glider_pose_sheet.svg"
echo "player/glider attachment preview: ${output_dir}/player_glider_attachment_sheet.svg"
if [[ -f "${output_dir}/player_pose_sheet.png" ]]; then
  echo "player pose preview screenshot: ${output_dir}/player_pose_sheet.png"
fi
if [[ -f "${output_dir}/player_transition_pose_sheet.png" ]]; then
  echo "player transition pose preview screenshot: ${output_dir}/player_transition_pose_sheet.png"
fi
if [[ -f "${output_dir}/glider_pose_sheet.png" ]]; then
  echo "glider pose preview screenshot: ${output_dir}/glider_pose_sheet.png"
fi
if [[ -f "${output_dir}/player_glider_attachment_sheet.png" ]]; then
  echo "player/glider attachment preview screenshot: ${output_dir}/player_glider_attachment_sheet.png"
fi
echo "player pose preview manifest: ${output_dir}/manifest.json"
