from __future__ import annotations

import io
import unittest
from contextlib import redirect_stdout

from hearth_env.training.interactive import describe_action, render_state


class InteractiveRenderingTests(unittest.TestCase):
    def test_discover_actions_render_the_localized_card_names(self) -> None:
        decision = {
            "observation": {
                "pending_choice": {
                    "prompt": "Discover a spell",
                    "options": [
                        {
                            "label": "Fireball",
                            "value": {"kind": "card", "card_id": "CS2_029"},
                        },
                        {
                            "label": "Frostbolt",
                            "value": {"kind": "card", "card_id": "CS2_024"},
                        },
                    ],
                },
                "entities": [],
            }
        }

        self.assertEqual(
            describe_action(
                decision,
                {"kind": "choose", "choice_index": 0},
                {"CS2_029": "火球术", "CS2_024": "寒冰箭"},
            ),
            "选择 火球术",
        )
        self.assertEqual(
            describe_action(
                decision,
                {"kind": "choose", "choice_index": 1},
                {"CS2_029": "火球术", "CS2_024": "寒冰箭"},
            ),
            "选择 寒冰箭",
        )

    def test_state_renders_both_weapons_with_remaining_durability(self) -> None:
        player = {
            "class": "warrior",
            "hero": 1,
            "weapon": 3,
            "board": [],
            "hand": [],
            "hand_size": 0,
            "deck_size": 20,
            "mana": 3,
            "max_mana": 4,
        }
        opponent = {
            "class": "paladin",
            "hero": 2,
            "weapon": 4,
            "board": [],
            "hand": [],
            "hand_size": 2,
            "deck_size": 18,
            "mana": 2,
            "max_mana": 4,
        }
        decision = {
            "observation": {
                "turn": 4,
                "phase": "main",
                "self_player": player,
                "opponent": opponent,
                "entities": [
                    {"entity": 1, "card_id": "builtin_hero", "kind": "hero", "max_health": 30, "damage": 4, "armor": 2, "cost": 0},
                    {"entity": 2, "card_id": "builtin_hero", "kind": "hero", "max_health": 30, "damage": 6, "armor": 0, "cost": 0},
                    {"entity": 3, "card_id": "WEAPON_ONE", "kind": "weapon", "attack": 3, "max_health": 2, "damage": 1, "cost": 3},
                    {"entity": 4, "card_id": "WEAPON_TWO", "kind": "weapon", "attack": 1, "max_health": 4, "damage": 1, "cost": 1},
                ],
            }
        }
        output = io.StringIO()
        with redirect_stdout(output):
            render_state(
                decision,
                {"builtin_hero": "英雄", "WEAPON_ONE": "战斧", "WEAPON_TWO": "小刀"},
            )
        rendered = output.getvalue()
        self.assertIn("你的武器：战斧[3] 费用3 攻击3 耐久1", rendered)
        self.assertIn("对手武器：小刀[4] 费用1 攻击1 耐久3", rendered)


if __name__ == "__main__":
    unittest.main()
