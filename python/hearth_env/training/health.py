from __future__ import annotations

from collections import Counter
from dataclasses import asdict, dataclass, field
from typing import Any


@dataclass
class EpisodeHealth:
    episodes: int = 0
    decisions: int = 0
    terminated: int = 0
    truncated: int = 0
    errors: int = 0
    end_turns: int = 0
    avoidable_end_turns: int = 0
    mana_available_at_end: float = 0.0
    max_mana_at_end: float = 0.0
    action_kinds: Counter[str] = field(default_factory=Counter)

    def add(
        self, episode: dict[str, Any], *, controlled_seats: set[int] | None = None
    ) -> None:
        self.episodes += 1
        self.terminated += int(bool(episode.get("terminated")))
        self.truncated += int(bool(episode.get("truncated")))
        self.errors += int(bool(episode.get("error")))
        for step in episode.get("steps", []):
            decision = step["decision"]
            if (
                controlled_seats is not None
                and int(decision.get("actor_seat", -1)) not in controlled_seats
            ):
                continue
            actions = decision.get("actions", [])
            index = int(step["action_index"])
            if index < 0 or index >= len(actions):
                self.errors += 1
                continue
            chosen = actions[index]
            kind = str(chosen.get("kind", "unknown"))
            self.action_kinds[kind] += 1
            self.decisions += 1
            if kind != "end_turn":
                continue
            self.end_turns += 1
            alternatives = {
                str(action.get("kind", "unknown"))
                for action in actions
                if action.get("kind") not in {"end_turn", "concede"}
            }
            self.avoidable_end_turns += int(bool(alternatives))
            player = decision.get("observation", {}).get("self_player", {})
            self.mana_available_at_end += float(player.get("mana", 0))
            self.max_mana_at_end += float(player.get("max_mana", 0))

    def summary(self) -> dict[str, Any]:
        result = asdict(self)
        result["action_kinds"] = dict(sorted(self.action_kinds.items()))
        result["truncation_rate"] = self.truncated / max(self.episodes, 1)
        result["avoidable_end_turn_rate"] = self.avoidable_end_turns / max(
            self.end_turns, 1
        )
        result["mean_unused_mana_at_end"] = self.mana_available_at_end / max(
            self.end_turns, 1
        )
        result["mana_utilization"] = 1.0 - (
            self.mana_available_at_end / max(self.max_mana_at_end, 1.0)
        )
        result["play_rate"] = (
            self.action_kinds["play_card"] + self.action_kinds["play_card_at"]
        ) / max(self.decisions, 1)
        result["attack_rate"] = self.action_kinds["attack"] / max(self.decisions, 1)
        return result


def health_gate(
    health: EpisodeHealth,
    *,
    max_avoidable_end_turn_rate: float = 0.05,
    max_truncation_rate: float = 0.01,
) -> list[str]:
    summary = health.summary()
    failures: list[str] = []
    if health.errors:
        failures.append(f"errors={health.errors}")
    if summary["avoidable_end_turn_rate"] > max_avoidable_end_turn_rate:
        failures.append(
            "avoidable_end_turn_rate="
            f"{summary['avoidable_end_turn_rate']:.3%} > "
            f"{max_avoidable_end_turn_rate:.3%}"
        )
    if summary["truncation_rate"] > max_truncation_rate:
        failures.append(
            f"truncation_rate={summary['truncation_rate']:.3%} > "
            f"{max_truncation_rate:.3%}"
        )
    return failures
