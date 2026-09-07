from __future__ import annotations

from copy import deepcopy
from dataclasses import replace
from pathlib import Path
import tempfile
import unittest

import torch

from hearth_env import HearthEnv
from hearth_env.training.catalog import CardCatalog
from hearth_env.training.checkpoint import load_checkpoint, save_checkpoint
from hearth_env.training.config import ModelConfig
from hearth_env.training.distribution import policy_logits
from hearth_env.training.health import EpisodeHealth, health_gate
from hearth_env.training.model import HearthQNetwork
from hearth_env.training.policies import ModelPolicy
from hearth_env.training.semantics import KEYWORD_INDEX
from hearth_env.training.tensorize import Tensorizer, collate

ROOT = Path(__file__).parents[2]


class SemanticTrainingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.deck = ["CS2_120"] * 30
        cls.env = HearthEnv(
            ROOT / "data",
            {
                "decks": [cls.deck, cls.deck],
                "classes": ["mage", "mage"],
                "unrestricted": True,
            },
            seed=3,
        )
        cls.catalog = CardCatalog.build(
            cls.env.card_catalog, cls.env.pack_hash, hash_dim=32
        )
        cls.config = ModelConfig(
            hidden_dim=32,
            text_dim=16,
            card_hash_dim=32,
            transformer_layers=1,
            dropout=0,
            max_history=16,
        )
        cls.tensorizer = Tensorizer(cls.catalog, cls.config)

    def decision(self) -> dict:
        return deepcopy(self.env.reset(seed=3))

    def test_public_facts_and_complete_history_survive_tensorization(self) -> None:
        original = self.decision()
        changed = deepcopy(original)
        changed["observation"]["self_player"].update(
            resources={"corpses": 10},
            resources_spent={"corpses": 20},
            public_statuses=["crystal_core"],
            history={"minions_died": ["CS2_120", "CS2_120"]},
        )
        a = self.tensorizer.encode(original, self.deck)
        b = self.tensorizer.encode(changed, self.deck)
        self.assertEqual(b["fact_mask"].sum().item(), 3)
        self.assertEqual(b["memory_counts"].sum().item(), 2)
        self.assertFalse(torch.equal(a["fact_values"], b["fact_values"]))
        self.assertFalse(torch.equal(a["memory_cards"], b["memory_cards"]))

    def test_acquired_taunt_and_poisonous_do_not_collide(self) -> None:
        a = self.decision()
        b = deepcopy(a)
        a["observation"]["entities"][0]["keywords"] = ["taunt"]
        b["observation"]["entities"][0]["keywords"] = ["poisonous"]
        x = self.tensorizer.encode(a, self.deck)["entity_keywords"][0]
        y = self.tensorizer.encode(b, self.deck)["entity_keywords"][0]
        self.assertNotEqual(KEYWORD_INDEX["taunt"], KEYWORD_INDEX["poisonous"])
        self.assertFalse(torch.equal(x, y))

    def test_conditional_imitation_ignores_unrelated_actions_and_validates_support(
        self,
    ):
        from hearth_env.training.learn import imitation_loss
        from hearth_env.training.trajectory import TrainingSample

        sample = TrainingSample(
            {
                "actions": [
                    {"kind": "end_turn"},
                    {"kind": "play_card"},
                    {"kind": "play_card"},
                ]
            },
            [],
            1,
            0,
            acceptable_actions=(1,),
            action_support=(1, 2),
        )
        logits = torch.tensor([[100.0, 0.0, 1.0]], requires_grad=True)
        losses, correct = imitation_loss(logits, [sample])
        torch.testing.assert_close(
            losses, torch.nn.functional.softplus(torch.tensor([1.0]))
        )
        self.assertFalse(correct.item())
        losses.sum().backward()
        self.assertEqual(logits.grad[0, 0].item(), 0.0)
        self.assertLess(logits.grad[0, 1].item(), 0.0)
        self.assertGreater(logits.grad[0, 2].item(), 0.0)
        changed = logits.detach().clone()
        changed[0, 0] = -100
        torch.testing.assert_close(imitation_loss(changed, [sample])[0], losses)
        with self.assertRaisesRegex(ValueError, "inside the conditional support"):
            imitation_loss(logits, [replace(sample, acceptable_actions=(0,))])
        with self.assertRaisesRegex(ValueError, "invalid action support"):
            imitation_loss(logits, [replace(sample, action_support=(1, 4))])

    def test_padding_trim_preserves_logits_and_value(self) -> None:
        sample = self.tensorizer.encode(self.decision(), self.deck)
        full = {key: value.unsqueeze(0) for key, value in sample.items()}
        full["action_mask"] = torch.ones(
            1, len(sample["action_kinds"]), dtype=torch.bool
        )
        trimmed = collate([sample])
        model = HearthQNetwork(self.catalog, self.config).eval()
        with torch.no_grad():
            full_logits, full_value = model.policy_value(full)
            logits, value = model.policy_value(trimmed)
        torch.testing.assert_close(logits, full_logits, atol=1e-6, rtol=1e-5)
        torch.testing.assert_close(value, full_value, atol=1e-6, rtol=1e-5)

    def test_history_role_binding_changes_state_encoding(self) -> None:
        a = self.decision()
        a["observation"]["history"]["events"] = [
            {
                "turn": 3,
                "cursor": 100,
                "event": {
                    "kind": "damaged",
                    "amount": 3,
                    "entities": [
                        {"role": "source", "card_id": "CS2_029"},
                        {"role": "target", "card_id": "CS2_120"},
                    ],
                },
            }
        ]
        b = deepcopy(a)
        b["observation"]["history"]["events"][0]["event"]["entities"][0]["role"] = (
            "target"
        )
        b["observation"]["history"]["events"][0]["event"]["entities"][1]["role"] = (
            "source"
        )
        model = HearthQNetwork(self.catalog, self.config).eval()
        with torch.no_grad():
            _, x = model._encode_state(collate([self.tensorizer.encode(a, self.deck)]))
            _, y = model._encode_state(collate([self.tensorizer.encode(b, self.deck)]))
        self.assertFalse(torch.allclose(x, y))

    def test_new_id_same_effect_has_identical_policy_and_survives_reload(self) -> None:
        extra = deepcopy(self.catalog.entries["CS2_120"])
        extra["definition"].update(id="UNSEEN_VANILLA", name="An unseen name")
        expanded = CardCatalog.build(
            [*self.catalog.entries.values(), extra], "expanded", hash_dim=32
        )
        model = HearthQNetwork(self.catalog, self.config).eval()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "semantic.pt"
            save_checkpoint(path, model, self.catalog)
            restored, _ = load_checkpoint(path, expanded)
            restored.eval()
            a = self.decision()
            b = deepcopy(a)
            for entity in b["observation"]["entities"]:
                if entity["card_id"] == "CS2_120":
                    entity["card_id"] = "UNSEEN_VANILLA"
            t = Tensorizer(expanded, restored.config)
            with torch.no_grad():
                x, vx = restored.policy_value(collate([t.encode(a, self.deck)]))
                y, vy = restored.policy_value(
                    collate([t.encode(b, ["UNSEEN_VANILLA"] * 30)])
                )
            torch.testing.assert_close(x, y, rtol=0, atol=0)
            torch.testing.assert_close(vx, vy, rtol=0, atol=0)
            self.assertFalse(restored.card_id_embedding.weight.requires_grad)

    def test_discover_is_permutation_equivariant_and_visible_to_value(self) -> None:
        a = self.decision()
        a["observation"]["phase"] = "choice"
        a["observation"]["pending_choice"] = {
            "prompt": "Discover",
            "options": [
                {"label": card, "value": {"kind": "card", "card_id": card}}
                for card in ("CS2_029", "CS2_120")
            ],
        }
        a["actions"] = [
            {"index": i, "kind": "choose", "choice_index": i} for i in range(2)
        ]
        b = deepcopy(a)
        b["observation"]["pending_choice"]["options"].reverse()
        c = deepcopy(a)
        c["observation"]["pending_choice"]["options"][1]["value"]["card_id"] = "CS2_024"
        model = HearthQNetwork(self.catalog, self.config).eval()
        torch.nn.init.normal_(model.value_head[-1].weight)
        with torch.no_grad():
            x, vx = model.policy_value(collate([self.tensorizer.encode(a, self.deck)]))
            y, vy = model.policy_value(collate([self.tensorizer.encode(b, self.deck)]))
            _, vz = model.policy_value(collate([self.tensorizer.encode(c, self.deck)]))
        torch.testing.assert_close(x, y.flip(1))
        torch.testing.assert_close(vx, vy)
        self.assertFalse(torch.allclose(vx, vz))

    def test_actor_log_probability_matches_learning_support(self) -> None:
        d = self.decision()
        d["actions"] = [
            {"index": 0, "kind": "end_turn"},
            {"index": 1, "kind": "concede"},
        ]
        model = HearthQNetwork(self.catalog, self.config)
        policy = ModelPolicy(model, self.tensorizer, device="cpu", sample=True)
        chosen = policy.choose(d, self.deck)
        with torch.no_grad():
            batch = collate([self.tensorizer.encode(d, self.deck)])
            logits = policy_logits(model(batch), batch)
        self.assertTrue((logits.softmax(1) > 0).all())
        self.assertAlmostEqual(
            policy.last_behavior["log_probability"],
            logits.log_softmax(1)[0, chosen].item(),
            places=5,
        )

    def test_information_overflow_is_explicit(self) -> None:
        t = Tensorizer(self.catalog, replace(self.config, max_entities=1))
        with self.assertRaisesRegex(ValueError, "capacity exceeded"):
            t.encode(self.decision(), self.deck)

    def test_resource_preserving_policy_is_not_rejected_by_league(self) -> None:
        self.assertEqual(
            health_gate(
                EpisodeHealth(episodes=10, end_turns=100, avoidable_end_turns=90)
            ),
            [],
        )


if __name__ == "__main__":
    unittest.main()
