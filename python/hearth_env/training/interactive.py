from __future__ import annotations

import re
from collections.abc import Mapping, Sequence
from typing import Any

from hearth_env import HearthEnv

from .model import HearthQNetwork
from .policies import ModelPolicy
from .tensorize import Tensorizer

_HTML_TAG = re.compile(r"<[^>]+>")
_CLASS_NAMES = {
    "druid": "德鲁伊",
    "hunter": "猎人",
    "mage": "法师",
    "paladin": "圣骑士",
    "priest": "牧师",
    "rogue": "潜行者",
    "shaman": "萨满祭司",
    "warlock": "术士",
    "warrior": "战士",
}
_ACTION_NAMES = {
    "attack": "攻击",
    "choose": "选择",
    "concede": "认输",
    "end_turn": "结束回合",
    "mulligan": "替换起手牌",
    "play_card": "使用卡牌",
    "play_card_at": "召唤随从",
    "trade_card": "交易卡牌",
    "use_card_action": "使用卡牌动作",
    "use_hero_power": "使用英雄技能",
    "use_location": "使用地标",
}


def card_names(
    raw_catalog: Sequence[Mapping[str, Any]], locale: str
) -> dict[str, str]:
    result: dict[str, str] = {}
    for entry in raw_catalog:
        definition = entry["definition"]
        localized = definition.get("localizations", {}).get(locale, {})
        result[str(definition["id"])] = str(
            localized.get("name") or definition.get("name") or definition["id"]
        )
    result["builtin_hero"] = "英雄"
    return result


def _entity_label(
    reference: int | None,
    entities: Mapping[int, Mapping[str, Any]],
    names: Mapping[str, str],
    *,
    detailed: bool = False,
) -> str:
    if reference is None:
        return "无目标"
    entity = entities.get(reference)
    if entity is None:
        return f"实体#{reference}"
    card_id = str(entity.get("card_id", "?"))
    name = names.get(card_id, card_id)
    if not detailed:
        return f"{name}[{reference}]"
    kind = entity.get("kind")
    cost = int(entity.get("cost", 0))
    if kind in {"minion", "hero"}:
        attack = int(entity.get("attack", 0))
        health = int(entity.get("max_health", 0)) - int(entity.get("damage", 0))
        armor = int(entity.get("armor", 0))
        suffix = f" {attack}/{health}"
        if armor:
            suffix += f" 护甲{armor}"
        return f"{name}[{reference}] 费用{cost}{suffix}"
    if kind == "weapon":
        attack = int(entity.get("attack", 0))
        durability = int(entity.get("max_health", 0)) - int(entity.get("damage", 0))
        return f"{name}[{reference}] 费用{cost} 攻击{attack} 耐久{durability}"
    return f"{name}[{reference}] 费用{cost}"


def describe_action(
    decision: Mapping[str, Any],
    action: Mapping[str, Any],
    names: Mapping[str, str],
) -> str:
    entities = {
        int(entity["entity"]): entity
        for entity in decision["observation"].get("entities", [])
    }
    kind = str(action.get("kind", "?"))
    sources = [
        _entity_label(int(reference), entities, names)
        for reference in action.get("sources", [])
    ]
    target = action.get("target")
    target_label = (
        _entity_label(int(target), entities, names) if target is not None else None
    )
    if kind == "mulligan":
        return "保留全部起手牌" if not sources else "替换 " + "、".join(sources)
    if kind == "attack" and sources:
        return f"{sources[0]} → 攻击 {target_label}"
    if kind in {"play_card", "play_card_at"} and sources:
        text = f"{_ACTION_NAMES[kind]} {sources[0]}"
        if action.get("board_position") is not None:
            text += f"，位置 {int(action['board_position']) + 1}"
        if target_label:
            text += f" → {target_label}"
        return text
    if kind in {"use_hero_power", "use_location", "use_card_action"}:
        text = _ACTION_NAMES[kind]
        if sources:
            text += " " + "、".join(sources)
        if action.get("card_action"):
            text += f" ({action['card_action']})"
        if target_label:
            text += f" → {target_label}"
        return text
    if kind == "choose":
        choice_index = action.get("choice_index")
        pending = decision["observation"].get("pending_choice") or {}
        options = pending.get("options", [])
        if isinstance(choice_index, int) and 0 <= choice_index < len(options):
            option = options[choice_index]
            value = option.get("value") or {}
            card_id = value.get("card_id")
            label = names.get(str(card_id), str(card_id)) if card_id else None
            label = label or option.get("label") or f"选项 {choice_index + 1}"
            return f"选择 {label}"
        return f"选择选项 {choice_index if choice_index is not None else '?'}"
    return _ACTION_NAMES.get(kind, kind)


def _player_line(
    label: str,
    player: Mapping[str, Any],
    entities: Mapping[int, Mapping[str, Any]],
    names: Mapping[str, str],
) -> str:
    hero = entities.get(int(player["hero"]), {})
    health = int(hero.get("max_health", 30)) - int(hero.get("damage", 0))
    armor = int(hero.get("armor", 0))
    card_class = _CLASS_NAMES.get(str(player.get("class")), str(player.get("class")))
    return (
        f"{label}：{card_class} 生命{health} 护甲{armor} "
        f"法力{player.get('mana', 0)}/{player.get('max_mana', 0)} "
        f"手牌{player.get('hand_size', 0)} 牌库{player.get('deck_size', 0)}"
    )


def render_state(decision: Mapping[str, Any], names: Mapping[str, str]) -> None:
    observation = decision["observation"]
    entities = {
        int(entity["entity"]): entity for entity in observation.get("entities", [])
    }
    print("\n" + "=" * 72)
    print(
        f"回合 {observation.get('turn', 0)} | 阶段 {observation.get('phase')} | 轮到你操作"
    )
    print(_player_line("对手", observation["opponent"], entities, names))
    opponent_weapon = observation["opponent"].get("weapon")
    print(
        "对手武器："
        + (
            _entity_label(int(opponent_weapon), entities, names, detailed=True)
            if opponent_weapon is not None
            else "（无）"
        )
    )
    opponent_board = observation["opponent"].get("board", [])
    print(
        "对手场面："
        + (
            " | ".join(
                _entity_label(int(ref), entities, names, detailed=True)
                for ref in opponent_board
            )
            if opponent_board
            else "（空）"
        )
    )
    print("-" * 72)
    own_board = observation["self_player"].get("board", [])
    print(
        "你的场面："
        + (
            " | ".join(
                _entity_label(int(ref), entities, names, detailed=True)
                for ref in own_board
            )
            if own_board
            else "（空）"
        )
    )
    print(_player_line("你", observation["self_player"], entities, names))
    own_weapon = observation["self_player"].get("weapon")
    print(
        "你的武器："
        + (
            _entity_label(int(own_weapon), entities, names, detailed=True)
            if own_weapon is not None
            else "（无）"
        )
    )
    hand = observation["self_player"].get("hand", [])
    print(
        "你的手牌："
        + (
            " | ".join(
                _entity_label(int(ref), entities, names, detailed=True) for ref in hand
            )
            if hand
            else "（空）"
        )
    )
    pending = observation.get("pending_choice")
    if pending:
        print(f"当前选择：{pending.get('prompt') or '请选择一项'}")


def play_interactive_match(
    env: HearthEnv,
    model: HearthQNetwork,
    tensorizer: Tensorizer,
    decks: Sequence[Sequence[str]],
    *,
    device: str,
    seed: int,
    human_seat: int,
    locale: str,
) -> None:
    names = card_names(env.card_catalog, locale)
    policy = ModelPolicy(model, tensorizer, device=device, seed=seed ^ 0xA17)
    decision = env.reset(seed=seed)
    transition: dict[str, Any] | None = None
    print("输入动作前的编号并回车；输入 q 退出。")
    while decision is not None:
        actor = int(decision["actor_seat"])
        if actor != human_seat:
            action_index = policy.choose(decision, decks[actor])
            action = next(
                action
                for action in decision["actions"]
                if int(action["index"]) == action_index
            )
            if action["kind"] == "mulligan":
                print("\nAI 已完成起手换牌。")
            else:
                print(f"\nAI：{describe_action(decision, action, names)}")
        else:
            render_state(decision, names)
            legal = {int(action["index"]): action for action in decision["actions"]}
            print("\n合法动作：")
            for index, action in legal.items():
                print(f"  {index:>3}  {describe_action(decision, action, names)}")
            while True:
                try:
                    raw = input("你的选择> ").strip().lower()
                except (EOFError, KeyboardInterrupt):
                    print("\n对局已退出。")
                    return
                if raw in {"q", "quit", "exit"}:
                    print("对局已退出。")
                    return
                try:
                    action_index = int(raw)
                except ValueError:
                    print("请输入合法动作编号，或输入 q 退出。")
                    continue
                if action_index in legal:
                    break
                print("该编号不在当前合法动作列表中。")
        transition = env.step(action_index)
        decision = transition["next"]

    assert transition is not None
    print("\n" + "=" * 72)
    if transition.get("truncated"):
        print("对局因步数上限被截断。")
        return
    reward = float(transition.get("rewards", [0.0, 0.0])[human_seat])
    if reward > 0:
        print("你赢了！")
    elif reward < 0:
        print("AI 获胜。")
    else:
        print("平局。")
