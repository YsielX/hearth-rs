from __future__ import annotations

import hashlib
import random
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from typing import Any

from hearth_env import HearthEnv

from .policies import Policy
from .health import EpisodeHealth
from .rollout import play_episode


@dataclass
class Evaluation:
    wins: int = 0
    losses: int = 0
    draws: int = 0
    truncated: int = 0
    by_matchup: dict[str, Evaluation] = field(default_factory=dict)
    health: EpisodeHealth = field(default_factory=EpisodeHealth)
    paired_scores: list[float] = field(default_factory=list)
    by_seat: dict[int, Evaluation] = field(default_factory=dict)

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
        interval = None
        if len(self.paired_scores) >= 2:
            rng = random.Random(0)
            n = len(self.paired_scores)
            boot = sorted(
                sum(rng.choices(self.paired_scores, k=n)) / n for _ in range(2000)
            )
            interval = [boot[49], boot[1949]]
        return {
            "games": self.games,
            "wins": self.wins,
            "losses": self.losses,
            "draws": self.draws,
            "truncated": self.truncated,
            "score": self.score,
            "health": self.health.summary(),
            "complete_pair_groups": len(self.paired_scores),
            "paired_score": sum(self.paired_scores) / len(self.paired_scores)
            if self.paired_scores
            else None,
            "paired_score_ci95": interval,
            "by_seat": {
                str(seat): value.summary() for seat, value in self.by_seat.items()
            },
        }


def _deck_label(config: dict[str, Any], seat: int) -> str:
    digest = hashlib.sha256(
        "\0".join(sorted(config["decks"][seat])).encode()
    ).hexdigest()[:8]
    return f"{config['classes'][seat]}-{digest}"


def paired_configurations(base_config: dict[str, Any], *, swap_decks: bool = True):
    for swap in ((False, True) if swap_decks else (False,)):
        config = dict(base_config)
        if swap:
            for key in ("decks", "hero_powers", "classes", "sideboards"):
                if key in config:
                    config[key] = list(reversed(config[key]))
        for candidate_seat in (0, 1):
            yield config, candidate_seat


def paired_evaluate(
    env: HearthEnv,
    candidate: Callable[[int], Policy],
    opponent: Callable[[int], Policy],
    matches: list[dict[str, Any]],
    *,
    seed: int = 0,
    episodes: Sequence[dict[str, Any]] | None = None,
    swap_decks: bool = True,
) -> Evaluation:
    group_size = 4 if swap_decks else 2
    if episodes is not None and len(episodes) != group_size * len(matches):
        raise ValueError(f"evaluation requires exactly {group_size} ordered episodes per match")
    episode_iter = iter(episodes) if episodes is not None else None
    result = Evaluation()
    for index, base_config in enumerate(matches):
        game_seed = seed + index
        group_scores = []
        matchup_scores: dict[str, list[float]] = {}
        for config, candidate_seat in paired_configurations(base_config, swap_decks=swap_decks):
            if episode_iter is None:
                policies = [opponent(game_seed), opponent(game_seed ^ 0xA5A5)]
                policies[candidate_seat] = candidate(game_seed ^ 0x5A5A)
                episode = play_episode(env, policies, config, game_seed)
            else:
                episode = next(episode_iter)
            if episode.get("error"):
                raise RuntimeError(f"evaluation environment error: {episode['error']}")
            result.health.add(episode, controlled_seats={candidate_seat})
            opponent_seat = 1 - candidate_seat
            matchup = (
                f"{_deck_label(config, candidate_seat)}_vs_"
                f"{_deck_label(config, opponent_seat)}"
            )
            bucket = result.by_matchup.setdefault(matchup, Evaluation())
            bucket.health.add(episode, controlled_seats={candidate_seat})
            seat_bucket = result.by_seat.setdefault(candidate_seat, Evaluation())
            seat_bucket.health.add(episode, controlled_seats={candidate_seat})
            if episode["truncated"]:
                result.record(None)
                bucket.record(None)
                seat_bucket.record(None)
                continue
            reward = episode["rewards"][candidate_seat]
            result.record(reward)
            bucket.record(reward)
            seat_bucket.record(reward)
            score = (float(reward) + 1) / 2
            group_scores.append(score)
            matchup_scores.setdefault(matchup, []).append(score)
        if len(group_scores) == group_size:
            result.paired_scores.append(sum(group_scores) / group_size)
            for matchup, scores in matchup_scores.items():
                result.by_matchup[matchup].paired_scores.append(
                    sum(scores) / len(scores)
                )
    return result
