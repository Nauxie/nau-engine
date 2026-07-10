#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

output_dir="${1:-target/eval/world_floor_visual_evidence}"
preview_width="${NAU_WORLD_FLOOR_PREVIEW_WIDTH:-1280}"
min_preview_count="${NAU_WORLD_FLOOR_MIN_PREVIEW_COUNT:-4}"
min_preview_bytes="${NAU_WORLD_FLOOR_MIN_PREVIEW_BYTES:-8192}"

file_size_bytes() {
  if stat -f%z "$1" >/dev/null 2>&1; then
    stat -f%z "$1"
  else
    stat -c%s "$1"
  fi
}

if ! [[ "${preview_width}" =~ ^[0-9]+$ ]] || (( preview_width == 0 )); then
  echo "NAU_WORLD_FLOOR_PREVIEW_WIDTH must be a positive integer" >&2
  exit 2
fi

if ! [[ "${min_preview_count}" =~ ^[0-9]+$ ]]; then
  echo "NAU_WORLD_FLOOR_MIN_PREVIEW_COUNT must be a non-negative integer" >&2
  exit 2
fi

if ! [[ "${min_preview_bytes}" =~ ^[0-9]+$ ]]; then
  echo "NAU_WORLD_FLOOR_MIN_PREVIEW_BYTES must be a non-negative integer" >&2
  exit 2
fi

NAU_EVAL_SCREENSHOT=1 \
  NAU_EVAL_VISUAL_AUDIT="${NAU_EVAL_VISUAL_AUDIT:-0}" \
  NAU_EVAL_ASSET_AUDIT="${NAU_EVAL_ASSET_AUDIT:-0}" \
  NAU_EVAL_SEMANTIC_SCENE_AUDIT="${NAU_EVAL_SEMANTIC_SCENE_AUDIT:-0}" \
  ./tools/eval.sh long_glide_visibility "${output_dir}"

preview_dir="${output_dir}/previews"
rm -rf "${preview_dir}"
mkdir -p "${preview_dir}"

if command -v sips >/dev/null 2>&1; then
  while IFS= read -r image_path; do
    [[ -n "${image_path}" ]] || continue
    sips -Z "${preview_width}" "${image_path}" \
      --out "${preview_dir}/$(basename "${image_path}")" >/dev/null
  done < <(
    {
      find "${output_dir}/checkpoints" -maxdepth 1 -type f -name '*.png' 2>/dev/null
      find "${output_dir}" -maxdepth 1 -type f -name 'final.png' 2>/dev/null
    } | sort
  )
else
  while IFS= read -r image_path; do
    [[ -n "${image_path}" ]] || continue
    cp "${image_path}" "${preview_dir}/$(basename "${image_path}")"
  done < <(
    {
      find "${output_dir}/checkpoints" -maxdepth 1 -type f -name '*.png' 2>/dev/null
      find "${output_dir}" -maxdepth 1 -type f -name 'final.png' 2>/dev/null
    } | sort
  )
fi

preview_count=0
while IFS= read -r preview_path; do
  [[ -n "${preview_path}" ]] || continue
  preview_count=$((preview_count + 1))
  preview_size="$(file_size_bytes "${preview_path}")"
  if (( preview_size < min_preview_bytes )); then
    echo "suspiciously small world-floor preview (${preview_size} bytes): ${preview_path}" >&2
    exit 1
  fi
done < <(find "${preview_dir}" -maxdepth 1 -type f -name '*.png' | sort)

if (( preview_count < min_preview_count )); then
  echo "expected at least ${min_preview_count} world-floor preview PNGs, found ${preview_count}: ${preview_dir}" >&2
  exit 1
fi

cat <<EOF

World-floor visual evidence written to:
  ${output_dir}

Preview PNGs:
  ${preview_dir}

This is a screenshot evidence bundle, not a replacement for the final short
human smoke test before merge/release.
EOF
