#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-target/wind_visual_export}"
audit_path="${output_dir}/audit.json"

cargo run -- --export-wind-visuals "${output_dir}"
cargo run --quiet --bin wind_visual_export_audit -- "${output_dir}/manifest.json" > "${audit_path}"

echo "wind visual export audit: ${audit_path}"
