#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-target/terrain_export}"
audit_path="${output_dir}/audit.json"

cargo run -- --export-terrain "${output_dir}"
cargo run --quiet --bin terrain_export_audit -- "${output_dir}/manifest.json" > "${audit_path}"

echo "terrain export audit: ${audit_path}"
