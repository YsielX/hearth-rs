"""Decision exposure counts; visibility is not a claim of learned mastery."""

from collections import Counter


def decision_coverage(episodes):
    visible, offered, selected, classes = (Counter() for _ in range(4))
    decisions = count = 0
    for episode in episodes:
        count += 1
        seats = episode.get("training_seats", [0, 1])
        for step in episode.get("steps", []):
            decision = step["decision"]
            if decision["actor_seat"] not in seats:
                continue
            decisions += 1
            observation = decision["observation"]
            classes[observation["self_player"].get("class", "unknown")] += 1
            entities = {entity["entity"]: entity for entity in observation["entities"]}
            visible.update(
                {
                    entity["card_id"]
                    for entity in entities.values()
                    if entity.get("card_id")
                }
            )

            def action_cards(action):
                cards = set()
                if action.get("semantic_card_id"):
                    cards.add(action["semantic_card_id"])
                options = (observation.get("pending_choice") or {}).get("options", [])
                choice = action.get("choice_index")
                if choice is not None and 0 <= choice < len(options):
                    option = options[choice]
                    cards.update(option.get("semantic_card_ids", []))
                    value = option.get("value") or {}
                    if value.get("card_id"):
                        cards.add(value["card_id"])
                    if value.get("entity") in entities:
                        card = entities[value["entity"]].get("card_id")
                        if card:
                            cards.add(card)
                for ref in action.get("sources", []):
                    entity = entities.get(ref, {})
                    if entity.get("card_id"):
                        cards.add(entity["card_id"])
                    cards.update(entity.get("public_cards", []))
                return cards

            actions = decision["actions"]
            offered.update(set().union(*(action_cards(action) for action in actions)))
            selected.update(action_cards(actions[step["action_index"]]))
    return {
        "episodes": count,
        "decisions": decisions,
        "by_class": dict(sorted(classes.items())),
        "distinct_visible_cards": len(visible),
        "distinct_offered_cards": len(offered),
        "distinct_selected_cards": len(selected),
        "cards": {
            card: {
                "visible_decisions": visible[card],
                "offered_decisions": offered[card],
                "selected_decisions": selected[card],
            }
            for card in sorted(visible.keys() | offered.keys() | selected.keys())
        },
    }
