#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-target/island_art_direction_gate}"
audit_path="${output_dir}/audit.json"

cargo run -- --export-visual-content "${output_dir}"
cargo run --quiet --bin island_art_direction_audit -- "${output_dir}/manifest.json" > "${audit_path}"

echo "island art-direction audit: ${audit_path}"
