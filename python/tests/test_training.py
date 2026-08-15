from __future__ import annotations

import tempfile
import unittest
from copy import deepcopy
from pathlib import Path

import torch
from hearth_env import HearthEnv
from hearth_env.training.catalog import CardCatalog
from hearth_env.training.checkpoint import load_checkpoint, save_checkpoint
from hearth_env.training.config import ModelConfig, TrainConfig
from hearth_env.training.learn import train_batch
from hearth_env.training.model import HearthQNetwork
from hearth_env.training.tensorize import Tensorizer, collate
from hearth_env.training.trajectory import TrainingSample

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


if __name__ == "__main__":
    unittest.main()
