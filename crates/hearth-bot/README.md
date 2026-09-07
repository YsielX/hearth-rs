# hearth-bot

Heuristic controllers using `PlayerView` and the engine's legal actions. Card-aware evaluation also reads public card definitions. The bot has no access to hidden hands, deck order, or RNG state.

## Modules

| Module | Responsibility |
| --- | --- |
| `controller` | Difficulty settings and `PlayerController` implementations |
| `policy` | Decision flow and public action-selection entry points |
| `combat` | Board lethal, trades, and face attacks |
| `spending` | Mana combinations, target preferences, and locations |
| `effects` | Recognized card-effect descriptions |
| `evaluation` | Card-aware action scores and material estimates |
| `tests` | Shared fixtures, policy tests, and tactical evaluation tests |

`lib.rs` re-exports `BotDifficulty`, `DifficultyBot`, `SimpleBot`, `choose_action`, `choose_action_for`, `choose_action_with_cards`, and `position_value`.

The CLI, app, and Python environment use `choose_action_with_cards`. Recognized damage, healing, draw, and buff effects receive tactical scores; other effects use the base policy. Easy difficulty retains its simple action order.

Run a bot mirror:

```bash
cargo run -p hearth-cli --release -- play \
  --deck-one decks/quest_rogue.json \
  --deck-two decks/quest_rogue.json \
  --player-one bot \
  --player-two bot
```
