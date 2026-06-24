#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-target/visual_content_export}"
audit_path="${output_dir}/audit.json"

cargo run -- --export-visual-content "${output_dir}"
cargo run --quiet --bin visual_content_audit -- "${output_dir}/manifest.json" > "${audit_path}"

echo "visual content export audit: ${audit_path}"
