from __future__ import annotations

import hashlib
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from hearth_env import HearthEnv

from .policies import Policy
from .rollout import play_episode


@dataclass
class Evaluation:
    wins: int = 0
    losses: int = 0
    draws: int = 0
    truncated: int = 0
    by_matchup: dict[str, Evaluation] = field(default_factory=dict)

    @property
    def games(self) -> int:
        return self.wins + self.losses + self.draws

    @property
    def score(self) -> float:
        return (self.wins + 0.5 * self.draws) / max(self.games, 1)

    def record(self, reward: float | None) -> None:
        if reward is None:
            self.truncated += 1
        elif reward > 0:
            self.wins += 1
        elif reward < 0:
            self.losses += 1
        else:
            self.draws += 1

    def summary(self) -> dict[str, float | int]:
        return {
            "games": self.games,
            "wins": self.wins,
            "losses": self.losses,
            "draws": self.draws,
            "truncated": self.truncated,
            "score": self.score,
        }


def _deck_label(config: dict[str, Any], seat: int) -> str:
    digest = hashlib.sha256("\0".join(config["decks"][seat]).encode()).hexdigest()[:8]
    return f"{config['classes'][seat]}-{digest}"


def paired_evaluate(
    env: HearthEnv,
    candidate: Callable[[int], Policy],
    opponent: Callable[[int], Policy],
    matches: list[dict[str, Any]],
    *,
    seed: int = 0,
) -> Evaluation:
    result = Evaluation()
    for index, config in enumerate(matches):
        game_seed = seed + index
        for candidate_seat in (0, 1):
            policies = [opponent(game_seed), opponent(game_seed ^ 0xA5A5)]
            policies[candidate_seat] = candidate(game_seed ^ 0x5A5A)
            episode = play_episode(env, policies, config, game_seed)
            opponent_seat = 1 - candidate_seat
            matchup = (
                f"{_deck_label(config, candidate_seat)}_vs_"
                f"{_deck_label(config, opponent_seat)}"
            )
            bucket = result.by_matchup.setdefault(matchup, Evaluation())
            if episode["truncated"]:
                result.record(None)
                bucket.record(None)
                continue
            reward = episode["rewards"][candidate_seat]
            result.record(reward)
            bucket.record(reward)
    return result
