# State-machine Fuzzer Library

This crate contains the deterministic state-machine fuzzing implementation and
the reusable `FuzzController`. It is a library only; the user-facing entry point
is the `fuzz` subcommand of `hearth-cli`:

```bash
cargo run -p hearth-cli --release -- fuzz --seeds 8 --steps 180
```

Keeping the campaign implementation outside ordinary integration tests ensures
that normal `cargo test --workspace` runs do not start fuzz campaigns. The
fuzzer generates class-legal decks, samples engine-enumerated legal actions,
validates state after every step, and checks replay determinism.
