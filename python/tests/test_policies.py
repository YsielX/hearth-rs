from copy import deepcopy
from pathlib import Path
import tempfile
import unittest

import torch

from hearth_env import HearthEnv
from hearth_env.training.bc import train_behavior_clone, evaluate_behavior_clone
from hearth_env.training.catalog import CardCatalog
from hearth_env.training.config import ModelConfig, TrainConfig
from hearth_env.training.distribution import policy_logits
from hearth_env.training.policies import HeuristicPolicy, RandomPolicy
from hearth_env.training.rollout import play_episode
from hearth_env.training.trajectory import (
    episode_samples,
    stream_samples,
    write_episodes,
)

ROOT = Path(__file__).parents[2]


class PolicyBoundaryTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        torch.set_num_threads(1)
        deck = ["CS2_120"] * 30
        cls.match = {
            "decks": [deck, deck],
            "classes": ["mage", "mage"],
            "unrestricted": True,
        }
        cls.env = HearthEnv(ROOT / "data", cls.match, max_steps=500, history_limit=8)
        cls.catalog = CardCatalog.build(
            cls.env.card_catalog, cls.env.pack_hash, hash_dim=32
        )

    def test_python_bot_uses_rust_and_rejects_stale_decisions(self):
        decision = self.env.reset(seed=20)
        policy = HeuristicPolicy()
        policy.env = self.env
        expected = self.env._native.heuristic_action(decision["id"])
        self.assertEqual(policy.choose(decision, []), expected)
        self.env.step(expected)
        with self.assertRaisesRegex(RuntimeError, "stale decision"):
            policy.choose(decision, [])
        episode = play_episode(
            self.env, [HeuristicPolicy(), HeuristicPolicy()], self.match, 21
        )
        self.assertNotIn("error", episode)
        self.assertTrue(episode["terminated"])
        self.assertTrue(all("behavior" not in s for s in episode["steps"]))

    def test_all_legal_actions_including_concede_can_be_learned(self):
        logits = torch.tensor([[1.0, 2.0, 3.0]], requires_grad=True)
        batch = {
            "action_mask": torch.tensor([[True, True, False]]),
            "action_kinds": torch.tensor([[1, 9, 1]]),
        }
        probabilities = policy_logits(logits, batch).softmax(1)
        self.assertGreater(probabilities[0, 1], probabilities[0, 0])
        self.assertEqual(probabilities[0, 2], 0)
        (-probabilities[0, 1].log()).backward()
        self.assertLess(logits.grad[0, 1], 0)
        d = {
            "actions": [
                {"index": 0, "kind": "end_turn"},
                {"index": 1, "kind": "concede"},
            ]
        }
        policy = RandomPolicy(20)
        self.assertEqual({policy.choose(d, []) for _ in range(40)}, {0, 1})
        decision = self.env.reset(seed=22)
        decision = deepcopy(decision)
        decision["actions"] = d["actions"]
        episode = {
            "match_config": self.match,
            "steps": [{"decision": decision, "action_index": 1}],
        }
        self.assertEqual(
            next(episode_samples(episode, behavior_clone=True)).action_index, 1
        )

    def test_ppo_uses_terminal_returns_and_rejects_teacher_or_stale_behavior(self):
        from hearth_env.training.model import HearthQNetwork
        from hearth_env.training.policies import ModelPolicy
        from hearth_env.training.ppo import build_ppo_experiences
        from hearth_env.training.tensorize import Tensorizer

        config = ModelConfig(
            hidden_dim=32,
            text_dim=16,
            card_hash_dim=32,
            transformer_layers=1,
            max_history=8,
            dropout=0,
        )
        model = HearthQNetwork(self.catalog, config)
        tensorizer = Tensorizer(self.catalog, config)
        policies = [
            ModelPolicy(model, tensorizer, device="cpu", sample=True, seed=i)
            for i in (31, 32)
        ]
        episode = play_episode(self.env, policies, self.match, 31)
        self.assertTrue(episode["terminated"])
        train = TrainConfig(gamma=1, gae_lambda=1, batch_size=128)
        experiences = build_ppo_experiences(
            [(episode, {0, 1})], model, tensorizer, train, device="cpu"
        )
        self.assertEqual(len(experiences), len(episode["steps"]))
        for experience in experiences:
            self.assertAlmostEqual(
                experience.return_value,
                episode["rewards"][experience.decision["actor_seat"]],
                places=5,
            )
        for tamper, message in (
            ("missing", "recorded behavior"),
            ("stale", "stale/off-policy"),
            ("support", "support/temperature"),
        ):
            bad = deepcopy(episode)
            behavior = bad["steps"][0]["behavior"]
            if tamper == "missing":
                bad["steps"][0].pop("behavior")
            elif tamper == "stale":
                behavior["log_probability"] += 1
            else:
                behavior["action_support"] = "teacher"
            with self.assertRaisesRegex(ValueError, message):
                build_ppo_experiences(
                    [(bad, {0, 1})], model, tensorizer, train, device="cpu"
                )

    def test_supervision_updates_and_validation_checks_versions(self):
        decision = self.env.reset(seed=25)
        episode = {
            "pack_hash": self.env.pack_hash,
            "engine_build": self.env.engine_build,
            "match_config": self.match,
            "steps": [
                {
                    "decision": decision,
                    "action_index": 0,
                    "acceptable_actions": [0],
                    "weight": 2.0,
                }
            ],
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "labels.jsonl.gz"
            write_episodes(path, [episode], append=False)
            model = train_behavior_clone(
                self.catalog,
                [path],
                Path(directory) / "student.pt",
                TrainConfig(device="cpu", bc_epochs=1, batch_size=1),
                model_config=ModelConfig(
                    hidden_dim=32,
                    text_dim=16,
                    card_hash_dim=32,
                    transformer_layers=1,
                    max_history=8,
                    dropout=0,
                ),
            )
            held = dict(episode, held_out_evaluation=True)
            write_episodes(path, [held], append=False)
            self.assertEqual(
                evaluate_behavior_clone(self.catalog, [path], model)["samples"], 1
            )
            with self.assertRaisesRegex(ValueError, "evaluation-only"):
                list(stream_samples([path], expected_pack_hash=self.env.pack_hash))
            write_episodes(path, [dict(held, engine_build="wrong")], append=False)
            with self.assertRaisesRegex(ValueError, "engine differs"):
                evaluate_behavior_clone(self.catalog, [path], model)
            write_episodes(
                path, [dict(episode, source="synthetic_public_probe")], append=False
            )
            with self.assertRaisesRegex(ValueError, "retired synthetic"):
                list(stream_samples([path], behavior_clone=True))
