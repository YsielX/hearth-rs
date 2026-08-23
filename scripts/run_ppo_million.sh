#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_dir"

run_dir=${1:-runs/ppo-million-20260823}
manifest=runs/cpu-20260820/manifests/deck_split_manifest.json
mkdir -p "$run_dir"

mapfile -t train_decks < <(jq -r '.splits.train[].path' "$manifest")
if [[ ${#train_decks[@]} -ne 255 ]]; then
    echo "expected 255 training decks, found ${#train_decks[@]}" >&2
    exit 1
fi

deck_args=()
for deck_path in "${train_decks[@]}"; do
    if [[ ! -f "$deck_path" ]]; then
        echo "training deck does not exist: $deck_path" >&2
        exit 1
    fi
    deck_args+=(--deck "$deck_path")
done

export OMP_NUM_THREADS=64
export MKL_NUM_THREADS=64
export PYTHONUNBUFFERED=1

checkpoint_args=(
    --init runs/ppo-board-control-20260821/ppo-256/league/snapshot-000016.pt
)
if [[ -f "$run_dir/latest.pt" ]]; then
    checkpoint_args=(--resume "$run_dir/latest.pt")
fi

.venv/bin/hearth-train \
    --device cpu \
    --workers 96 \
    --seed 20260823 \
    --batch-size 512 \
    --ppo-learning-rate 1e-5 \
    "${deck_args[@]}" \
    train-ppo \
    "${checkpoint_args[@]}" \
    --reference runs/ppo-board-control-20260821/corrected-bc.pt \
    --run-dir "$run_dir" \
    --iterations 48 \
    --episodes-per-iteration 512 \
    --ppo-epochs 2 \
    --reference-kl-coefficient 0.01 \
    --shaping-coefficient 0 \
    --checkpoint-every 1 \
    --league-snapshot-every 2 \
    --specialist-probability 0 \
    --curated-probability 0.5 \
    --perturb-probability 0.35 \
    2>&1 | tee -a "$run_dir/train.log"
