from __future__ import annotations

from collections import defaultdict
from typing import Any


def attack_target_diagnostic(
    episodes: list[tuple[dict[str, Any], set[int]]],
) -> dict[str, Any]:
    totals: defaultdict[str, int] = defaultdict(int)
    by_class: dict[str, defaultdict[str, int]] = {}

    def record(bucket: defaultdict[str, int], key: str) -> None:
        bucket[key] += 1
        totals[key] += 1

    for episode, controlled_seats in episodes:
        if episode.get("truncated") or episode.get("error"):
            totals["skipped_episodes"] += 1
            continue
        classes = episode["match_config"]["classes"]
        for step in episode.get("steps", []):
            decision = step["decision"]
            seat = int(decision["actor_seat"])
            if seat not in controlled_seats:
                continue
            actions = decision.get("actions", [])
            action_index = int(step["action_index"])
            if action_index < 0 or action_index >= len(actions):
                totals["invalid_actions"] += 1
                continue
            chosen = actions[action_index]
            if chosen.get("kind") != "attack":
                continue
            class_bucket = by_class.setdefault(classes[seat], defaultdict(int))
            record(class_bucket, "attack_choices")
            entities = {
                int(entity["entity"]): entity
                for entity in decision["observation"].get("entities", [])
            }
            target = entities.get(int(chosen.get("target", -1)), {})
            sources = tuple(chosen.get("sources", []))
            source = entities.get(int(sources[0]), {}) if sources else {}
            source_attack = float(source.get("attack", 0))
            if target.get("area") != "hero":
                record(class_bucket, "board_attacks")
                continue
            record(class_bucket, "face_attacks")

            hero_health = max(
                float(target.get("max_health", 0))
                - float(target.get("damage", 0))
                + float(target.get("armor", 0)),
                0.0,
            )
            lethal = source_attack >= hero_health
            killable_minion = False
            board_option = False
            for alternative in actions:
                if alternative.get("kind") != "attack":
                    continue
                if tuple(alternative.get("sources", [])) != sources:
                    continue
                alternative_target = entities.get(
                    int(alternative.get("target", -1)), {}
                )
                if alternative_target.get("area") != "board":
                    continue
                board_option = True
                remaining = max(
                    float(alternative_target.get("max_health", 0))
                    - float(alternative_target.get("damage", 0)),
                    0.0,
                )
                killable_minion |= source_attack >= remaining
            if board_option:
                record(class_bucket, "face_with_board_option")
            if killable_minion:
                record(class_bucket, "face_with_killable_minion")
            if lethal and board_option:
                record(class_bucket, "face_lethal_with_board_option")
            if not lethal and killable_minion:
                record(class_bucket, "nonlethal_face_with_killable_minion")

    attacks = totals["attack_choices"]
    face = totals["face_attacks"]
    result: dict[str, Any] = dict(totals)
    result.update(
        {
            "face_attack_rate": face / max(attacks, 1),
            "board_attack_rate": totals["board_attacks"] / max(attacks, 1),
            "nonlethal_face_with_killable_minion_rate": totals[
                "nonlethal_face_with_killable_minion"
            ]
            / max(attacks, 1),
            "by_class": {
                card_class: {
                    **dict(bucket),
                    "face_attack_rate": bucket["face_attacks"]
                    / max(bucket["attack_choices"], 1),
                }
                for card_class, bucket in sorted(by_class.items())
            },
        }
    )
    return result
