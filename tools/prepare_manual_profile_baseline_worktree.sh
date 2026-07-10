#!/usr/bin/env bash
set -euo pipefail

if ! command -v git >/dev/null 2>&1; then
  echo "git is required to prepare a play-profile baseline worktree" >&2
  exit 1
fi

repo_root="$(git rev-parse --show-toplevel)"
baseline_ref="${NAU_MANUAL_PROFILE_BASELINE_REF:-origin/main}"
worktree_dir="${1:-${repo_root}/target/perf_worktrees/manual_profile_main}"
profile_path="${2:-${repo_root}/target/eval/play_profile/main_scripted_freeflight.json}"
run_profile="${NAU_RUN_PLAY_PROFILE:-${NAU_RUN_MANUAL_PROFILE:-0}}"
refresh_worktree="${NAU_REFRESH_PLAY_PROFILE_BASELINE_WORKTREE:-auto}"

case "${worktree_dir}" in
  /*) ;;
  *) worktree_dir="${repo_root}/${worktree_dir}" ;;
esac

case "${profile_path}" in
  /*) ;;
  *) profile_path="${repo_root}/${profile_path}" ;;
esac

case "${run_profile}" in
  0 | 1) ;;
  *)
    echo "NAU_RUN_PLAY_PROFILE must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${refresh_worktree}" in
  auto | 0 | 1) ;;
  *)
    echo "NAU_REFRESH_PLAY_PROFILE_BASELINE_WORKTREE must be auto, 0, or 1" >&2
    exit 2
    ;;
esac

generated_worktree_path() {
  case "${worktree_dir}" in
    "${repo_root}/target/perf_worktrees/"*) return 0 ;;
    *) return 1 ;;
  esac
}

refresh_generated_worktree_allowed() {
  [[ "${refresh_worktree}" == "1" ]] \
    || [[ "${refresh_worktree}" == "auto" && generated_worktree_path ]]
}

refresh_baseline_worktree() {
  if ! refresh_generated_worktree_allowed; then
    cat >&2 <<EOF
baseline worktree is dirty and does not match the current profiling instrumentation patch: ${worktree_dir}
Review or remove it before preparing a fresh instrumented baseline.
Set NAU_REFRESH_PLAY_PROFILE_BASELINE_WORKTREE=1 to refresh a custom disposable worktree.
EOF
    exit 1
  fi

  echo "Refreshing stale generated play-profile baseline worktree: ${worktree_dir}" >&2
  git -C "${repo_root}" worktree remove --force "${worktree_dir}"
  git -C "${repo_root}" worktree add --detach "${worktree_dir}" "${baseline_ref}"
}

ensure_world_floor_baseline_stub() {
  if [[ ! -f "${worktree_dir}/src/main.rs" ]]; then
    return
  fi
  if ! grep -q '^mod world_floor_runtime;' "${worktree_dir}/src/main.rs"; then
    return
  fi
  if git -C "${worktree_dir}" ls-files --error-unmatch src/world_floor_runtime.rs >/dev/null 2>&1; then
    return
  fi

  cat >"${worktree_dir}/src/world_floor_runtime.rs" <<'EOF'
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, Resource)]
pub(crate) struct WorldFloorDiagnostics {
    pub(crate) visible_tile_count: usize,
    pub(crate) max_visible_tile_count: usize,
    pub(crate) resident_tile_count: usize,
    pub(crate) max_resident_tile_count: usize,
    pub(crate) initial_spawned_tile_count: usize,
    pub(crate) spawned_tiles_this_frame: usize,
    pub(crate) despawned_tiles_this_frame: usize,
    pub(crate) max_spawned_tiles_per_frame: usize,
    pub(crate) max_despawned_tiles_per_frame: usize,
    pub(crate) total_spawned_tiles: usize,
    pub(crate) total_despawned_tiles: usize,
    pub(crate) mesh_vertex_count: usize,
    pub(crate) mesh_triangle_count: usize,
    pub(crate) material_count: usize,
    pub(crate) biome_count: usize,
    pub(crate) terrain_feature_count: usize,
    pub(crate) color_band_count: usize,
    pub(crate) river_vertex_count: usize,
    pub(crate) min_height_y: f32,
    pub(crate) max_height_y: f32,
    pub(crate) relief_range_m: f32,
    pub(crate) active_radius_tiles: i32,
    pub(crate) tile_size_m: f32,
}

pub(crate) fn update_world_floor_streaming() {}
EOF
}

required_paths=(
  src/main.rs
  src/app_tests.rs
  src/camera_runtime.rs
  src/eval_runtime.rs
  src/play_profile_runtime.rs
  src/player_runtime.rs
  src/scene_setup_runtime.rs
  src/eval/accumulator.rs
  src/eval/accumulator/summary_report/derived.rs
  src/eval/accumulator/summary_report/metrics_summary.rs
  src/eval/accumulator/world.rs
  src/eval/sample/builders.rs
  src/eval/sample/json.rs
  src/eval/sample/types.rs
  src/eval/summary.rs
  src/eval_app_runtime.rs
  src/eval_app_runtime/metrics.rs
  src/eval_app_runtime/scene.rs
  tools/manual_play_profile.sh
  tools/scripted_play_profile.sh
)

instrumentation_paths=(
  src/main.rs
  src/app_tests.rs
  src/camera_runtime.rs
  src/eval_runtime.rs
  src/play_profile_runtime.rs
  src/player_runtime.rs
  src/scene_setup_runtime.rs
  src/eval/accumulator.rs
  src/eval/accumulator/summary_report/derived.rs
  src/eval/accumulator/summary_report/metrics_summary.rs
  src/eval/accumulator/world.rs
  src/eval/sample/builders.rs
  src/eval/sample/json.rs
  src/eval/sample/types.rs
  src/eval/summary.rs
  src/eval_app_runtime.rs
  src/eval_app_runtime/metrics.rs
  src/eval_app_runtime/scene.rs
)

for path in "${required_paths[@]}"; do
  if [[ ! -e "${repo_root}/${path}" ]]; then
    echo "missing required profiling path in source worktree: ${path}" >&2
    exit 1
  fi
done

mkdir -p "$(dirname "${worktree_dir}")" "$(dirname "${profile_path}")"

if [[ ! -e "${worktree_dir}" ]]; then
  git -C "${repo_root}" worktree add --detach "${worktree_dir}" "${baseline_ref}"
fi

if [[ ! -d "${worktree_dir}/.git" && ! -f "${worktree_dir}/.git" ]]; then
  echo "baseline worktree does not look like a git worktree: ${worktree_dir}" >&2
  exit 1
fi

tmp_patch="$(mktemp)"
existing_patch="$(mktemp)"
trap 'rm -f "${tmp_patch}" "${existing_patch}"' EXIT

(
  cd "${repo_root}"
  git diff "${baseline_ref}" -- "${instrumentation_paths[@]}"
  if ! git ls-files --error-unmatch src/play_profile_runtime.rs >/dev/null 2>&1; then
    git diff --no-index -- /dev/null src/play_profile_runtime.rs || true
  fi
) >"${tmp_patch}"

if [[ ! -s "${tmp_patch}" ]]; then
  echo "no play-profile instrumentation patch was generated" >&2
  exit 1
fi

if [[ -n "$(git -C "${worktree_dir}" status --porcelain)" ]]; then
  ensure_world_floor_baseline_stub
  (
    cd "${worktree_dir}"
    git diff -- "${instrumentation_paths[@]}"
    if [[ -f src/play_profile_runtime.rs ]] \
      && ! git ls-files --error-unmatch src/play_profile_runtime.rs >/dev/null 2>&1; then
      git diff --no-index -- /dev/null src/play_profile_runtime.rs || true
    fi
  ) >"${existing_patch}"

  if cmp -s "${tmp_patch}" "${existing_patch}"; then
    baseline_commit="$(git -C "${worktree_dir}" rev-parse HEAD)"
    cat <<EOF
Prepared instrumented play-profile baseline worktree.

Baseline ref: ${baseline_ref}
Baseline commit: ${baseline_commit}
Worktree: ${worktree_dir}
Profile: ${profile_path}

The worktree already contains the current profiling instrumentation patch.
Do not use it as merge evidence for gameplay/content changes.
EOF

    printf 'Perf baseline command: (cd %q && NAU_PERF_VISIBLE_WINDOW=1 NAU_PERF_CAPTURE_SCREENSHOT=0 NAU_PERF_OUTPUT_DIR=%q %q baseline_route long_glide_visibility)\n' \
      "${worktree_dir}" \
      "${repo_root}/target/eval/perf_baseline/main_visible_no_screenshot" \
      "${repo_root}/tools/perf_baseline.sh"
    printf 'Scripted profile command: (cd %q && %q %q)\n' \
      "${worktree_dir}" "${repo_root}/tools/scripted_play_profile.sh" "${profile_path}"
    printf 'Manual smoke profile command: (cd %q && %q %q)\n' \
      "${worktree_dir}" "${repo_root}/tools/manual_play_profile.sh" "${profile_path}"

    if [[ "${run_profile}" == "1" ]]; then
      (cd "${worktree_dir}" && "${repo_root}/tools/scripted_play_profile.sh" "${profile_path}")
    fi
    exit 0
  fi

  refresh_baseline_worktree
fi

git -C "${worktree_dir}" apply --check "${tmp_patch}"
git -C "${worktree_dir}" apply "${tmp_patch}"
ensure_world_floor_baseline_stub

baseline_commit="$(git -C "${worktree_dir}" rev-parse HEAD)"
cat <<EOF
Prepared instrumented play-profile baseline worktree.

Baseline ref: ${baseline_ref}
Baseline commit: ${baseline_commit}
Worktree: ${worktree_dir}
Profile: ${profile_path}

This worktree is intentionally dirty with profiling instrumentation only. Do not use it as merge evidence for gameplay/content changes.
EOF

printf 'Perf baseline command: (cd %q && NAU_PERF_VISIBLE_WINDOW=1 NAU_PERF_CAPTURE_SCREENSHOT=0 NAU_PERF_OUTPUT_DIR=%q %q baseline_route long_glide_visibility)\n' \
  "${worktree_dir}" \
  "${repo_root}/target/eval/perf_baseline/main_visible_no_screenshot" \
  "${repo_root}/tools/perf_baseline.sh"
printf 'Scripted profile command: (cd %q && %q %q)\n' \
  "${worktree_dir}" "${repo_root}/tools/scripted_play_profile.sh" "${profile_path}"
printf 'Manual smoke profile command: (cd %q && %q %q)\n' \
  "${worktree_dir}" "${repo_root}/tools/manual_play_profile.sh" "${profile_path}"

if [[ "${run_profile}" == "1" ]]; then
  (cd "${worktree_dir}" && "${repo_root}/tools/scripted_play_profile.sh" "${profile_path}")
fi
