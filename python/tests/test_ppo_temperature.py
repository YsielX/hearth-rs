"""Tempered actors must remain exactly on-policy at update and resume."""

from copy import deepcopy
from dataclasses import replace
from pathlib import Path
import random
import tempfile
import unittest

import torch

from hearth_env import HearthEnv
from hearth_env.training.catalog import CardCatalog
from hearth_env.training.config import ModelConfig, TrainConfig
from hearth_env.training.decks import Deck, DeckPool, match_config
from hearth_env.training.distribution import policy_logits
from hearth_env.training.model import HearthQNetwork
from hearth_env.training.policies import ModelPolicy
from hearth_env.training.ppo import build_ppo_experiences, train_ppo, train_ppo_epochs
from hearth_env.training.rollout import play_episode
from hearth_env.training.tensorize import Tensorizer, collate
from hearth_env.training.trajectory import read_episodes

ROOT = Path(__file__).parents[2]


class PPOTemperatureTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.threads = torch.get_num_threads()
        torch.set_num_threads(1)
        cls.deck = Deck(
            "test", "mage", ("CS2_120",) * 30, "HERO_08bp", unrestricted=True
        )
        cls.match = match_config(cls.deck, cls.deck)
        cls.env = HearthEnv(ROOT / "data", cls.match, history_limit=8)
        cls.catalog = CardCatalog.build(
            cls.env.card_catalog, cls.env.pack_hash, hash_dim=32
        )
        cls.model_config = ModelConfig(
            architecture_version=3,
            hidden_dim=32,
            text_dim=16,
            card_hash_dim=32,
            transformer_layers=1,
            max_history=8,
            dropout=0,
        )

    @classmethod
    def tearDownClass(cls):
        torch.set_num_threads(cls.threads)

    def test_temperature_changes_exploration_without_changing_support_or_greedy(self):
        logits = torch.tensor([[0.0, 8.0, 20.0, 30.0]])
        batch = {
            "action_mask": torch.tensor([[True, True, True, False]]),
            "action_kinds": torch.tensor([[1, 1, 9, 1]]),
        }
        cold = policy_logits(logits, batch).softmax(-1)
        warm = policy_logits(logits, batch, temperature=3).softmax(-1)
        self.assertEqual(warm.argmax().item(), cold.argmax().item())
        self.assertGreater(warm[0, 0].item(), 100 * cold[0, 0].item())
        self.assertGreater(warm[0, 2].item(), 0)
        self.assertEqual(warm[0, 3].item(), 0)
        for value in (0.0, -1.0, float("inf"), float("-inf"), float("nan")):
            with self.assertRaisesRegex(ValueError, "positive and finite"):
                TrainConfig(ppo_temperature=value)
            with self.assertRaisesRegex(ValueError, "positive and finite"):
                policy_logits(logits, batch, temperature=value)

    def test_native_behavior_ratio_entropy_and_reference_share_temperature(self):
        torch.manual_seed(42)
        model = HearthQNetwork(self.catalog, self.model_config)
        tensorizer = Tensorizer(self.catalog, self.model_config)
        temperature = 2.5
        episode = play_episode(
            self.env,
            [
                ModelPolicy(
                    model,
                    tensorizer,
                    device="cpu",
                    seed=i,
                    sample=True,
                    temperature=temperature,
                )
                for i in (71, 72)
            ],
            self.match,
            71,
        )
        self.assertTrue(episode["terminated"])
        config = TrainConfig(
            ppo_temperature=temperature,
            batch_size=1024,
            ppo_epochs=1,
            reference_kl_coefficient=0.2,
            entropy_coefficient=0.1,
        )
        experiences = build_ppo_experiences(
            [(episode, {0, 1})], model, tensorizer, config, device="cpu"
        )
        self.assertGreater(len(experiences), 1)
        with self.assertRaisesRegex(ValueError, "temperature differs"):
            build_ppo_experiences(
                [(episode, {0, 1})],
                model,
                tensorizer,
                replace(config, ppo_temperature=1),
                device="cpu",
            )
        corrupted = deepcopy(episode)
        for step in corrupted["steps"]:
            step["behavior"]["temperature"] = 1.0
        with self.assertRaisesRegex(ValueError, "stale/off-policy"):
            build_ppo_experiences(
                [(corrupted, {0, 1})],
                model,
                tensorizer,
                replace(config, ppo_temperature=1),
                device="cpu",
            )
        batch = collate(
            [tensorizer.encode(x.decision, x.self_deck) for x in experiences]
        )
        with torch.no_grad():
            logits = policy_logits(model(batch), batch, temperature=temperature)
            entropy = (
                -(logits.softmax(-1) * logits.log_softmax(-1)).sum(-1).mean().item()
            )
        before = {k: p.clone() for k, p in model.named_parameters()}
        metrics = train_ppo_epochs(
            model,
            torch.optim.SGD(model.parameters(), lr=0),
            tensorizer,
            experiences,
            config,
            device="cpu",
            rng=random.Random(3),
            reference_model=deepcopy(model),
        )
        self.assertEqual(metrics.updates, 1)
        self.assertAlmostEqual(metrics.approximate_kl, 0.0, places=6)
        self.assertAlmostEqual(metrics.reference_kl, 0.0, places=6)
        self.assertAlmostEqual(metrics.entropy, entropy, places=6)
        self.assertEqual(metrics.clip_fraction, 0.0)
        for name, parameter in model.named_parameters():
            torch.testing.assert_close(parameter, before[name], rtol=0, atol=0)

    def test_train_resume_temperature_metadata_and_legacy_default(self):
        config = TrainConfig(
            device="cpu",
            seed=202,
            batch_size=64,
            ppo_iterations=1,
            episodes_per_iteration=2,
            ppo_epochs=1,
            checkpoint_every=1,
            league_snapshot_every=1,
            ppo_temperature=2.5,
            history_limit=8,
        )

        def pool():
            return DeckPool(
                self.catalog,
                [self.deck],
                seed=config.seed,
                curated_probability=1,
                perturb_probability=0,
            )

        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            kwargs = dict(
                data_path=ROOT / "data",
                catalog=self.catalog,
                run_dir=run,
                model_config=self.model_config,
                specialist_probability=1.0,
            )
            train_ppo(deck_pool=pool(), train_config=config, **kwargs)
            checkpoint = run / "latest.pt"
            payload = torch.load(checkpoint, map_location="cpu", weights_only=False)
            self.assertEqual(
                payload["run_manifest"]["train_config"]["ppo_temperature"], 2.5
            )
            self.assertEqual(payload["run_manifest"]["format_version"], 4)
            # Same cards and RNG are insufficient when perturbation protection,
            # era filtering or native deck legality changes on resume.
            for changes in (
                {"protected_cards": self.deck.cards},
                {"era_cutoff": "BASIC"},
                {"unrestricted": not self.deck.unrestricted},
            ):
                changed_pool = DeckPool(
                    self.catalog,
                    [replace(self.deck, **changes)],
                    seed=config.seed,
                    curated_probability=1,
                    perturb_probability=0,
                )
                with (
                    self.subTest(changes=changes),
                    self.assertRaisesRegex(
                        ValueError, "resume configuration differs: decks"
                    ),
                ):
                    train_ppo(
                        deck_pool=changed_pool,
                        train_config=replace(config, ppo_iterations=2),
                        resume_checkpoint=checkpoint,
                        **kwargs,
                    )
            missing_metadata = deepcopy(payload)
            missing_metadata["run_manifest"]["format_version"] = 2
            missing_metadata["run_manifest"]["decks"] = [
                {k: d[k] for k in ("name", "card_class", "cards", "hero_power")}
                for d in missing_metadata["run_manifest"]["decks"]
            ]
            missing_path = run / "missing-deck-metadata.pt"
            torch.save(missing_metadata, missing_path)
            with self.assertRaisesRegex(ValueError, "lacks complete deck metadata"):
                train_ppo(
                    deck_pool=pool(),
                    train_config=replace(config, ppo_iterations=2),
                    resume_checkpoint=missing_path,
                    **kwargs,
                )
            for episode in read_episodes([run / "rollouts/iteration-000000.jsonl.gz"]):
                for step in episode["steps"]:
                    if step["decision"]["actor_seat"] in episode["training_seats"]:
                        self.assertEqual(step["behavior"]["temperature"], 2.5)
            with self.assertRaisesRegex(
                ValueError, "resume training setting differs: ppo_temperature"
            ):
                train_ppo(
                    deck_pool=pool(),
                    train_config=replace(config, ppo_iterations=2, ppo_temperature=3),
                    resume_checkpoint=checkpoint,
                    **kwargs,
                )
            legacy = deepcopy(payload)
            legacy["run_manifest"]["train_config"].pop("ppo_temperature")
            legacy_path = run / "legacy.pt"
            torch.save(legacy, legacy_path)
            with self.assertRaisesRegex(
                ValueError, "resume training setting differs: ppo_temperature"
            ):
                train_ppo(
                    deck_pool=pool(),
                    train_config=replace(config, ppo_iterations=2),
                    resume_checkpoint=legacy_path,
                    **kwargs,
                )
            # A genuine saved T=2.5 run resumes with identical temperature.
            train_ppo(
                deck_pool=pool(),
                train_config=replace(config, ppo_iterations=2, workers=2),
                resume_checkpoint=checkpoint,
                **kwargs,
            )
            resumed = torch.load(checkpoint, map_location="cpu", weights_only=False)
            self.assertEqual(resumed["ppo_iteration"], 2)
            for episode in read_episodes([run / "rollouts/iteration-000001.jsonl.gz"]):
                for step in episode["steps"]:
                    if step["decision"]["actor_seat"] in episode["training_seats"]:
                        self.assertEqual(step["behavior"]["temperature"], 2.5)

    def test_old_default_checkpoint_resumes_at_default_temperature(self):
        config = TrainConfig(
            device="cpu",
            seed=209,
            batch_size=64,
            ppo_iterations=1,
            episodes_per_iteration=1,
            ppo_epochs=1,
            checkpoint_every=1,
            league_snapshot_every=1,
            history_limit=8,
        )
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            kwargs = dict(
                data_path=ROOT / "data",
                catalog=self.catalog,
                run_dir=run,
                model_config=self.model_config,
                specialist_probability=1.0,
            )
            pool = lambda: DeckPool(
                self.catalog,
                [self.deck],
                seed=config.seed,
                curated_probability=1,
                perturb_probability=0,
            )
            train_ppo(deck_pool=pool(), train_config=config, **kwargs)
            checkpoint = run / "latest.pt"
            payload = torch.load(checkpoint, map_location="cpu", weights_only=False)
            payload["run_manifest"]["train_config"].pop("ppo_temperature")
            legacy = run / "legacy.pt"
            torch.save(payload, legacy)
            train_ppo(
                deck_pool=pool(),
                train_config=replace(config, ppo_iterations=2),
                resume_checkpoint=legacy,
                **kwargs,
            )
            resumed = torch.load(checkpoint, map_location="cpu", weights_only=False)
            self.assertEqual(resumed["ppo_iteration"], 2)
            self.assertEqual(
                resumed["run_manifest"]["train_config"]["ppo_temperature"], 1.0
            )


if __name__ == "__main__":
    unittest.main()
