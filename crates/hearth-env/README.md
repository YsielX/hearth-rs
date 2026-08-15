# hearth-env

`hearth-env` is the learning-framework-neutral boundary around the authoritative
game. It contains environment mechanics, not learning code:

- perspective-normalized player observations;
- episode-stable, policy-local entity references instead of authoritative
  `EntityId` values;
- a normalized public-event history with optional observation windows;
- structured, indexed legal actions;
- decision tokens that reject stale action indices;
- deterministic reset while reusing the loaded Lua runtime;
- terminal zero-sum rewards and adapter-only step truncation.

The underlying `PlayerCommand` values remain private inside `HearthEnv`.
`hearth-core` continues to own game rules, legal-action validation, public game
facts, deterministic randomness, and replay.

The companion `hearth-env-py` crate exposes JSON at the Python ABI boundary.
The small `python/hearth_env` package converts it to ordinary dictionaries and
does not depend on Gymnasium, PyTorch, or a particular RL algorithm.

## Boundary

The engine owns rules, deterministic randomness, legal-action validation,
replay, and the information-safe `PlayerView` / `PlayerController` protocol.
`PlayerView::history` is a chronological, viewer-specific `PublicEvent` stream;
the aggregate card histories in this adapter are derived from that stream, not
from private engine caches. This adapter incrementally converts that stream to
perspective-normalized events, assigns public cursors with no hidden core-log
gaps, and can expose only the latest `history_limit` events while keeping the
derived aggregates complete. It also owns episode lifecycle, stable local
entity references, indexed actions, rewards, and time-limit truncation. Tensor
layouts, batching, vector workers, self-play matchmaking, and learning code
belong above this crate.

`max_steps = 0` disables adapter truncation. A terminal game returns seat-order
rewards (`[P1, P2]`); a time-limit truncation returns zero rewards and does not
write a synthetic result into the game state.

## Python smoke usage

Build/install the extension from the repository root with `maturin develop`,
then create an environment with a match configuration:

```python
from hearth_env import HearthEnv

deck = ["CS2_120"] * 20
env = HearthEnv(
    "data",
    {"decks": [deck, deck], "unrestricted": True},
    seed=7,
    history_limit=128,  # omit for the complete public event stream
)

decision = env.decision
transition = env.step(decision["actions"][0]["index"])
```

Action indices are valid only for the decision that contains them. The Python
facade supplies the corresponding decision token automatically; the Rust API
requires both values explicitly so stale actions fail rather than controlling
a later state accidentally. Choice options are structured as public card or
entity references when that identity is explicitly offered to the player;
arbitrary script continuation payloads remain opaque.
