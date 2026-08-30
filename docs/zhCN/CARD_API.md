# Lua 卡牌 API（版本 1）

[English](../CARD_API.md) | [简体中文](CARD_API.md) | [繁體中文](../zhTW/CARD_API.md)

Lua 卡牌的 `name` 和 `text` 必须使用英文作为默认后备文本。正式显示文本由
`data/locales/enUS.json`、`zhCN.json` 和 `zhTW.json` 按官方 ID 覆盖。

动态提示通过运行时 locale 选择：

```lua
local prompt = ctx:localize(
    "Discover a spell",
    "发现一张法术牌",
    "發現一張法術牌"
)
```

`ctx.locale` 是只读 locale 代码；不得用 locale 改变规则、候选池或随机结果。

每个 `.lua` 文件必须返回一个 table。文件加载期间没有 `io`、`os` 或任意原生模块访问权限。

## 元数据

| 字段 | 类型 | 必需 | 说明 |
| --- | --- | --- | --- |
| `api_version` | integer | 是 | 当前只能是 `1` |
| `module_type` | string | 否 | 卡牌默认 `card`；独立英雄技能使用 `hero_power`；关键词模块使用 `keyword` |
| `id` | string | 是 | 卡牌包内唯一 ID |
| `name` | string | 是 | 显示名称 |
| `text` | string | 否 | 卡牌文字 |
| `set` | string | 否 | 官方卡牌包代码；仓库内的官方牌必须填写 |
| `type` | string | 卡牌模块 | `hero`、`minion`、`spell`、`weapon` 或 `location`；英雄技能模块无需填写 |
| `collectible` | boolean | 否 | 主卡默认 `true`；内嵌 token 默认 `false`，用于动态牌池过滤 |
| `class` | string | 否 | 卡牌职业，默认 `neutral`；Lua 可自行定义命名体系 |
| `rarity` | string | 否 | 印刷稀有度，统一为小写，用于动态牌池过滤 |
| `spell_school` | string | 否 | 印刷法术派系，统一为小写，用于动态牌池过滤 |
| `rune_cost` | table | 否 | 印刷死亡骑士符文需求，可填写 `blood`、`frost`、`unholy` 数量 |
| `tags` | string[] | 否 | 种族或卡牌包自定义标签，用于 Lua 牌池过滤 |
| `cost` | integer | 是 | 基础费用 |
| `attack` | integer | 随从/武器必需 | 随从或武器的基础攻击力 |
| `health` | integer | 随从/武器/地标必需 | 随从生命值；武器或地标中表示耐久度 |
| `armor` | integer | 英雄牌必需 | 打出英雄牌时获得的护甲 |
| `hero_power` | string | 英雄牌必需 | 替换出的独立英雄技能模块官方 ID |
| `keywords` | string[] | 否 | 对 `data/keywords/*.lua` 模块 ID 的引用；加载时校验存在性 |
| `keyword_params` | table<string, integer> | 否 | 数值型关键词的静态参数；键必须同时出现在 `keywords` 中 |
| `deck_allowances` | table[] | 否 | 卡牌提供的通用跨职业构筑许可；游客牌必须填写 |
| `secret` | boolean | 否 | 旧卡包兼容字段；新卡应引用 `keywords = { "secret" }` |
| `target_mode` | string | 否 | `optional`（默认）、`required` 或 `required_if_available`；普通牌控制出牌目标，地标控制激活目标 |

正式卡包可在卡牌根目录提供 `locales/enUS.json`、`locales/zhCN.json` 和
`locales/zhTW.json`，每项格式为 `{ "id", "name", "text" }`。加载器按官方 ID
把三语文本合并进不可变 `CardDefinition`；Lua 中的 `name/text` 是缺少 locale
文件时的后备文本。locale 文件属于卡包哈希的一部分。

目标模式的语义：

- `required`：必须有合法目标并选择一个；没有合法目标时不能使用。适合法术和英雄技能；
- `required_if_available`：有合法目标时必须选择；没有时仍能打出，效果函数收到 `nil`。适合带目标的战吼或连击随从；
- `optional`：可以无目标使用；若同时提供目标选择器，也可主动选择其中一个目标。

非 `optional` 模式必须同时定义对应的 `targets` 或 `location_targets`。旧字段
`requires_target = true` 暂时兼容并等价于 `target_mode = "required"`，新卡应使用
`target_mode`。

目标 selector 只在玩家声明动作时验证。目标选定后使用稳定 `EntityId` 记忆；出牌前置触发、控制权变化或位置光环不会令 Rust 再次运行 selector。这样攻击力条件等即使在战吼真正执行前发生变化，仍按原目标继续结算。

## 关键词模块

关键词文件同样返回 table，但使用 `module_type = "keyword"`。`rules` 中的函数签名为 `(ctx, self, current, other) -> value`：`current` 是前一个模块折叠后的值，`other` 是攻击者、目标或来源实体（没有时为 `nil`）。规则函数必须是只读的，不能输出效果。

```lua
return {
    api_version = 1,
    module_type = "keyword",
    id = "windfury",
    name = "风怒",
    rules = {
        max_attacks = function(ctx, self, current, other)
            return math.max(current, 2)
        end,
    },
}
```

关键词也可声明与卡牌相同格式的 `triggers`，并使用全部 `ctx` 效果 API。比如圣盾通过 `damaged/before` 调用 `disable_keyword` 和 `cancel_event`，亡语通过 `entity_died/after` continuation 调用卡牌效果，复生则调用 `summon_fresh_copy`。完整模块见 `data/keywords/`。

卡牌定义也可直接声明同签名的 `rules`。`turn_time_limit_seconds(ctx, self, current)` 是前端消费的通用回合时限规则：`0` 表示不限制，正数由所有在场来源继续折叠。命令行前端为交互玩家维护整回合截止时间，超时后提交普通选择或结束回合命令，因此 replay 仍只记录确定性的玩家命令。

关键词还可在 `hooks` 中实现 `on_play(ctx, self, target)` 或 `on_location_use(ctx, self, target)`，接入卡牌生命周期并输出普通效果。战吼模块利用 `hooks.on_play` 转到卡牌的 `on_battlecry`，连击模块先检查 `ctx:combo_active(self)` 再转到 `on_combo`，压轴模块则在支付当前费用后剩余法力为零时转到 `on_finale`；这是通用模块遍历，不是 Rust 中的关键词分支。未知 lifecycle hook 或非函数值会在加载时被拒绝。

关键词还可声明 `actions`，卡牌以 `card_actions` / `action_targets` /
`action_target_modes` / `action_effects` 增加或补充命名动作。锻造、预备和泰坦能力都走
这一接口；Rust 只处理动作费用、合法目标、事务和 replay，不判断关键词 ID。关键词的
`required_card_actions` 会在加载时验证卡牌确实提供对应效果。

关键词需要调用每张卡自己的效果时，可以声明 `required_card_hooks`，并在 lifecycle hook 或事件触发器里输出命名 continuation。例如战吼要求 `on_battlecry`，连击要求 `on_combo`，压轴要求 `on_finale`，法术迸发要求 `on_spellburst`，亡语要求 `on_deathrattle`。加载器会验证所有引用关键词的卡都提供对应 Lua 函数；Rust 只执行通用 continuation，不认识这些关键词 ID。

数值型关键词可设置 `requires_param = true`。加载器会要求每张引用卡在 `keyword_params` 中提供同名整数，关键词模块用 `ctx:keyword_param(self, keyword_id)` 读取；多余的、未被 `keywords` 引用的参数也会被拒绝。例如过载完整逻辑位于 `overload.lua`：

```lua
-- 卡牌定义
keywords = { "overload" },
keyword_params = { overload = 2 },

-- 关键词 hooks.on_play
local amount = ctx:keyword_param(self, "overload")
ctx:overload(ctx:controller(self), amount)
```

关键词还可用 `required_card_fields` 强制卡牌提供任意 Lua 字段。游客模块据此要求
`deck_allowances`，牌组验证器按声明开放指定职业与卡包，并排除目标职业的游客牌：

```lua
keywords = { "tourist" },
deck_allowances = {
    {
        class = "druid",
        set = "ISLAND_VACATION",
        excluded_keywords = { "tourist" },
    },
},
```

普通牌组默认执行本职业/中立限制和这些许可。仅用于规则沙箱的演示牌组可在 JSON 中显式
设置 `"unrestricted": true`；正式牌组不应使用该开关。

## 同文件衍生物

主卡可以在 `tokens` 中内嵌任意数量的专属衍生物。token 的 `api_version` 默认继承主卡。下面使用官方鬼灵爬行者及其衍生物 ID：

```lua
return {
    api_version = 1,
    id = "FP1_002",
    name = "鬼灵爬行者",
    set = "NAXX",
    type = "minion",
    cost = 2,
    attack = 1,
    health = 2,

    tokens = {
        {
            id = "FP1_002t",
            name = "鬼灵蜘蛛",
            set = "NAXX",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
        },
    },
}
```

主卡和 token 都进入同一个卡牌目录，并进行全局 ID 冲突检查。内嵌 token 默认 `collectible = false`，但可以显式覆盖。初始牌组只接受 `collectible = true` 的随从、法术、武器和地标；不可收集定义仍可由 Lua 创建、召唤或用于变形。因此具有专属衍生物的新卡仍然只需要一个 Lua 文件。

标准卡牌包还提供官方不可收集法术 `GAME_005`。双方完成 Rust 调度阶段后，引擎把一个该定义的实例交给 P2；它的正文完全由 Lua 编写，并按普通法术产生出牌、施法、历史和临时法力事件。

卡牌可以提供以下函数：

```lua
targets(ctx, self) -> entity_id[]
on_play(ctx, self, target_or_nil)
on_battlecry(ctx, self, target_or_nil) -- 引用 battlecry 关键词时必需
on_combo(ctx, self, target_or_nil)     -- 引用 combo 关键词时必需
on_finale(ctx, self)                   -- 引用 finale 关键词时必需
location_targets(ctx, self) -> entity_id[]
on_location_use(ctx, self, target_or_nil)
```

`targets/on_play` 用于法术、普通出牌效果与英雄技能；官方战吼、连击、压轴牌分别引用 `battlecry` / `combo` / `finale`，并在对应回调中实现独有效果。地标出牌本身始终无目标，其激活目标和效果分别由 `location_targets/on_location_use` 定义。`self` 和目标都是稳定的整数 `EntityId`，不是可以直接修改的 Rust 对象。

## 只读查询

```lua
ctx:controller(entity)          -- 返回 0 或 1
ctx:opponent(player)
ctx:player(player)              -- 返回玩家只读快照
ctx:cards_played_this_turn(player)
ctx:cards_played_last_turn(player)
ctx:combo_active(entity)        -- 此实体本次从手牌打出时，之前是否已打过牌
ctx:outcast_active(entity)      -- 此实体离手前是否位于最左或最右
ctx:entered_hand_this_turn(entity)
ctx:turn()
ctx:active_player()             -- 当前行动玩家，返回 0 或 1
ctx:cards_played(player)        -- 本局从手牌打出的卡牌 ID；包含被反制的牌
ctx:spells_cast(player)         -- 本局成功施放的法术 ID
ctx:minions_played(player)      -- 本局成功打出的随从 ID
ctx:minions_summoned(player)    -- 本局成功召唤的随从 ID（包括打出与效果召唤）
ctx:minions_died(player)        -- 本局死亡随从的冻结定义 ID
ctx:minions_died_this_turn(player) -- 当前回合死亡随从的冻结定义 ID
ctx:minion_death_records(player) -- { card_id, turn, had_deathrattle, keywords }
ctx:discarded_cards(player)     -- 按弃牌事件顺序记录的原实体 ID
ctx:discarded_card_ids(player)  -- 按弃牌事件顺序冻结的卡牌定义 ID
ctx:starting_deck(player)       -- 冻结的开局牌库卡牌 ID 多重集
ctx:cards_added_to_hand(player) -- 开局后按事件顺序进入手牌的卡牌 ID
ctx:overload_queued_total(player) -- 本局累计过载的水晶数
ctx:hero_was_healed_this_turn(player)
ctx:weapons_played(player)      -- 本局成功打出的武器 ID
ctx:locations_played(player)    -- 本局成功打出的地标 ID
ctx:last_spell_cast(player)     -- 上一张成功完成施放的法术 ID；没有则为 nil
ctx:hero_power_uses(player)     -- 本局成功使用英雄技能的累计次数
ctx:hand(player)                -- 当前手牌顺序
ctx:deck(player)                -- 牌库顶到牌库底
ctx:board(player)               -- 战场从左到右
ctx:secrets(player)             -- 奥秘进入顺序
ctx:graveyard(player)           -- 进入墓地顺序
ctx:characters()                -- 双方英雄及战场随从
ctx:minions()                   -- 双方战场随从
ctx:enemy_characters(entity)
ctx:enemy_minions(entity)
ctx:friendly_minions(entity)
ctx:adjacent_minions(entity)    -- 按左、右顺序返回最多两个实体
ctx:board_position(entity)      -- 零基位置；不在战场时返回 nil
ctx:entity(entity)              -- 返回只读快照 table
ctx:keyword_param(entity, id)   -- 返回卡牌定义上的关键词整数参数；没有则为 nil
ctx:card_ids()                  -- 当前卡牌包的全部 ID，稳定排序
ctx:collectible_cards()         -- 仅返回 collectible=true 的 ID
ctx:card_definition(card_id)    -- 返回不可变卡牌定义快照
ctx:get_player_data(player, key)
ctx:resource(player, resource_id)
ctx:resource_spent(player, resource_id)
ctx:has_enchantment_from(entity, source)
```

`has_enchantment_from` 判断实体是否仍保留由指定来源实体产生的附魔，适用于按来源独立叠加、并在可沉默附魔被移除后失效的持续效果。

实体快照字段包括：

```text
id, card_id, name, owner, controller, attack, health, max_health, damage, armor, spell_damage, zone, type,
keywords（字符串数组）, silenced, frozen, attacks_this_turn, attack_at_death, started_in_deck, location_cooldown, enchantments,
attached_cards, hook_attachments,
cards_played_before, combo_active
```

玩家快照字段包括：

```text
id, class, hero, hero_power, hero_power_used, hero_power_uses, hero_power_uses_this_turn, weapon, keywords, resources, resources_spent, mana, max_mana, temporary_mana, overload_pending, overloaded_mana,
cards_played_this_turn, cards_played_this_game, spells_cast_this_game,
minions_played_this_game, minions_summoned_this_game, weapons_played_this_game, locations_played_this_game,
fatigue, deck_size, hand_size, board_size, secret_count, hero_power_used
```

`cards_played_this_turn` 在玩家回合开始时清零。引擎在牌离开手牌并支付费用后增加它，所以被反制的牌仍计入本回合出牌；但当前牌自己的连击资格使用打出前快照 `cards_played_before`，不会把自己算作前置牌。`combo.lua` 在 lifecycle 阶段调用 `ctx:combo_active(self)`。目标 selector 发生得更早，带连击目标的卡应在 `targets` 中查询 `ctx:cards_played_this_turn(ctx:controller(self))`，未激活时返回空数组。两种上下文都属于 Rust 权威状态，可随等待选择、snapshot 和 replay 恢复。

`starting_deck` 在开局后保持冻结，牌被抽取、变形或摧毁都不会改变它；`cards_added_to_hand` 记录成功抽牌、生成牌和区域移动进入手牌，适合整局费用或进度规则。

五个历史查询返回按发生顺序冻结的卡牌定义 ID 数组，而不是动态反查实体：随从即使后来变形，历史中仍保留它打出时的原定义。`cards_played` 在费用支付并离手时记录，因此反制牌也存在；四个类型历史在相应 `spell_cast/minion_played/weapon_played/location_played` 成功时记录。法术自己的 `on_play` 结算期间尚未进入 `spells_cast`，完成正文后才成为 `last_spell_cast`。只有玩家实际施放的法术进入成功施法历史；由 `cast_spell` 或 `cast_existing_spell` 效果施放的法术带有 `generated_by`，既不进入该历史，也不触发“每当你施放法术”。这些数组属于 Rust 权威状态并进入 snapshot/replay，不应由 Lua 全局变量代替。

卡牌定义快照包含 `id, name, text, set, type, collectible, class, classes, rarity, spell_school, rune_cost, tags, cost, attack, health, secret, target_mode, requires_target, keywords, keyword_params`。`classes` 用于三职业等多职业牌；`rune_cost = { blood, frost, unholy }` 暴露印刷符文需求，无符文牌的三个值均为 0。其中 `requires_target` 只作为旧脚本兼容字段；新逻辑应读取 `target_mode`。例如可以完全在 Lua 中构造动态发现池：

```lua
local candidates = {}
for _, card_id in ipairs(ctx:collectible_cards()) do
    local card = ctx:card_definition(card_id)
    if card.type == "minion" and card.cost == 1 then
        table.insert(candidates, card_id)
    end
end
ctx:choose_cards(ctx:controller(self), "选择一张1费随从", candidates, "on_selected")
```

## 效果输出

这些函数不会在 Lua 中直接改变状态，只会把效果描述加入本次调用的输出缓冲区。Lua 函数成功返回后，Rust 才会验证和执行它们。

```lua
cardlib.effects.damage(ctx, target, amount)
cardlib.effects.damage_ignoring_spell_damage(ctx, target, amount)
cardlib.effects.damage_all(ctx, targets, amount)
cardlib.effects.heal(ctx, target, amount)
ctx:gain_armor(player, amount)
ctx:overload(player, amount)
ctx:unlock_mana(player, amount)
ctx:clear_overload(player)
ctx:gain_temporary_mana(player, amount)
ctx:gain_mana_crystals(player, amount, filled)
ctx:fill_mana_crystals(player, amount)
ctx:refresh_mana_crystals(player, amount?)
ctx:destroy_mana_crystals(player, amount)
ctx:spend_mana(player, amount)
ctx:gain_resource(player, resource_id, amount)
ctx:spend_resource_and_continue(player, resource_id, minimum, maximum, hook)
-- hook(ctx, self, spent) 在解析时执行；资源不足 minimum 时 spent 为 0
ctx:draw(player, count)
ctx:draw_entity(player, deck_entity)
ctx:create_card(player, card_id, spec_or_nil)
ctx:consume_sideboard_card(player, owner_card_id, card_id)
ctx:give_copy(player, entity, options_or_nil)
ctx:replace_hero(player, hero_card_id)
ctx:replace_hero_power(player, card_id)
ctx:refresh_hero_power(player)
ctx:exchange_zone_contents(first_player, second_player, "deck" | "hand" | "graveyard")
ctx:equip_weapon(player, card_id)
ctx:lose_weapon_durability(weapon, amount)
ctx:discard(player, entity)
ctx:cast_spell(player, card_id, options_or_nil)
ctx:cast_existing_spell(card, options_or_nil)
ctx:summon(player, card_id, options_or_nil)
ctx:summon_copy(player, target, options_or_nil)
ctx:recruit(player, deck_entity, options_or_nil)
ctx:summon_from_hand(card)
ctx:summon_existing(player, graveyard_entity, options_or_nil)
ctx:move(target, destination, options_or_nil)
ctx:shuffle_copy_into_deck(player, target)
ctx:change_controller(target, player)
ctx:change_controller_until_end_of_turn(target, player)
cardlib.effects.transform(ctx, target, card_id)
cardlib.effects.transform_all(ctx, targets, card_id)
cardlib.effects.transform_batch(ctx, { { target, card_id }, ... })
ctx:transform_into_copy(target, template, options_or_nil)
cardlib.effects.transform_preserving_scripts(ctx, target, card_id)
cardlib.effects.destroy(ctx, target)
cardlib.effects.destroy_all(ctx, targets)
cardlib.effects.damage_batch(ctx, { { target, amount }, ... })
cardlib.effects.damage_batch_ignoring_spell_damage(ctx, { { target, amount }, ... })
cardlib.effects.damage_batch_from(ctx, source, { { target, amount }, ... })
cardlib.effects.damage_from(ctx, source, target, amount)
ctx:add_attack_collateral(event_id, targets, amount)
ctx:force_attack(attacker, defender)
ctx:take_extra_turn(player)
ctx:win_game(player)
ctx:set_health(target, amount)
cardlib.effects.heal_all(ctx, targets, amount)
ctx:trigger_hook(target, hook)
ctx:attach_hook(target, hook, card_id)
ctx:attach_script(target, card_id)
ctx:board_position(target)
ctx:buff(target, options_or_nil)
cardlib.effects.modify_all(ctx, targets, modifier_table)
ctx:grant_keyword_until_next_turn(target, keyword)
ctx:disable_keyword(target, keyword)
ctx:grant_player_keyword(player, keyword)
ctx:grant_public_player_status(player, status)
ctx:disable_public_player_status(player, status)
ctx:disable_player_keyword(player, keyword)
ctx:set_player_class(player, class_id)
ctx:summon_fresh_copy(target, options_or_nil)
ctx:silence(target)
ctx:freeze(target)
ctx:reveal_secret(secret)
ctx:cancel_event(event)
cardlib.effects.set_event_amount(ctx, event, amount)
cardlib.effects.add_event_amount(ctx, event, amount)
cardlib.effects.multiply_event_amount(ctx, event, factor)
cardlib.effects.give_card(ctx, player, card_id)
cardlib.effects.give_card_at(ctx, player, card_id, position)
cardlib.effects.shuffle_card_into_deck(ctx, player, card_id)
cardlib.effects.give_copy_with_stats(ctx, player, target, attack, health, cost_or_nil)
cardlib.effects.give_base_copy(ctx, player, target)
cardlib.effects.give_base_copy_with_stats(ctx, player, target, attack, health, cost_or_nil)
cardlib.effects.summon_at(ctx, player, card_id, position)
cardlib.effects.summon_with_stats(ctx, player, card_id, attack, health, keywords_or_nil)
cardlib.effects.summon_with_base_stats(ctx, player, card_id, attack, health, keywords_or_nil)
cardlib.effects.summon_existing_at(ctx, player, target, position)
cardlib.effects.recruit_at(ctx, player, target, position)
cardlib.effects.move_to_hand(ctx, player, target)
cardlib.effects.shuffle_entity_into_deck(ctx, player, target)
cardlib.effects.transform_into_copy_with_stats(ctx, target, template, attack, health)
cardlib.effects.buff(ctx, target, attack_delta, health_delta)
cardlib.effects.buff_until_end_of_turn(ctx, target, attack_delta, health_delta)
cardlib.effects.grant_keyword(ctx, target, keyword)
cardlib.effects.grant_keyword_until_end_of_turn(ctx, target, keyword)
cardlib.effects.summon_copy_at(ctx, player, target, position)
cardlib.effects.summon_copy_with_stats(ctx, player, target, attack, health)
cardlib.effects.summon_fresh_copy(ctx, target, position_or_nil, health, without_keywords)
cardlib.effects.summon_fresh_copy_with_stats(ctx, target, position_or_nil, attack, health, without_keywords)
ctx:set_attack_defender(event_id, defender)
ctx:set_damage_target(event_id, target)
ctx:set_spell_target(event_id, target)
ctx:replace_trade_draw(event_id, replacement_entity)
ctx:continue_with(hook_name)
ctx:continue_with_entity(hook_name, entity)
ctx:continue_with_card(hook_name, card_id)
ctx:continue_with_number(hook_name, number)
ctx:continue_with_value(hook_name, serializable_value)
ctx:set_player_data(player, key, value)
ctx:increment_player_data(player, key, delta)
```

所有效果的 `source` 自动设为当前执行 hook 的卡牌实体。

`cardlib.effects` 是卡牌层的 Lua 便捷库。每种 Rust 效果只保留一个参数化原语：`create_card`、`give_copy`、`summon`、`summon_existing`、`recruit`、`move`、`transform_into_copy`、`buff`、`summon_copy` 和 `summon_fresh_copy`；位置、目标区域、持续时间、复制状态和属性变体都由 Lua 语法糖组合。`summon` 支持 `position`、互斥的 `base_stats`/`final_stats` 和 `keywords`；`give_copy` 支持 `state = "preserve" | "definition"`、`final_stats` 和 `cost`；`move` 通过 `{ player = ... }` 指定目标玩家；`buff` 支持 `attack`、`health`、`keywords` 与 `duration = "permanent" | "end_of_turn"`。fresh-copy 的 `remaining_health` 与 `final_stats` 互斥。批量接口仍用于保证组操作的原子性。

光环中的 `cost` 是加法层；卡牌文字写“费用为（1）”时使用 `cost_set = 1`（也可为函数），需要限制最终费用时使用 `cost_cap`。费用光环顺序为 `Aura SET → Aura ADD → Aura CAP`。

`spell_damage` 光环可以指向随从或英雄。玩家的法术伤害加成为己方场上随从与英雄所承载数值之和，因此双方玩家级效果无需任何卡牌特判。

`replace_hero` 要求目标定义为 Hero 且声明有效的 `hero_power`：新英雄使用定义中的生命上限并回满生命，保留原英雄的护甲、冻结状态和本回合攻击次数，同时替换英雄技能并发布 `hero_replaced`/`hero_power_replaced`。`grant_player_keyword` 与 `disable_player_keyword` 只管理可执行的玩家级脚本机制；公开展示与规则执行正交，由 `grant_public_player_status` 和 `disable_public_player_status` 管理并投影到双方视图与 RL 观察。

`destroy_all` 在同一个死亡检查点摧毁所有目标，适用于“摧毁所有随从”一类同时结算；`move` 的目标区域包括 `hand`、`secret`、`deck_top`、`deck_bottom`、`deck_random`、`graveyard` 和 `removed`。移动到 `secret` 时会校验该实体确实具有奥秘规则且奥秘区未满。`shuffle_entity_into_deck` 使用 Rust 确定性随机把原实体洗入指定玩家牌库，同时转移 owner/controller 并执行隐藏区重置。

`transform` 允许手牌和牌库隐藏区中的实体跨卡牌类型原位替换，并保留实体身份与区域顺序。`transform_preserving_scripts` 还会保留 `attached_cards` 与脚本数据；需要跨自身变形持续的规则应先用 `attach_script` 附加可复用模块。`attach_hook` 可向任意命名 Lua 钩子附加有序、可叠加的卡牌脚本；沉默会移除随从已有的钩子附件。

`cast_spell` 从定义创建法术，`cast_existing_spell` 施放隐藏区或终止区中的已有实体；两者都接受 `{ target = entity, skip_if_invalid = true, random_target = true, choice_policy = "random" }`。随机目标和自动抉择现在是显式策略，不再借用隐藏脚本数据。连续随机施法由 Lua 组合，公共库 `cardlib.random_spell` 复用权威 `random_value` 与 `cast_spell`。

`create_card` 支持 `destination`、可选手牌 `position`、`attack`、`health`、`cost`、`spell_damage`、`keywords`、`attached_scripts` 和 `started_in_deck`。`consume_sideboard_card` 只移除指定身份；脚本在同一个事务命令中把它与 `create_card(..., { started_in_deck = true })` 组合。属性合成公式留在 Lua；`cardlib.fusion.create_minion` 是可复用的合成随从实现。声明 `module_type = "library"` 的文件会暴露为 `cardlib[id]`，参与校验和确定性卡包哈希，但不会注册成卡牌。

`damage_ignoring_spell_damage` 仍走普通顺序伤害与事件流程，但不叠加来源控制者的法术伤害。`spend_mana` 原子地花费玩家当前可用法力（优先临时法力），并按实际正数花费发布 `mana_spent`。`increment_player_data` 对玩家脚本数据执行原子有符号累加，发布带 `old/new/delta` 的 `player_script_data_changed`，避免同一快照收集出的多个触发器互相覆盖。死亡记录保存卡牌定义是否原生具有亡语：沉默不清除该标记，附加亡语也不会设置它。

`give_copy` 用于向前或同区域复制，保留来源实体的永久状态；`give_copy_with_stats` 再附加最终攻击、生命及可选费用定值。`give_base_copy*` 用于战场到手牌等向后区域复制，只从印刷定义创建无增益副本。

`draw_entity` 从指定玩家牌库抽取该原实体，并走可取消的普通 CardDrawn/CardBurned 流程。`summon_existing` 把墓地或移除区的原随从送入完整可取消召唤流程，取消或满场时恢复；`summon_existing_at` 还会使用记录的原战场位置。`move_to_hand` 可把原实体转入指定玩家手牌，`shuffle_copy_into_deck` 会保留被复制实体的状态。`summon_copy` 会保留牌库、手牌或战场中存活随从的运行时状态；可选的 `final_stats` 会在同一原子操作内给新实体附加可沉默的最终攻血。墓地实体应使用 `summon_fresh_copy`，它默认从卡牌定义创建满血、无增益的新实例；`remaining_health` 保留印刷生命上限但以受伤状态入场，`final_stats` 则通过可沉默的最终定值替换显示攻血。`summon_with_stats` 会附加可沉默的最终攻血；`summon_with_base_stats` 会替换印刷基础属性，因此青玉魔像等动态衍生物不会因沉默恢复。`lose_weapon_durability` 扣除已装备武器耐久，归零时走普通可取消的 `weapon_destroyed` 生命周期。`add_attack_collateral` 为待结算攻击加入同批战斗伤害。

`damage_batch` 对冻结目标集原子结算不同伤害值，其忽略法术伤害版本不叠加法强；`damage_batch_from` 还可显式指定造成整批伤害的实体。`set_spell_target` 只在 `spell_targeted` 事件的触发效果仍在结算时有效；它把目标改为一个仍在战场上的随从，并让法术正文随后使用新目标。`modify_all` 对冻结目标组应用同一属性规格；`modify_batch` 接受逐实体规格，每个属性操作不同时可传 `modifiers` 数组。两者都支持 `reset_damage = true`。`force_attack` 无需攻击者处于可攻击状态即可发起完整攻击事件；`take_extra_turn` 为指定玩家排入可回放的额外回合。`grant_keyword_until_next_turn` 在该随从控制者的下回合开始时到期，且不依赖来源实体继续存在。

区域查询返回稳定顺序的实体 ID 副本，脚本不能修改 Rust 内部列表。`hand`、`deck` 等接口也允许查询对手隐藏区；Lua 卡牌是服务端可信规则代码，UI 不会直接获得这些结果。需要向玩家公开或选择隐藏信息时，应由卡牌显式构造 `choose_*` 选项。

`discard` 只对参数玩家当前手牌中的实体生效；实体已被更早效果移走时安全地不做任何事。引擎先发布 `card_discarded/before`，此时目标仍在手牌，因此目标自己的 `active_zones = { "hand" }` 触发器可以调用 `cancel_event`。成功提交后卡牌进入墓地，依次批量发布 `card_discarded/after` 和 `zone_changed/after`；取消时不发布二者。事件字段为 `source`、`player`、`entity`。随机弃牌应先从 `ctx:hand(player)` 过滤候选，再调用 `random_entity` 并在 resume hook 中 `discard`，以保持 Rust RNG、事件日志和 replay 的确定性。

`cast_spell` 由 Rust 创建一个新的真实法术实体，并用它作为法术伤害、治疗和后续事件的来源。普通法术进入墓地，满足 `enters_secret_zone` Lua 规则的法术进入奥秘/任务区；不支付法力、不产生 `card_played`、不增加连击计数，也不会触发只反制手牌出牌的 `card_played/before` 奥秘。法术正文全部结算后发布 `spell_cast/after` 并进入 `spells_cast` 历史。该事件额外包含 `generated`、`generated_by` 和声明的 `target`；直接从手牌施放时前两者分别为 `false/nil`，效果施放时指向产生它的实体，无目标法术的 `target` 为 `nil`。抽到即施放使用 `cast_existing_spell(self, { skip_if_invalid = true })`，移动并施放同一个实体。

目标法术必须传入目标，且目标必须通过该法术自身的 Lua `targets` 选择器以及 Rust 的潜行/免疫过滤；缺失或非法目标会令当前玩家命令事务回滚。无目标法术可省略第四个参数。自动施放非 spell 定义同样是脚本错误。自动施放奥秘时若奥秘区已满则不产生任何实体或事件。`CastSpell` 本身可序列化，因此可以位于等待选择的剩余结算队列中，并由 snapshot/replay 恢复。

连续调用多次 `damage` 表示依次结算多个伤害效果。需要“同时造成伤害”时使用 `damage_all`：Rust 会去重目标，为每个目标分别发布 `damaged/before`，然后一次性提交所有未取消的伤害，再以同一份提交后状态批量发布 `damaged/after`，最后进入死亡检查点。

`summon` 把随从放在战场最右侧；`summon_at` 使用从 `0` 开始的位置，允许值为 `0..当前随从数`。位置越界会令本次玩家命令事务回滚。如果 `minion_summoned/before` 的效果令战场缩短，提交时会把原位置收缩到新的最右边界。

`entity_died` 事件包含控制者与该随从被移除时的零基 `position`。同一死亡检查点按入场顺序移除随从，后死者的位置在先死者移除后计算。`deathrattle.lua` 会把该值作为第三个参数传给 `on_deathrattle(ctx, self, position)`；若要从死亡位置开始放置衍生物，应显式调用 `summon_at(player, card_id, position)`。连续召唤到同一位置时，后一次会插在前一次左侧。复生模块也使用该位置恢复实体。

`ctx:summon_copy` 复制效果结算时仍在牌库、手牌或战场中的存活随从状态。副本获得新的 `EntityId`，继承当前卡牌定义、伤害、冻结、沉默、已消耗关键字、enchantment 和 `script_data`；每个 enchantment 获得新 ID，原本以自身为来源的 enchantment 会重映射到副本。options 中的 `final_stats` 会在同一个效果内额外附加最终攻血并清除副本伤害，`position` 指定零基战场位置。光环不作为永久状态复制，而是在副本入场后按新位置重新计算。副本的拥有者和控制者是参数中的玩家，攻击次数重置，并遵循普通新召唤随从的休眠/冲锋规则。`cardlib.effects.summon_copy_at` 和 `summon_copy_with_stats` 只负责构造这些 options，不是额外的 Rust 操作。

复制仍发布可取消的 `minion_summoned/before` 和成功后的 `after`，不会再次执行被复制随从的 `on_play`、关键词 lifecycle hook 或战吼。墓地实体不能作为状态复制模板；复活或其他需要无增益、无伤害实例的效果应使用 `ctx:summon_fresh_copy`，常见参数组合由 `cardlib.effects.summon_fresh_copy*` 包装。

`recruit` 和 `recruit_at` 从参数玩家的牌库移动指定随从实体到战场，后者使用零基位置。它保留原 `EntityId`，不创建副本、不执行 `on_play`、关键词 lifecycle hook 或战吼，但会发布普通的 `minion_summoned/before/after`，因此召唤触发器、取消和光环行为与其他效果召唤一致。常见的随机招募应组合牌库查询、Rust RNG 和命名恢复：

```lua
local candidates = {}
local player = ctx:controller(self)
for _, entity in ipairs(ctx:deck(player)) do
    if ctx:entity(entity).type == "minion" then
        candidates[#candidates + 1] = entity
    end
end
if #candidates > 0 then
    ctx:random_entity(candidates, "on_recruit")
end

function card.on_recruit(ctx, self, entity)
    cardlib.effects.recruit_at(ctx, ctx:controller(self), entity, 0)
end
```

目标已不在该玩家牌库或战场已满时安全地不做任何事；牌库中的非随从目标属于脚本错误。引擎在 before 阶段把原实体移入内部 `set_aside`，同时保存原下标及前后实体锚点。取消、before 效果填满战场或终局截断时会按仍存在的锚点恢复其相对顺序；等待选择时该预留信息随 snapshot/replay 序列化。

`gain_armor` 为指定玩家的英雄增加护甲并发布 `armor_gained/after`。英雄受到伤害时先扣护甲，再扣生命；`damaged` 的数值和吸血仍按本次实际结算的伤害量计算。治疗只恢复生命，不恢复或消耗护甲。

`overload` 是通用的法力效果原语，把数值累加到玩家的 `overload_pending`，并发布 `overload_queued/after`。具体“过载”关键词的出牌时机和参数读取由 `data/keywords/overload.lua` 定义。该玩家下个回合开始时，待过载转换为不超过当前最大法力的 `overloaded_mana`，可用法力设为 `max_mana - overloaded_mana`，发布 `mana_locked/after`，随后才发布 `turn_started`。锁定只持续该回合；新产生的过载继续进入再下回合债务。

`unlock_mana` 只解锁当前回合的 `overloaded_mana`，恢复等量可用法力并发布 `mana_unlocked/after`，不会清除 `overload_pending`。数值超过当前锁定量时只解锁实际存在的数量。

`clear_overload` 同时清零当前 `overloaded_mana` 和下回合 `overload_pending`，恢复当前被锁定的可用法力，并发布 `overload_cleared/after`（包含 `locked` 与 `pending`）。熔岩震击使用这个通用原语；它不会影响该效果结算之后新产生的过载。

`gain_temporary_mana` 增加本回合可用法力与 `temporary_mana`，发布 `temporary_mana_gained/after`。支付卡牌或英雄技能费用时优先消耗临时法力；`mana_spent/after` 包含 `player`、支付来源 `source`、总 `amount` 和其中的 `temporary`。0 费动作不发布支付事件。剩余临时法力在 `turn_ended` 触发及回合末 enchantment 结算后扣除，并发布 `temporary_mana_expired/after`。

交易是玩家行动而不是效果 API。具有 `can_trade = true` 规则的手牌可以花费 1 点法力交易；牌库为空时不能交易。引擎先把该实体暂存，发布可响应的 `trade_draw/before`，执行一次普通抽牌，再将原实体插入牌库的确定性随机位置，因此不会抽回同一个实体，满手牌时也不会爆掉替换牌。交易不增加出牌计数或任何出牌历史，保留原实体上的 enchantment，最终发布 `card_traded/after`，并完整进入 replay。

`trade_draw/before` 的 `event_id` 可交给 `replace_trade_draw`，把默认牌库顶抽牌替换成指定牌库实体；指定实体在提交前离开牌库时安全回退到普通抽牌。before hook 可以先把事件 ID 保存到 `script_data`，调用 `discover_entities` 暂停结算，再在 resume hook 中选择替代实体。抽牌完成后发布 `trade_draw/after`，此时事件的 `replacement` 为实际请求的实体；随后原交易牌进入牌库并发布 `card_traded/after`。

`gain_mana_crystals(player, amount, filled)` 最多把 `max_mana` 增至 10；`filled = true` 时同时增加等量当前法力，否则获得空水晶。`destroy_mana_crystals` 降低最大法力，并将当前永久法力与锁定量收缩到新容量，不影响临时法力。对应事件为 `mana_crystals_gained`（额外含 `filled`）和 `mana_crystals_destroyed`。

`move` 支持以下目标位置：

```text
hand, board, secret, deck_top, deck_bottom, deck_random, graveyard, removed
```

移动到手牌、牌库或战场时，实体回到拥有者控制，清除伤害、沉默、enchantment 和 `script_data`。`board` 只接受墓地中的随从且不会发布 `minion_summoned`，用于休眠复归等明确不属于召唤的效果。手牌已满时，返回手牌会改为进入墓地。`deck_random` 只随机选择插入位置，不扰乱牌库中其他卡牌的相对顺序；随机结果由 Rust RNG 决定并可通过 replay 精确重建。英雄、英雄技能、`set_aside` 和 `removed` 实体不能再次通过此 API 移动。

`trigger_hook(target, hook)` 在目标实体上调用指定的 Lua 生命周期钩子，按普通效果队列继续结算。它适合“触发一个随从的亡语”这类不伴随死亡事件的效果；调用方仍负责按卡牌规则筛选目标和处理重复次数。

`change_controller` 对战场随从和奥秘生效。目标玩家对应区域已满、目标已经离场或控制权已经相同时不产生变化；随从成功时移动到新控制者战场最右侧并进入休眠，再由关键词的 `ready_on_summon` 规则决定是否解除休眠；奥秘则移动到新控制者的奥秘区。该操作发布可取消的 `controller_changed/before`，提交后发布 `controller_changed/after`。实体的拥有者 `owner` 不变，之后返回手牌或牌库仍回到拥有者一方。

`change_controller_until_end_of_turn` 记录可逆的战场随从控制权：沉默会立即把随从归还原控制者，变形会清除归还标记并让当前控制权永久化；回合结束时若原方战场已满，该随从会被消灭。`refresh_mana_crystals` 只刷新现有且未被过载锁定的永久水晶；可选数量省略时补满，并始终保留临时法力、当前过载和待结算过载。`summon_with_stats` 使用可沉默的最终属性层，`summon_with_base_stats` 则直接设置召唤物基础攻血，沉默不会把青玉魔像等成长衍生物还原。

`transform` 只接受战场随从和另一张随从定义。变形保留实体 ID、拥有者、控制者、战场位置、休眠状态和本回合攻击次数；基础属性与卡牌脚本替换为新定义，并清除伤害、冻结、沉默、enchantment、已消耗关键字状态和 `script_data`。变形不算死亡或召唤，发布可取消的 `transformed/before`，提交后发布 `transformed/after`。`transform_all` 对整组应用同一定义，`transform_batch` 对每个实体应用各自定义，二者都统一提交并只重算一次光环。`transform_into_copy` 复制模板实体完整状态，再施加可沉默的最终攻血值。

`destroy` 可以消灭战场随从、摧毁战场地标或已装备武器。随从进入统一死亡检查点并发布 `entity_died`；地标立即移入其控制者墓地并发布 `location_destroyed`；武器走可取消的 `weapon_destroyed` 生命周期。其他区域及其他实体类型不会被该效果改变。

`set_health(target, amount)` 不发布治疗事件，直接把当前生命值和生命值上限都设为指定数值。它会创建一个可沉默的永久 enchantment，适合表达生命值交换等卡牌正文。

`buff` 和 `grant_keyword` 会创建可追踪的永久 enchantment，而不是直接篡改基础属性。`silence` 会移除可沉默 enchantment、印刷关键字和脚本触发能力。

通用属性修改使用 `modify`：

```lua
cardlib.effects.modify(ctx, target, {
    stat = "attack",             -- attack / health / cost / spell_damage
    operation = "set",           -- set / add / pre_final_add / multiply / final_set
    value = 5,
    duration = "end_of_turn",    -- permanent（默认）/ end_of_turn
    silenciable = true,           -- 默认 true；持续整局规则可设为 false
})
```

没有 `final_set` 时，永久属性按 `SET → ADD/PRE_FINAL_ADD → MULTIPLY` 分层。存在 `final_set` 时，最后一层定值成为基准，仅继续应用它之后创建的普通 Set/Add/Multiply；`pre_final_add` 永远位于该定值之前。实时光环最后应用。回合末 enchantment 在 `turn_ended` 触发全部结算后统一移除，然后再次进行光环、死亡和胜负检查。

`ctx:remove_enchantments_from(target, source)` 删除指定来源创建的全部 enchantment。
它与 `silenciable = false` 组合，可表达水晶核心这类不能被沉默、但随控制权变化需要移除的持续规则。

## 标准关键字

关键词由 Lua 在元数据或 `grant_keyword` 中引用；具体规则由同 ID 的 Lua 关键词模块执行。Rust 只折叠 `attack_priority`、`can_be_attacked`、`can_be_targeted_by_enemy`、`can_attack_while_exhausted`、`ready_on_summon`、`max_attacks`、`can_trade` 等通用规则，并执行触发器输出的通用效果。

- `immune`：不能成为敌方定向效果或攻击目标；`damaged/before` 由模块取消；
- `taunt`：存在敌方嘲讽随从时，其他角色不能成为攻击目标；
- `charge`：进入战场的当回合即可攻击任意合法目标；
- `rush`：进入战场的当回合只能攻击随从，下个己方回合解除限制；
- `windfury`：每回合最多攻击两次；
- `divine_shield`：在 `damaged/before` 禁用自身并取消伤害，发布通用 `keyword_disabled/after`；
- `poisonous`：对随从造成正数实际伤害后，将其标记为致死；不能穿过圣盾。
- `lifesteal`：按实际正数伤害治疗来源控制者的英雄，不超过其已损失生命；在死亡检查点之前结算。
- `stealth`：敌方不能把潜行随从选作定向卡牌目标或攻击目标；随机和群体效果仍可命中。该随从发起攻击时失去潜行。
- `reborn`：随从第一次死亡后召唤一个全新的同卡牌实体，以 1 点生命回到战场且不再具有复生；新实体会正常发布 `minion_summoned`。
- `elusive`：拒绝敌方法术和英雄技能的定向目标查询。
- `tradeable`：在手牌中通过通用 `can_trade` rule 开放交易行动；不依赖 Rust 关键词名分支。

关键词文件结构见 [架构说明](ARCHITECTURE.md)。`disable_keyword` 是可用于任意关键词的通用原语；返回手牌或牌库时，已禁用状态会随其他战场状态一起重置。再次通过 `grant_keyword` 获得同一关键词会恢复它。沉默会移除印刷关键词、可沉默 enchantment 提供的关键词和卡牌/关键词触发能力。

冻结不是可沉默关键字，而是角色状态。`ctx:freeze(target)` 可冻结战场随从或英雄并发布 `frozen/after`；被冻结角色不能攻击，在其控制者结束自己的下一个回合时解冻。返回手牌或牌库会清除冻结。

武器攻击的伤害来源实体是英雄。设置 `weapon_inherits_to_hero = true` 的关键词模块会参与当前出鞘武器的英雄规则查询；关键词事件模块也可在 `weapon` 区监听英雄作为来源的伤害。

法术伤害的印刷值是参数化 Lua 关键词，而不是 `CardDefinition` 特殊字段：

```lua
keywords = { "spell_damage" },
keyword_params = { spell_damage = 1 },
```

`spell_damage.lua` 通过通用 `base_spell_damage` 数值规则提供基础值。Rust 随后应用 `SET → ADD → MULTIPLY → aura` 属性分层，并把场上己方随从最终的 `entity.spell_damage` 加到每个来源类型为 `spell` 的正数伤害效果上。随从战吼、英雄技能、攻击、疲劳和武器伤害不会获得该加成。`modify(... stat = "spell_damage")` 仍可修改该通用属性；沉默会移除印刷法强和可沉默 enchantment 提供的法强，外部光环仍按普通光环规则重新计算。

## 地标

地标是 `type = "location"` 的普通可收集卡牌，`health` 表示耐久度。它与随从共用七个战场位置，但不是角色：不能攻击、不能成为攻击目标，也不会出现在 `characters`、`minions`、`friendly_minions`、`enemy_characters` 或 `adjacent_minions` 的结果中。`ctx:board(player)` 会返回该玩家战场上的所有随从和地标，因此脚本可以检查 `ctx:entity(id).type` 来筛选地标。

地标打出时不选择目标，可照常实现 `on_play`。进入战场后可以立刻免费使用；每次使用消耗 1 点耐久度，并在下一个己方回合保持冷却，到再下一个己方回合恢复。实体快照中的 `location_cooldown` 在使用后为 `2`，两个己方回合开始时依次变成 `1`、`0`。耐久耗尽后自动进入墓地；最后一次使用会先清出地标的战场位置，再执行能力，因此满场时仍可由该能力召唤随从。普通伤害、治疗、冻结、属性增益和光环都不能改变地标耐久；需要移除地标时使用 `cardlib.effects.destroy(ctx, target)`。

```lua
return {
    api_version = 1,
    id = "CUSTOM_TRAINING_GROUNDS",
    name = "训练场",
    type = "location",
    cost = 1,
    health = 2,
    target_mode = "required",

    location_targets = function(ctx, self)
        return ctx:friendly_minions(self)
    end,

    on_location_use = function(ctx, self, target)
        ctx:buff(target, 1, 1)
    end,
}
```

`location_used` 支持 before/after。使用时会在 before 之前预留本次耐久消耗与冷却；取消事件会保留这些消耗，但不执行 `on_location_use`，也不发布 after。成功打出与摧毁分别发布 after-only 的 `location_played`、`location_destroyed`。

## 武器、英雄技能和奥秘

武器使用普通卡牌定义；`attack` 是英雄在自己回合获得的攻击力，`health` 是耐久度。对手回合武器处于收起状态，不向英雄提供攻击、剧毒或吸血等来源关键字，因此敌方随从攻击该英雄时不会受到武器反击。回到持有者回合后武器重新生效。英雄每次主动完成一次攻击后失去 1 点耐久度，耐久度归零时武器进入墓地。装备新武器会摧毁旧武器。

`weapon_equipped` 和 `weapon_destroyed` 均支持 before/after：

- 取消装备：费用与卡牌仍被消耗，武器进入墓地，仍发布 `card_played/after` 并执行该武器的 `on_play`；
- 取消替换旧武器的摧毁：旧武器保留，新武器进入墓地；
- 取消耐久归零的摧毁：若脚本没有修复耐久，引擎将其恢复为 1。

英雄技能是独立 Lua 模块：每个文件声明 `module_type = "hero_power"`，加载器自动赋予 `type = "hero_power"` 和 `collectible = false`。它可以实现 `targets` 和 `on_play`，费用、目标与效果复用统一运行时接口。牌组 JSON 用 `hero_power` 指定技能；省略时使用官方 ID `HERO_08bp`。Rust 只负责当前回合限用一次和支付费用。

英雄牌仍是可收集卡牌模块，使用 `type = "hero"`，并声明 `health`、`armor` 和替换技能的官方 ID `hero_power`。打出时保留当前生命值和伤害，获得护甲，替换英雄实体与英雄技能，然后执行 Lua 生命周期钩子；引擎会发布 `hero_replaced` 和 `hero_power_replaced`。示例牌组配置：

```json
{
  "name": "自定义牌组",
  "class": "mage",
  "hero_power": "MY_HERO_POWER",
  "cards": ["MY_CARD"]
}
```

`class` 是 1 到 64 字节的玩家职业标识，省略时为 `mage`。它进入 `PlayerState`、replay 和 snapshot，可由 Lua 通过 `ctx:player(player).class` 查询。普通 `Game::new*` 建局会限制牌组只能包含本职业、中立、包含本职业的多职业卡，以及 Tourist 等卡牌声明许可的跨职业卡；规则测试必须显式使用 `Game::new_unrestricted*` 才能混合职业。

`hero_power_used` 支持 before/after。费用与本回合次数会在 before 之前预留；取消事件会保留这些消耗，但不发布 after，也不执行英雄技能 `on_play`。

奥秘是引用 `secret` 关键词的法术。`secret.lua` 提供 `enters_secret_zone` 规则；打出后实体进入 `secret` 区域，触发器必须把该区域列入 `active_zones`，触发时显式揭示：

```lua
triggers = {
    {
        event = "attack",
        active_zones = { "secret" },
        condition = function(ctx, self, event)
            local defender = ctx:entity(event.defender)
            return defender.type == "hero"
                and defender.controller == ctx:controller(self)
        end,
        effect = function(ctx, self, event)
            ctx:reveal_secret(self)
            cardlib.effects.damage(ctx, event.attacker, 2)
        end,
    },
}
```

`attack` 的 `before` 时点表示“攻击已经声明、战斗伤害尚未发生”。它产生的触发效果和死亡会先结算；若攻击者或防御者已离场或濒死，本次战斗取消。

## 实体脚本数据

卡牌的计数器和任务进度必须保存在实体上，不能放进共享的 Lua module table：

```lua
local count = ctx:get_data(self, "counter")
ctx:set_data(self, "counter", count + 1)
```

`script_data` 的值当前为有符号 64 位整数，键最长 64 字节。它属于 `GameState`，因此支持事务回滚、序列化和 replay。

## 持续光环

光环由 Lua 声明来源生效区域、目标和修正值，Rust 在抽牌、出牌、区域、沉默或 enchantment 变化后重新计算。省略 `active_zones` 时默认只在战场生效：

```lua
auras = {
    {
        attack = 1,
        health = 1,
        cost = 0,
        spell_damage = 0,
        keywords = { "taunt" },
        targets = function(ctx, self)
            local result = {}
            for _, entity in ipairs(ctx:friendly_minions(self)) do
                if entity ~= self then
                    result[#result + 1] = entity
                end
            end
            return result
        end,
    },
}
```

`attack`、`health`、`cost` 和 `spell_damage` 可以是固定整数，也可以是只读函数 `(ctx, self) -> integer`。例如一张在手牌中根据当前手牌数动态减费的随从可以完全由 Lua 定义：

```lua
auras = {
    {
        active_zones = { "hand" },
        cost = function(ctx, self)
            return -#ctx:hand(ctx:controller(self))
        end,
        targets = function(ctx, self)
            return { self }
        end,
    },
}
```

`active_zones` 可使用与触发器相同的 `hero/hero_power/deck/hand/board/weapon/secret/graveyard`。来源进入内部 `set_aside` 或 `removed` 时永不生效；被沉默的来源也不产生光环。目标选择器和动态数值函数都是只读的，尝试输出效果会被视为脚本错误并回滚玩家命令。

一次重算会先移除全部旧光环，基于无光环的同一份稳定状态收集所有 selector 和动态数值，再按目标聚合，最后统一应用并夹取属性范围。因此 `-2` 与 `+2` 不会因为来源创建顺序不同而得到不同结果。费用在永久 enchantment 的 `SET → ADD → MULTIPLY` 之后加上光环总和并夹到 `0..255`；攻击、生命和法强同样在自身 enchantment 层之后应用。

## 玩家选择和确定性随机

需要暂停结算等待玩家输入时，传入候选实体和一个模块中的命名回调：

```lua
local card = {
    api_version = 1,
    id = "CUSTOM_CHOICE",
    name = "自定义抉择",
    type = "spell",
    cost = 1,
}

function card.on_play(ctx, self, target)
    ctx:choose_entities(
        ctx:controller(self),
        "选择目标",
        ctx:enemy_characters(self),
        "on_target_chosen"
    )
end

function card.on_target_chosen(ctx, self, choice)
    cardlib.effects.damage(ctx, choice, 2)
end

return card
```

引擎会把选项、来源实体和 `on_target_chosen` 名称保存到 `GameState.pending_input`。玩家提交 `choose <编号>` 后，Rust 根据名称重新调用函数；没有保存 Lua closure 或 coroutine。

随机实体也使用同样的命名 continuation：

```lua
ctx:random_entity(ctx:enemy_characters(self), "on_random_target")
```

选择由 Rust 的种子 RNG 完成，并记录 `random_choice_made` 事件。沙箱已删除 `math.random` 和 `math.randomseed`。

卡牌选项和创建卡牌使用：

```lua
ctx:choose_cards(
    ctx:controller(self),
    "选择一张牌",
    { "CS2_029", "CS2_120" },
    "on_card_chosen"
)

ctx:discover_cards(
    ctx:controller(self),
    "发现一张牌",
    candidates,
    3,
    "on_card_chosen"
)

-- 命名 resume hook
function card.on_card_chosen(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end
```

`choose_cards` 展示传入的全部候选项。`discover_cards` 只负责对 Lua 给出的池执行确定性抽样，不会暗中加入职业或类型规则。官方普通发现牌应在 Lua 中组合卡牌定义与玩家职业，例如：

```lua
local player_class = ctx:player(player).class
for _, card_id in ipairs(ctx:collectible_cards()) do
    local card = ctx:card_definition(card_id)
    if card.type == "spell"
        and (card.class == "neutral" or card.class == player_class) then
        candidates[#candidates + 1] = card_id
    end
end
```

随后 `discover_cards` 由 Rust RNG 从去重后的候选池中无放回抽取至多指定数量，再建立 `ChoiceValue::Card` 玩家选择。抽样结果写入 `random_cards_sampled`，包含 `source`、稳定顺序的 `cards` 和去重后 `population`；玩家职业、随机计数、选项和剩余结算队列均能进入 replay/snapshot。候选中的未知卡牌、空牌池或数量 `0` 会使当前命令回滚。

`discover_entities(player, prompt, candidates, count, resume_hook)` 使用实体副本数作为抽样权重，但同一 `card_id` 最多展示一次；候选和回调值是被抽中的稳定实体 ID，并发布 `random_entities_sampled`。它适用于从牌库、手牌或战场中的真实实体里发现一个对象，不会创建定义副本。拍卖师亚克森用它从 `ctx:deck(player)` 发现实体，再调用 `replace_trade_draw`。

需要让每个选项携带复合数据时使用通用选择接口：

```lua
ctx:choose_options(ctx:controller(self), "选择计划", {
    {
        label = "进攻",
        value = {
            target = enemy_hero,
            damage = 2,
            draw = true,
            tags = { "attack", "fast" },
        },
    },
    {
        label = "防守",
        value = { armor = 3, draw = false },
    },
}, "on_plan_chosen")

function card.on_plan_chosen(ctx, self, plan)
    if plan.damage then
        cardlib.effects.damage(ctx, plan.target, plan.damage)
    end
end
```

`value` 可为 `nil`、boolean、有符号整数、UTF-8 字符串、稠密数组或纯字符串键对象，并可递归组合。玩家看到 `label`，resume hook 收到对应 `value`。同类的 `ctx:random_value(values, resume_hook)` 让 Rust RNG 从任意可序列化值数组中选择；它与 `random_entity` 一样增加随机计数并发布 `random_choice_made`。

## 状态稳定后的命名序列

同一个 Lua hook 看到的是调用开始时的只读快照。需要“先执行效果，再根据新状态继续”时，不能在一次 hook 中发出伤害后立刻读取生命值，而应使用命名 continuation：

```lua
function card.on_play(ctx, self, target)
    cardlib.effects.damage(ctx, target, 1)
    ctx:continue_with_entity("after_damage", target)
end

function card.after_damage(ctx, self, target)
    if ctx:entity(target).zone == "graveyard" then
        ctx:draw(ctx:controller(self), 1)
    end
end
```

continuation 本身是可序列化 `EffectSpec`。到达它之前，排在前面的效果、事件触发器、光环和死亡检查点会全部结算；随后引擎按名称重新查找模块函数。continuation 可以继续发出另一个 continuation，hook 名必须为 1–64 字节。

无参数时使用 `continue_with`。兼容接口 `continue_with_entity/card/number` 分别保留实体、卡牌和数值类型；`continue_with_value(hook, value)` 可以保存与 `choose_options` 相同的递归结构，恢复 hook 时作为第三个参数传入。

结构化值会立即复制成 Rust 权威数据，不保留 Lua table、closure 或 coroutine。为保证 snapshot 大小和脚本复杂度有界，单个值最多 16 层、512 个节点和 16 KiB 字符串数据；玩家选择最多 256 项，prompt 最多 4 KiB，单项 label 最多 1 KiB。浮点数、函数、线程、userdata、循环/共享 table、稀疏数组、混合数组/对象或其他键类型会令整条玩家命令事务回滚。空 table 解释为对象；确需复用相同内容时应创建两个独立 table。

## 事件触发器

```lua
triggers = {
    {
        event = "entity_died",
        timing = "after",
        active_zones = { "graveyard" },

        condition = function(ctx, self, event)
            return event.entity == self
        end,

        effect = function(ctx, self, event)
            ctx:draw(ctx:controller(self), 1)
        end,
    },
}
```

`timing` 可为 `before` 或 `after`，省略时默认 `after`。传给 condition/effect 的事件 table 包含：

```text
name, event_id, timing，以及该事件自己的 player/source/target/amount 等字段
```

出牌、弃牌、抽牌、效果召唤、通用区域移动、控制权变化、变形、疲劳、攻击、伤害和治疗拥有前置提交阶段。在 `before` trigger 中可以输出普通效果，也可以取消或替换正在等待提交的事件：

```lua
{
    event = "damaged",
    timing = "before",
    condition = function(ctx, self, event)
        return event.target == self and event.amount > 1
    end,
    effect = function(ctx, self, event)
        -- 二选一：完全取消，或把最终伤害改为 1。
        -- ctx:cancel_event(event)
        cardlib.effects.set_event_amount(ctx, event, 1)
    end,
}
```

`cancel_event` 适用于上述所有 before 时点；不同事件的取消语义由 Rust 统一定义：

- `card_played`：费用已经预留，卡牌进入墓地，不执行 `on_play`，发布 `card_countered`；
- `card_drawn` / `card_burned`：预留卡牌精确放回牌库顶，不发布 after 事件；
- 由 `ctx:summon` 产生的 `minion_summoned`：预留 token 进入 `removed` 区域，不发布 after 事件；
- `zone_changed`：实体留在原区域；若另一个嵌套效果已先移动该实体，较旧的移动自动失效；
- `fatigue`：取消本次疲劳通知和伤害，但 Rust 的疲劳计数仍增长；
- `location_used`：耐久与冷却已经预留；取消能力效果，但不返还这些消耗；
- `attack`、`damaged`、`healed`：取消对应的攻击、伤害或治疗提交。

`cardlib.effects.set_event_amount`、`add_event_amount` 和 `multiply_event_amount` 适用于 `damaged`、`healed` 和 `fatigue`，并按 EffectSpec 队列顺序组合。修改疲劳数值只影响本次伤害，不改写下次疲劳计数。在 after trigger、事件已经提交后或不支持数值替换的事件上调用，会令当前玩家命令失败并事务回滚。

before trigger 也可以调用 `choose_entities`/`choose_cards`。Rust 会把尚未执行的事件提交和战斗动作一起序列化进 `PendingInput`；玩家选择后才继续，而不是让攻击在选择前偷跑。

若省略 `active_zones`，触发器只在该实体位于 `board` 时有效。区域名为：

```text
hero, hero_power, deck, hand, board, weapon, secret, graveyard
```

当前事件名：

```text
game_started, turn_started, turn_ended, card_drawn, card_burned, card_created, fatigue,
card_played, spell_targeted, spell_cast, minion_played, weapon_played, location_played, card_countered, card_discarded, card_traded, trade_draw,
minion_summoned, magnetized, weapon_equipped, weapon_destroyed, location_used, location_destroyed,
hero_power_used, hero_power_replaced, secret_played, secret_revealed, zone_changed, controller_changed, transformed, attack, damaged, damage_prevented, healed,
armor_gained, overload_queued, mana_locked, mana_unlocked, temporary_mana_gained,
temporary_mana_expired, mana_crystals_gained, mana_crystals_destroyed, mana_spent, player_resource_gained, player_resource_spent,
keyword_disabled, frozen, entity_died, conceded, game_ended,
choice_requested, choice_made, random_choice_made, random_cards_sampled, random_entities_sampled
```

事件 table 始终有 `name`，并按事件包含 `player`、`entity`、`source`、`target`、`amount`、`attacker` 或 `defender` 等字段。`card_created` 同时包含新实体 `entity` 与创建它的效果来源 `source`。`card_drawn` 和 `card_burned` 的 `entity` 是被抽取的牌，`source` 是造成这次抽牌的效果实体；自然回合抽牌与起手抽牌为 nil，脚本、英雄技能和交易替换抽牌保留实际来源。`spell_targeted` 在成功通过反制后、法术正文前发布，包含法术 `entity`、声明的 `target`、`generated` 与 `generated_by`；`spell_cast` 在正文后包含同样的字段，无目标时 `target` 为 nil。`keyword_disabled` 包含 `keyword`；`entity_died` 包含 `player` 和该实体被移除时的零基 `position`。`trade_draw` 包含 `player/entity/replacement`；`card_traded` 包含 `player/entity`，在替换抽牌完成且原实体进入牌库后发布。

`damage_prevented` 包含 `source`、`target` 和当前的 `reason = "immune"`，它替代该次伤害的 `damaged/after`。地标不是伤害目标，因此对地标输出伤害效果时不会创建伤害事件。

`game_ended` 的胜者事件包含 `outcome = "winner"` 和 `winner`；双方英雄在同一死亡检查点死亡时，包含 `outcome = "draw"`，没有 `winner`。

`card_played.cost` 是提交出牌命令时冻结的实际卡牌费用，不会被离开费用光环或牌面效果后续消耗法力所改变。

目前 `card_played`、`card_discarded`、`trade_draw`、正常对局中的 `card_drawn/card_burned`、由 `ctx:summon` 产生的 `minion_summoned`、`zone_changed`、`controller_changed`、`transformed`、`fatigue`、`weapon_equipped`、`weapon_destroyed`、`location_used`、`hero_power_used`、`attack`、`damaged` 和普通 `healed` 会发布 `before`；成功提交后发布 `after`。弃牌引起的附带 `zone_changed` 目前只发布 after。`spell_targeted` 是通过反制后的正文前通知；`spell_cast`、`minion_played`、`weapon_played`、`location_played` 是成功出牌后的细分 after 通知，其中 `minion_played` 与效果召唤产生的 `minion_summoned` 不同。被反制的法术只发布 `card_countered`，不会发布 `spell_targeted`、`card_played/after` 或类型细分事件，但仍消耗费用并激活后续连击。初始化起手不会发布 before，直接打出的随从和地标目前只在 `card_played` 阶段可被反制。`card_traded`、`location_destroyed` 与其他列出的事件当前只发布 `after`。

成功提交手牌出牌后，引擎先完整结算卡牌 `on_play` 和关键词 lifecycle hook，再发布 `card_played/after`、类型细分事件及随从的 `minion_summoned/after`。因此战吼效果和战吼内部召唤会先完成，随后才触发原卡牌的 After Play / After Summon 监听器。法术正文同样先于 `spell_cast/after`，英雄技能正文先于 `hero_power_used/after`。

同一事件的监听实体使用 APNAP 顺序：当前玩家控制的实体先触发，非当前玩家随后触发；每组内部按实体 timestamp 排序。每个实体脚本中的多个 trigger 保持 Lua 数组顺序。

同一死亡检查点按实体入场顺序逐一移除致死随从并记录位置；全部移除后，所有 `entity_died` 事件会先写入日志并在同一份稳定状态上收集监听器，然后才执行亡语效果。复生排在该批次的亡语效果之后，因此亡语召出的实体和复生实体都不会倒过来监听本批次尚未发布的死亡。

事件前置阶段可能把卡牌暂存在内部 `set_aside` 区域；被阻止且不应进入墓地的衍生物进入 `removed`。这两个区域会出现在 `ctx:entity` 快照中，但不能作为普通 trigger 的 `active_zones`。

## 脚本约束

- 不要把对局状态保存在 Lua 全局变量或 module table 中；同一份定义会服务多个实体。
- 不要使用 Lua 随机数；使用 `ctx:random_entity(..., "resume_hook")`，由 Rust RNG 选择。
- 不要依赖 table/文件遍历的偶然顺序。
- 一个 hook 最多执行约 200,000 条 VM 指令。
- 单次玩家命令最多解析 10,000 个效果，超过后整条命令回滚。
