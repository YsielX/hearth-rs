#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_dir"

checkpoint=${1:?usage: run_milestone_eval.sh CHECKPOINT OUTPUT [MATCHES] [OPPONENT]}
output=${2:?usage: run_milestone_eval.sh CHECKPOINT OUTPUT [MATCHES] [OPPONENT]}
matches=${3:-16}
opponent=${4:-runs/ppo-board-control-20260821/ppo-256/league/snapshot-000016.pt}
manifest=runs/cpu-20260820/manifests/deck_split_manifest.json

mapfile -t deck_paths < <(jq -r '.splits.validation[].path' "$manifest")
if [[ ${#deck_paths[@]} -eq 0 ]]; then
    echo "validation split is empty" >&2
    exit 1
fi

deck_args=()
for deck_path in "${deck_paths[@]}"; do
    if [[ ! -f "$deck_path" ]]; then
        echo "validation deck does not exist: $deck_path" >&2
        exit 1
    fi
    deck_args+=(--deck "$deck_path")
done

mkdir -p "$(dirname "$output")"
export OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1
export PYTHONUNBUFFERED=1

.venv/bin/hearth-train \
    --device cpu \
    --seed 40460823 \
    "${deck_args[@]}" \
    evaluate \
    --checkpoint "$checkpoint" \
    --opponent-checkpoint "$opponent" \
    --matches "$matches" \
    --curated-probability 1 \
    --perturb-probability 0 \
    --output "$output"
