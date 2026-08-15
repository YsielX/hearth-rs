# hearth-bot

`hearth-bot` is a deterministic, non-cheating baseline controller. It consumes only `PlayerView` plus the authoritative list of `LegalAction` values; it cannot read the opponent's hand, deck order, Secret identities, hidden aura sources, RNG, or replay.

Decision order:

1. If the currently legal attacks from friendly board minions contain enough damage to kill the opposing Hero, attack face until lethal.
2. Choose a currently legal combination that leaves as little Mana unspent as possible. Dynamic costs are supplied by the engine rather than reimplemented by the Bot.
3. Make advantageous minion trades.
4. Use a ready Location, then attack face when legal.
5. When Taunt or another rule prevents attacking face, take the least costly forced trade.
6. End the turn; never Concede voluntarily.

An advantageous trade must kill the defender and either preserve the attacker or exchange it for a strictly more valuable defender. Combat value uses current Attack and remaining Health, with small public-keyword premiums for Taunt, Divine Shield, Poisonous, Lifesteal, Windfury, Mega-Windfury, and Deathrattle. Divine Shield and Poisonous also affect the kill/survival calculation directly.

Run a Bot mirror:

```bash
cargo run -p hearth-cli --release -- play \
  --deck-one decks/quest_rogue.json \
  --deck-two decks/quest_rogue.json \
  --player-one bot \
  --player-two bot
```
