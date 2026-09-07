from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import tempfile
import threading
import unittest
from unittest.mock import patch

from hearth_env import HearthEnv
from hearth_env.training.catalog import CardCatalog
from hearth_env.training.bc import mix_replay
from hearth_env.training.decks import Deck, DeckPool
from hearth_env.training.evaluate import paired_evaluate
from hearth_env.training.manifests import deck_fingerprint, load_split_paths
from hearth_env.training.league import CheckpointLeague
from hearth_env.training.rollout import ParallelCollector, RolloutJob

ROOT = Path(__file__).parents[2]


class PipelineTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cards = ["CS2_120"] * 30
        cls.env = HearthEnv(
            ROOT / "data", {"decks": [cards, cards], "unrestricted": True}
        )
        cls.catalog = CardCatalog.build(cls.env.card_catalog, cls.env.pack_hash)

    def test_replay_fraction_preserves_primary_and_cycles_auxiliary(self):
        result = list(mix_replay(range(6), lambda: iter(["aux"]), 0.25))
        self.assertEqual([item for item in result if item != "aux"], list(range(6)))
        self.assertEqual(result.count("aux"), 2)
        with self.assertRaisesRegex(ValueError, "no decisions"):
            list(mix_replay([1, 2], lambda: iter([]), 0.5))

    def test_manifest_detects_changed_deck(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "deck.json"
            deck = {"class": "mage", "cards": ["CS2_120"] * 30, "unrestricted": True}
            path.write_text(json.dumps(deck))
            manifest = Path(directory) / "split.json"
            manifest.write_text(
                json.dumps(
                    {
                        "splits": {
                            "train": [
                                {
                                    "path": "deck.json",
                                    "fingerprint": deck_fingerprint(
                                        Deck.from_file(path)
                                    ),
                                }
                            ]
                        }
                    }
                )
            )
            self.assertEqual(load_split_paths(manifest, "train"), [str(path)])
            deck["cards"][0] = "CS2_029"
            path.write_text(json.dumps(deck))
            with self.assertRaisesRegex(ValueError, "changed since"):
                load_split_paths(manifest, "train")

    def test_resume_league_ignores_future_snapshots_and_rejects_changed_opponents(self):
        with tempfile.TemporaryDirectory() as directory:
            first = Path(directory) / "snapshot-000001.pt"
            first.write_bytes(b"frozen opponent")
            league = CheckpointLeague(directory)
            state = league.state()
            (Path(directory) / "snapshot-999999.pt").write_bytes(b"future opponent")
            league.freeze(state)
            self.assertEqual(league.snapshots(), [first])
            first.write_bytes(b"changed opponent")
            with self.assertRaisesRegex(ValueError, "missing or changed"):
                league.freeze(state)

    def test_collector_submits_after_fast_completion_while_first_job_waits(self):
        third_started = threading.Event()

        def play(job):
            if job.seed == 0:
                self.assertTrue(third_started.wait(2), "head-of-line blocking")
            elif job.seed == 2:
                third_started.set()
            return {"seed": job.seed}

        collector = ParallelCollector.__new__(ParallelCollector)
        collector.workers = 1
        collector.executor = ThreadPoolExecutor(max_workers=2)
        collector.failures, collector.failure_dir, collector.max_failures = 0, None, 0
        jobs = [RolloutJob({}, i, ({}, {})) for i in range(3)]
        try:
            with patch("hearth_env.training.rollout._worker_play", side_effect=play):
                self.assertEqual(
                    [e["seed"] for e in collector.collect(jobs)], [0, 1, 2]
                )
        finally:
            collector.close()

    def test_evaluation_covers_both_decks_in_both_seats(self):
        seen = []
        config = {
            "decks": [["A"], ["B"]],
            "classes": ["mage", "hunter"],
            "hero_powers": ["M", "H"],
        }

        def play(env, policies, match, seed):
            seat = policies.index("candidate")
            seen.append((seat, match["decks"][seat][0]))
            return {
                "steps": [],
                "terminated": True,
                "truncated": False,
                "rewards": [1.0, -1.0],
            }

        with patch("hearth_env.training.evaluate.play_episode", side_effect=play):
            result = paired_evaluate(
                None, lambda seed: "candidate", lambda seed: "opponent", [config]
            )
        self.assertEqual(set(seen), {(0, "A"), (0, "B"), (1, "A"), (1, "B")})
        self.assertEqual(result.games, 4)
        self.assertEqual(result.paired_scores, [0.5])

    def test_mirror_evaluation_uses_two_seats_without_duplicate_games(self):
        config = {"decks": [["A"], ["A"]], "classes": ["mage", "mage"]}
        episode = {
            "steps": [],
            "terminated": True,
            "truncated": False,
            "rewards": [1.0, -1.0],
        }
        with patch(
            "hearth_env.training.evaluate.play_episode", return_value=episode
        ) as play:
            result = paired_evaluate(
                None,
                lambda seed: "candidate",
                lambda seed: "opponent",
                [config],
                swap_decks=False,
            )
        self.assertEqual(play.call_count, 2)
        self.assertEqual(result.games, 2)
        self.assertEqual(result.by_seat[0].wins, 1)
        self.assertEqual(result.by_seat[1].losses, 1)
        self.assertEqual(result.paired_scores, [0.5])
        supplied = paired_evaluate(
            None,
            lambda seed: "candidate",
            lambda seed: "opponent",
            [config],
            swap_decks=False,
            episodes=[episode, episode],
        )
        self.assertEqual(supplied.summary(), result.summary())
        with self.assertRaisesRegex(ValueError, "exactly 2"):
            paired_evaluate(
                None, None, None, [config], swap_decks=False, episodes=[episode] * 4
            )

    def test_all_card_pool_includes_new_classes_and_respects_runes(self):
        paths = sorted((ROOT / "decks/frozen_throne").glob("*.json"))
        pool = DeckPool(
            self.catalog, [Deck.from_file(paths[0])], card_pool="all", seed=123
        )
        self.assertIn("death_knight", pool._pools)
        self.assertIn("demon_hunter", pool._pools)
        for _ in range(30):
            deck = pool._random()
            self.assertEqual(len(deck.cards), 30)
            self.assertTrue(pool._runes_fit(deck.cards))
            self.env.reset(
                seed=1,
                match_config={
                    "decks": [list(deck.cards)] * 2,
                    "classes": [deck.card_class] * 2,
                    "hero_powers": [deck.hero_power] * 2,
                },
            )


if __name__ == "__main__":
    unittest.main()
