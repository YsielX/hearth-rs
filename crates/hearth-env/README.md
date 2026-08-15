# hearth-env

`hearth-env` is the learning-framework-neutral boundary around the authoritative
game. It contains environment mechanics, not learning code:

- perspective-normalized player observations;
- policy-local entity references instead of authoritative `EntityId` values;
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
from private engine caches. This adapter owns episode lifecycle, perspective
normalization, policy-local entity references, indexed actions, rewards, and
time-limit truncation. Tensor layouts, history windows, batching, vector
workers, self-play matchmaking, and learning code belong above this crate.

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
)

decision = env.decision
transition = env.step(decision["actions"][0]["index"])
```

Action indices are valid only for the decision that contains them. The Python
facade supplies the corresponding decision token automatically; the Rust API
requires both values explicitly so stale actions fail rather than controlling
a later state accidentally.
