from __future__ import annotations

import unittest

from hearth_env.training.policies import HeuristicPolicy


def attack_decision(*, enemy_health: int = 30, enemy_attack: int = 4) -> dict:
    return {
        "observation": {
            "entities": [
                {
                    "entity": 1,
                    "area": "board",
                    "controller": "self_player",
                    "attack": 4,
                    "max_health": 5,
                    "damage": 0,
                },
                {
                    "entity": 2,
                    "area": "hero",
                    "controller": "opponent",
                    "max_health": 30,
                    "damage": 30 - enemy_health,
                    "armor": 0,
                },
                {
                    "entity": 3,
                    "area": "board",
                    "controller": "opponent",
                    "attack": enemy_attack,
                    "max_health": 4,
                    "damage": 1,
                },
            ]
        },
        "actions": [
            {"index": 0, "kind": "attack", "sources": [1], "target": 2},
            {"index": 1, "kind": "attack", "sources": [1], "target": 3},
        ],
    }


class HeuristicPolicyTest(unittest.TestCase):
    def test_prefers_favorable_board_trade_over_nonlethal_face(self) -> None:
        policy = HeuristicPolicy(seed=1, noise=0.0)
        self.assertEqual(policy.choose(attack_decision(), []), 1)

    def test_legal_lethal_has_priority_over_board_trade(self) -> None:
        policy = HeuristicPolicy(seed=1, noise=0.0)
        self.assertEqual(policy.choose(attack_decision(enemy_health=4), []), 0)


if __name__ == "__main__":
    unittest.main()
