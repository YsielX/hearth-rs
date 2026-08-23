from __future__ import annotations

import json
import random
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path

import torch
from hearth_env import HearthEnv
from hearth_env.training.catalog import CardCatalog
from hearth_env.training.checkpoint import load_checkpoint, save_checkpoint
from hearth_env.training.config import ModelConfig, TrainConfig
from hearth_env.training.decks import SET_ORDER, Deck, DeckPool, match_config
from hearth_env.training.learn import train_batch
from hearth_env.training.model import HearthQNetwork
from hearth_env.training.policies import HeuristicPolicy
from hearth_env.training.ppo import build_ppo_experiences, train_ppo_epochs
from hearth_env.training.rollout import play_episode
from hearth_env.training.tensorize import Tensorizer, collate
from hearth_env.training.trajectory import TrainingSample, read_episodes, write_episodes

ROOT = Path(__file__).parents[2]


def demo_config() -> dict:
    cards = ["CS2_120"] * 30
    return {
        "decks": [cards, cards],
        "hero_powers": ["HERO_08bp", "HERO_08bp"],
        "classes": ["mage", "mage"],
        "unrestricted": True,
    }


class TrainingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.env = HearthEnv(ROOT / "data", demo_config(), seed=3, history_limit=16)
        cls.catalog = CardCatalog.build(
            cls.env.card_catalog, cls.env.pack_hash, hash_dim=32
        )
        cls.config = ModelConfig(
            hidden_dim=32,
            card_hash_dim=32,
            transformer_layers=1,
            attention_heads=4,
            max_history=16,
        )

    def test_iteration_rollout_shard_can_atomically_replace_stale_data(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            shard = Path(directory) / "iteration-000003.jsonl.gz"
            write_episodes(shard, [{"seed": 1}])
            write_episodes(shard, [{"seed": 2}])
            self.assertEqual(
                [episode["seed"] for episode in read_episodes([shard])],
                [1, 2],
            )

            write_episodes(shard, [{"seed": 3}], append=False)
            self.assertEqual(list(read_episodes([shard])), [{"seed": 3}])
            self.assertFalse((shard.parent / f".{shard.name}.tmp").exists())

    def test_catalog_combines_definition_and_lua(self) -> None:
        feature = self.catalog.feature("CS2_120")
        self.assertEqual(len(feature), self.catalog.feature_dim)
        self.assertTrue(any(value != 0 for value in feature))
        self.assertIn("lua_source", self.catalog.entries["CS2_120"])

    def test_model_scores_every_legal_action_and_updates(self) -> None:
        decision = self.env.reset(seed=8)
        tensorizer = Tensorizer(self.catalog, self.config)
        encoded = tensorizer.encode(decision, demo_config()["decks"][0])
        batch = collate([encoded])
        model = HearthQNetwork(self.catalog, self.config)
        values = model(batch)
        self.assertEqual(tuple(values.shape), (1, len(decision["actions"])))
        sample = TrainingSample(decision, demo_config()["decks"][0], 0, 1.0)
        optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3)
        metrics = train_batch(
            model,
            optimizer,
            tensorizer,
            [sample],
            TrainConfig(batch_size=1, amp=False),
            "cpu",
            behavior_clone=False,
        )
        self.assertGreaterEqual(metrics.loss, 0.0)

        logits, value = model.policy_value(batch)
        self.assertEqual(tuple(logits.shape), tuple(values.shape))
        self.assertEqual(tuple(value.shape), (1,))
        self.assertTrue(torch.all(value >= -1.0))
        self.assertTrue(torch.all(value <= 1.0))

    def test_ppo_builds_terminal_returns_and_updates(self) -> None:
        episode = play_episode(
            self.env,
            [HeuristicPolicy(21), HeuristicPolicy(22)],
            demo_config(),
            21,
        )
        self.assertTrue(episode["terminated"])
        model = HearthQNetwork(self.catalog, self.config)
        tensorizer = Tensorizer(self.catalog, self.config)
        config = TrainConfig(
            batch_size=64,
            amp=False,
            ppo_epochs=1,
            shaping_coefficient=0.05,
        )
        experiences = build_ppo_experiences(
            [(episode, {0, 1})], model, tensorizer, config, device="cpu"
        )
        self.assertGreater(len(experiences), 0)
        self.assertTrue(any(item.return_value > 0 for item in experiences))
        self.assertTrue(any(item.return_value < 0 for item in experiences))
        optimizer = torch.optim.AdamW(model.parameters(), lr=1e-4)
        metrics = train_ppo_epochs(
            model,
            optimizer,
            tensorizer,
            experiences,
            config,
            device="cpu",
            rng=random.Random(1),
            reference_model=deepcopy(model),
        )
        self.assertGreater(metrics.updates, 0)
        self.assertTrue(torch.isfinite(torch.tensor(metrics.loss)))
        self.assertGreaterEqual(metrics.reference_kl, 0.0)

    def test_checkpoint_expands_for_new_cards(self) -> None:
        model = HearthQNetwork(self.catalog, self.config)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "model.pt"
            save_checkpoint(path, model, self.catalog, phase="test")
            entries = list(self.env.card_catalog)
            extra = deepcopy(entries[0])
            extra["definition"]["id"] = "TEST_NEW_CARD"
            entries.append(extra)
            expanded = CardCatalog.build(
                entries,
                self.env.pack_hash + ":new",
                hash_dim=32,
                prior_card_ids=self.catalog.card_ids,
            )
            restored, payload = load_checkpoint(path, expanded)
            self.assertEqual(
                restored.card_id_embedding.num_embeddings, len(expanded.card_ids)
            )
            self.assertEqual(payload["migration"]["new_cards"], 1)
            new_row = restored.card_id_embedding.weight[expanded.index("TEST_NEW_CARD")]
            self.assertEqual(int(torch.count_nonzero(new_row).item()), 0)

    def test_frozen_throne_deck_corpus_is_runnable(self) -> None:
        paths = sorted((ROOT / "decks/frozen_throne").glob("*.json"))
        self.assertEqual(len(paths), 354)
        decks = [Deck.from_file(path) for path in paths]
        self.assertEqual({deck.card_class for deck in decks}, set(DEFAULT_CLASSES))
        self.assertGreaterEqual(sum(deck.bc_eligible for deck in decks), 25)
        env = HearthEnv(
            ROOT / "data", match_config(decks[0], decks[0]), seed=1, history_limit=16
        )

        for path, deck in zip(paths, decks, strict=True):
            self.assertEqual(len(deck.cards), 30, path.name)
            value = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(
                sum(card["count"] for card in value["source_cards"]), 30, path.name
            )
            decision = env.reset(
                seed=1,
                match_config=match_config(deck, deck),
            )
            self.assertTrue(decision["actions"], path.name)

    def test_frozen_throne_pool_does_not_sample_future_cards(self) -> None:
        decks = [
            Deck.from_file(path)
            for path in sorted((ROOT / "decks/frozen_throne").glob("*.json"))
        ]
        pool = DeckPool(
            self.catalog,
            decks,
            seed=9,
            curated_probability=0.0,
            perturb_probability=0.0,
        )
        allowed = set(SET_ORDER[: SET_ORDER.index("ICECROWN") + 1])
        for _ in range(100):
            deck = pool.sample()
            self.assertIn(deck.card_class, DEFAULT_CLASSES)
            self.assertTrue(
                all(
                    self.catalog.entries[card_id]["definition"]["set"] in allowed
                    for card_id in deck.cards
                )
            )

        mixed_pool = DeckPool(self.catalog, decks, seed=10)
        env = HearthEnv(
            ROOT / "data", match_config(decks[0], decks[1]), seed=1, history_limit=8
        )
        for seed in range(30):
            env.reset(seed=seed, match_config=mixed_pool.sample_match())


DEFAULT_CLASSES = (
    "druid",
    "hunter",
    "mage",
    "paladin",
    "priest",
    "rogue",
    "shaman",
    "warlock",
    "warrior",
)


if __name__ == "__main__":
    unittest.main()
