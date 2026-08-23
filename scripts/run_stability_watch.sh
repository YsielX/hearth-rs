#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_dir"

output_dir=${1:-runs/stability-watch-20260823}
episodes=${2:-10000}
manifest=runs/cpu-20260820/manifests/deck_split_manifest.json
mkdir -p "$output_dir"

mapfile -t deck_paths < <(jq -r '.splits[][] | .path' "$manifest")
if [[ ${#deck_paths[@]} -ne 354 ]]; then
    echo "expected 354 manifest decks, found ${#deck_paths[@]}" >&2
    exit 1
fi

deck_args=()
for deck_path in "${deck_paths[@]}"; do
    if [[ ! -f "$deck_path" ]]; then
        echo "deck does not exist: $deck_path" >&2
        exit 1
    fi
    deck_args+=(--deck "$deck_path")
done

export OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1
export PYTHONUNBUFFERED=1

.venv/bin/hearth-train \
    --device cpu \
    --workers 24 \
    --seed 30360823 \
    "${deck_args[@]}" \
    stability \
    --episodes "$episodes" \
    --output-dir "$output_dir" \
    2>&1 | tee -a "$output_dir/stability.log"
