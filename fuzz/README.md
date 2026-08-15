# State-machine Fuzzer

This directory contains the long-running, deterministic state-machine fuzzer. It is deliberately separate from `crates/*/tests`: normal `cargo test --workspace` runs focused regression tests and does not execute fuzz campaigns.

The fuzzer independently chooses one of the 11 Constructed classes for each player, then builds each 30-card deck only from that class's cards, Neutral cards, and compatible multi-class cards. For every generated game it repeatedly chooses from the engine's legal actions and checks:

- every enumerated command can be dispatched;
- state invariants still hold after every action;
- replay reconstructs the exact final state.

Run a quick deterministic campaign:

```bash
cargo run -p hearth-fuzz --release -- --seeds 8 --steps 180
```

Run a larger campaign or resume at a known seed:

```bash
cargo run -p hearth-fuzz --release -- \
  --start-seed 10000 \
  --seeds 1000 \
  --steps 300
```

Failures report the seed and action step. Re-run that single seed with `--start-seed <seed> --seeds 1` to reproduce it exactly.
