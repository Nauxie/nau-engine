#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-target/player_pose_preview}"

cargo run --quiet --bin asset_fixture_audit -- --export-player-pose-preview "${output_dir}" > "${output_dir}.json"

chrome_bin="${CHROME_BIN:-}"
if [[ -z "${chrome_bin}" && -x "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" ]]; then
  chrome_bin="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
fi
render_timeout_secs="${NAU_PREVIEW_RENDER_TIMEOUT_SECS:-90}"

if [[ -n "${chrome_bin}" && -x "${chrome_bin}" ]]; then
  abs_output_dir="$(cd "${output_dir}" && pwd -P)"
  sheets=(
    player_pose_sheet
    player_anatomy_review_sheet
    player_rig_stress_review_sheet
    player_motion_integrity_review_sheet
    player_transition_pose_sheet
    glider_pose_sheet
    player_glider_attachment_sheet
  )

  render_sheet_png() {
    local sheet="$1"
    local svg_file="${abs_output_dir}/${sheet}.svg"
    local width
    local height
    width="$(sed -n 's/.*width="\([0-9][0-9]*\)".*/\1/p' "${svg_file}" | head -n 1)"
    height="$(sed -n 's/.*height="\([0-9][0-9]*\)".*/\1/p' "${svg_file}" | head -n 1)"
    width="${width:-1216}"
    height="${height:-2176}"
    local screenshot_file="${abs_output_dir}/${sheet}.png"
    rm -f "${screenshot_file}"
    local chrome_profile
    chrome_profile="$(mktemp -d)"
    "${chrome_bin}" \
      --headless=new \
      --disable-gpu \
      --disable-background-networking \
      --disable-default-apps \
      --disable-extensions \
      --hide-scrollbars \
      --no-first-run \
      --user-data-dir="${chrome_profile}" \
      --window-size="${width},${height}" \
      --screenshot="${screenshot_file}" \
      "file://${abs_output_dir}/${sheet}.svg" > /dev/null 2>&1 &

    local render_pid="$!"
    local elapsed_secs=0
    local previous_size=0
    local stable_size_ticks=0
    while kill -0 "${render_pid}" 2> /dev/null; do
      if [[ -s "${screenshot_file}" ]]; then
        local current_size
        current_size="$(wc -c < "${screenshot_file}")"
        if [[ "${current_size}" == "${previous_size}" ]]; then
          stable_size_ticks=$((stable_size_ticks + 1))
        else
          stable_size_ticks=0
          previous_size="${current_size}"
        fi
        if (( elapsed_secs >= 2 && stable_size_ticks >= 2 )); then
          kill "${render_pid}" 2> /dev/null || true
          wait "${render_pid}" 2> /dev/null || true
          rm -rf "${chrome_profile}"
          return 0
        fi
      fi
      if (( elapsed_secs >= render_timeout_secs )); then
        kill "${render_pid}" 2> /dev/null || true
        wait "${render_pid}" 2> /dev/null || true
        rm -rf "${chrome_profile}"
        return 1
      fi
      sleep 1
      elapsed_secs=$((elapsed_secs + 1))
    done
    local status=0
    wait "${render_pid}" || status="$?"
    rm -rf "${chrome_profile}"
    return "${status}"
  }

  for sheet in "${sheets[@]}"; do
    render_sheet_png "${sheet}"
    test -s "${abs_output_dir}/${sheet}.png"
  done
  cargo run --quiet --bin player_pose_preview_audit -- "${abs_output_dir}" \
    > "${abs_output_dir}/preview_audit.json"
elif [[ "${NAU_REQUIRE_PREVIEW_PNG:-0}" == "1" ]]; then
  echo "CHROME_BIN must point to a Chrome executable when NAU_REQUIRE_PREVIEW_PNG=1" >&2
  exit 1
fi

echo "player pose preview: ${output_dir}/player_pose_sheet.svg"
echo "player anatomy review preview: ${output_dir}/player_anatomy_review_sheet.svg"
echo "player rig stress review preview: ${output_dir}/player_rig_stress_review_sheet.svg"
echo "player motion integrity review preview: ${output_dir}/player_motion_integrity_review_sheet.svg"
echo "player transition pose preview: ${output_dir}/player_transition_pose_sheet.svg"
echo "glider pose preview: ${output_dir}/glider_pose_sheet.svg"
echo "player/glider attachment preview: ${output_dir}/player_glider_attachment_sheet.svg"
if [[ -f "${output_dir}/player_pose_sheet.png" ]]; then
  echo "player pose preview screenshot: ${output_dir}/player_pose_sheet.png"
fi
if [[ -f "${output_dir}/player_anatomy_review_sheet.png" ]]; then
  echo "player anatomy review screenshot: ${output_dir}/player_anatomy_review_sheet.png"
fi
if [[ -f "${output_dir}/player_rig_stress_review_sheet.png" ]]; then
  echo "player rig stress review screenshot: ${output_dir}/player_rig_stress_review_sheet.png"
fi
if [[ -f "${output_dir}/player_motion_integrity_review_sheet.png" ]]; then
  echo "player motion integrity review screenshot: ${output_dir}/player_motion_integrity_review_sheet.png"
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
if [[ -f "${output_dir}/preview_audit.json" ]]; then
  echo "player pose preview audit: ${output_dir}/preview_audit.json"
fi
