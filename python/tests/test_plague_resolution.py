"""Native integration regressions for replacement draws and unending Plagues."""

import os
from pathlib import Path
import shutil
import tempfile
import unittest

from hearth_env import HearthEnv

ROOT = Path(__file__).parents[2]
FIXTURE = """
return {
 api_version=1,id="TEST_PLAGUE_SETUP",name="Plague test setup",type="spell",cost=0,class="mage",
 on_play=function(ctx,self)
  local player=ctx:controller(self)
  for _,entity in ipairs(ctx:deck(player)) do ctx:move(entity,"removed") end
  cardlib.effects.grant_keyword(ctx,ctx:player(player).hero,"immune")
  ctx:grant_player_keyword(player,"unending_plagues")
  for _,id in ipairs(cardlib.plagues.ids) do cardlib.plagues.shuffle(ctx,ctx:opponent(player),player,id) end
  cardlib.effects.give_card(ctx,player,"TTN_450t")
  ctx:gain_temporary_mana(player,10)
 end,
}
"""


class PlagueResolutionTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.data = Path(cls.temporary.name) / "data"
        shutil.copytree(os.environ.get("HEARTH_TEST_DATA", ROOT / "data"), cls.data)
        # The test-only card sets up a reachable public rule interaction in one play.
        (cls.data / "sets/titans/diagnostic_fixture.lua").write_text(FIXTURE)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def setUp(self):
        deck = ["TEST_PLAGUE_SETUP", "CS2_023"] * 15
        self.env = HearthEnv(self.data, {"decks": [deck, deck], "unrestricted": True}, seed=7)
        for _ in range(2):
            decision = self.env.decision
            action = next(a for a in decision["actions"] if a["kind"] == "mulligan" and not a["sources"])
            self.env.step(action["index"])
        self.play("TEST_PLAGUE_SETUP")

    def play(self, card):
        decision = self.env.decision
        entities = {e["entity"]: e for e in decision["observation"]["entities"]}
        action = next(a for a in decision["actions"] if a["kind"] == "play_card" and any(
            entities[source]["card_id"] == card for source in a["sources"]
        ))
        return self.env.step(action["index"])

    def test_unending_replacement_draws_terminate_and_reshuffle_between_main_draws(self):
        result = self.play("CS2_023")
        self.assertFalse(result["terminated"])
        self.assertFalse(result["truncated"])
        obs = result["next"]["observation"]
        self.assertEqual(obs["self_player"]["deck_size"], 3)
        # Each of the two main draws consumes the three existing Plagues, reaches
        # fatigue once, and only then returns those three Plagues to the deck.
        self.assertEqual(obs["self_player"]["fatigue"], 2)
        summoned = [e for e in obs["entities"] if e["area"] == "board" and e["controller"] == "opponent"]
        self.assertEqual([e["card_id"] for e in summoned], ["RLK_070t", "RLK_070t"])
        hero = next(e for e in obs["entities"] if e["area"] == "hero" and e["controller"] == "self_player")
        self.assertEqual(hero["damage"], 0)

    def test_manual_plague_play_does_not_trigger_unending_draw_rule(self):
        result = self.play("TTN_450t")
        obs = result["next"]["observation"]
        self.assertEqual(obs["self_player"]["deck_size"], 3)
        self.assertEqual(obs["self_player"]["fatigue"], 0)
