#!/usr/bin/env bash
set -euo pipefail

output_root="${1:-target/world_content_gate}"

mkdir -p "${output_root}"

cargo test world::tests
./tools/terrain_export.sh "${output_root}/terrain"
./tools/visual_content_export.sh "${output_root}/visual_content"

for scenario in long_glide_visibility great_sky_plateau_route; do
  NAU_EVAL_SIM_ONLY=1 NAU_EVAL_ASSET_AUDIT=0 \
    ./tools/eval.sh "${scenario}" "${output_root}/${scenario}"
done

NAU_EVAL_ASSET_AUDIT=0 \
  ./tools/eval.sh camera_mouse_control "${output_root}/camera_mouse_control"

NAU_EVAL_SCREENSHOT=1 NAU_EVAL_ASSET_AUDIT=0 \
  ./tools/eval.sh great_sky_plateau_vistas "${output_root}/great_sky_plateau_vistas"

echo "world content gate: ${output_root}"
