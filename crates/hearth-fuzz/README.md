# State-machine Fuzzer

This crate contains the long-running, deterministic state-machine fuzzer and the reusable `FuzzController`. It is deliberately separate from ordinary integration tests: normal `cargo test --workspace` does not execute fuzz campaigns.

The fuzzer independently chooses one of the 11 Constructed classes for each player, then builds each 30-card deck only from that class's cards, Neutral cards, and compatible multi-class cards. For every generated game it repeatedly chooses from the engine's legal actions and checks:

- every enumerated command can be dispatched;
- state invariants still hold after every action;
- replay reconstructs the exact final state.

Run it through the main CLI:

```bash
cargo run -p hearth-cli --release -- fuzz --seeds 8 --steps 180
```

Or run the dedicated binary directly:

```bash
cargo run -p hearth-fuzz --release -- \
  --start-seed 10000 \
  --seeds 1000 \
  --steps 300
```

Each generated game assigns a `FuzzController` to both player slots. Controllers receive only their player-facing view and legal actions, while the campaign runner retains authoritative access solely for invariant and replay verification. Both entry points use the same implementation. Failures report the seed and action step. Re-run that single seed with `--start-seed <seed> --seeds 1` to reproduce it exactly.
