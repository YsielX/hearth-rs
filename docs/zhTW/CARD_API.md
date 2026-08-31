# Lua 卡牌 API（版本 1）

[English](../CARD_API.md) | [簡體中文](../zhCN/CARD_API.md) | [繁體中文](CARD_API.md)

Lua 卡牌的 `name` 和 `text` 必須使用英文作為預設後備文字。正式顯示文字由
`data/locales/enUS.json`、`zhCN.json` 和 `zhTW.json` 按官方 ID 覆蓋。

動態提示透過執行時 locale 選擇：

```lua
local prompt = ctx:localize(
    "Discover a spell",
    "發現一張法術牌",
    "發現一張法術牌"
)
```

`ctx.locale` 是隻讀 locale 程式碼；不得用 locale 改變規則、候選池或隨機結果。

每個 `.lua` 檔案必須返回一個 table。檔案載入期間沒有 `io`、`os` 或任意原生模組訪問許可權。

## 後設資料

| 欄位 | 型別 | 必需 | 說明 |
| --- | --- | --- | --- |
| `api_version` | integer | 是 | 當前只能是 `1` |
| `module_type` | string | 否 | 卡牌預設 `card`；獨立英雄技能使用 `hero_power`；關鍵詞模組使用 `keyword` |
| `id` | string | 是 | 卡牌包內唯一 ID |
| `name` | string | 是 | 顯示名稱 |
| `text` | string | 否 | 卡牌文字 |
| `set` | string | 否 | 官方卡牌包程式碼；倉庫內的官方牌必須填寫 |
| `type` | string | 卡牌模組 | `hero`、`minion`、`spell`、`weapon` 或 `location`；英雄技能模組無需填寫 |
| `collectible` | boolean | 否 | 主卡預設 `true`；內嵌 token 預設 `false`，用於動態牌池過濾 |
| `class` | string | 否 | 卡牌職業，預設 `neutral`；Lua 可自行定義命名體系 |
| `rarity` | string | 否 | 印刷稀有度，統一為小寫，用於動態牌池過濾 |
| `spell_school` | string | 否 | 印刷法術派系，統一為小寫，用於動態牌池過濾 |
| `rune_cost` | table | 否 | 印刷死亡騎士符文需求，可填寫 `blood`、`frost`、`unholy` 數量 |
| `tags` | string[] | 否 | 種族或卡牌包自定義標籤，用於 Lua 牌池過濾 |
| `cost` | integer | 是 | 基礎費用 |
| `attack` | integer | 隨從/武器必需 | 隨從或武器的基礎攻擊力 |
| `health` | integer | 隨從/武器/地標必需 | 隨從生命值；武器或地標中表示耐久度 |
| `armor` | integer | 英雄牌必需 | 打出英雄牌時獲得的護甲 |
| `hero_power` | string | 英雄牌必需 | 替換出的獨立英雄技能模組官方 ID |
| `keywords` | string[] | 否 | 對 `data/keywords/*.lua` 模組 ID 的引用；載入時校驗存在性 |
| `keyword_params` | table<string, integer> | 否 | 數值型關鍵詞的靜態引數；鍵必須同時出現在 `keywords` 中 |
| `deck_allowances` | table[] | 否 | 卡牌提供的通用跨職業構築許可；遊客牌必須填寫 |
| `secret` | boolean | 否 | 舊卡包相容欄位；新卡應引用 `keywords = { "secret" }` |
| `target_mode` | string | 否 | `optional`（預設）、`required` 或 `required_if_available`；普通牌控制出牌目標，地標控制啟用目標 |

正式卡包可在卡牌根目錄提供 `locales/enUS.json`、`locales/zhCN.json` 和
`locales/zhTW.json`，每項格式為 `{ "id", "name", "text" }`。載入器按官方 ID
把三語文字合併進不可變 `CardDefinition`；Lua 中的 `name/text` 是缺少 locale
檔案時的後備文字。locale 檔案屬於卡包雜湊的一部分。

目標模式的語義：

- `required`：必須有合法目標並選擇一個；沒有合法目標時不能使用。適合法術和英雄技能；
- `required_if_available`：有合法目標時必須選擇；沒有時仍能打出，效果函式收到 `nil`。適合帶目標的戰吼或連擊隨從；
- `optional`：可以無目標使用；若同時提供目標選擇器，也可主動選擇其中一個目標。

非 `optional` 模式必須同時定義對應的 `targets` 或 `location_targets`。舊欄位
`requires_target = true` 暫時相容並等價於 `target_mode = "required"`，新卡應使用
`target_mode`。

目標 selector 只在玩家宣告動作時驗證。目標選定後使用穩定 `EntityId` 記憶；出牌前置觸發、控制權變化或位置光環不會令 Rust 再次執行 selector。這樣攻擊力條件等即使在戰吼真正執行前發生變化，仍按原目標繼續結算。

## 關鍵詞模組

關鍵詞檔案同樣返回 table，但使用 `module_type = "keyword"`。`rules` 中的函式簽名為 `(ctx, self, current, other) -> value`：`current` 是前一個模組摺疊後的值，`other` 是攻擊者、目標或來源實體（沒有時為 `nil`）。規則函式必須是隻讀的，不能輸出效果。

```lua
return {
    api_version = 1,
    module_type = "keyword",
    id = "windfury",
    name = "風怒",
    rules = {
        max_attacks = function(ctx, self, current, other)
            return math.max(current, 2)
        end,
    },
}
```

關鍵詞也可宣告與卡牌相同格式的 `triggers`，並使用全部 `ctx` 效果 API。比如聖盾透過 `damaged/before` 呼叫 `disable_keyword` 和 `cancel_event`，亡語透過 `entity_died/after` continuation 呼叫卡牌效果，復生則呼叫 `summon_fresh_copy`。完整模組見 `data/keywords/`。

關鍵詞還可在 `hooks` 中實現 `on_play(ctx, self, target)` 或 `on_location_use(ctx, self, target)`，接入卡牌生命週期並輸出普通效果。戰吼模組利用 `hooks.on_play` 轉到卡牌的 `on_battlecry`，連擊模組先檢查 `ctx:combo_active(self)` 再轉到 `on_combo`，壓軸模組則在支付當前費用後剩餘法力為零時轉到 `on_finale`；這是通用模組遍歷，不是 Rust 中的關鍵詞分支。未知 lifecycle hook 或非函式值會在載入時被拒絕。

關鍵詞還可宣告 `actions`，卡牌以 `card_actions` / `action_targets` /
`action_target_modes` / `action_effects` 增加或補充命名動作。鍛造、預備和泰坦能力都走
這一介面；Rust 只處理動作費用、合法目標、事務和 replay，不判斷關鍵詞 ID。關鍵詞的
`required_card_actions` 會在載入時驗證卡牌確實提供對應效果。

關鍵詞需要呼叫每張卡自己的效果時，可以宣告 `required_card_hooks`，並在 lifecycle hook 或事件觸發器裡輸出命名 continuation。例如戰吼要求 `on_battlecry`，連擊要求 `on_combo`，壓軸要求 `on_finale`，法術迸發要求 `on_spellburst`，亡語要求 `on_deathrattle`。載入器會驗證所有引用關鍵詞的卡都提供對應 Lua 函式；Rust 只執行通用 continuation，不認識這些關鍵詞 ID。

數值型關鍵詞可設定 `requires_param = true`。載入器會要求每張引用卡在 `keyword_params` 中提供同名整數，關鍵詞模組用 `ctx:keyword_param(self, keyword_id)` 讀取；多餘的、未被 `keywords` 引用的引數也會被拒絕。例如過載完整邏輯位於 `overload.lua`：

```lua
-- 卡牌定義
keywords = { "overload" },
keyword_params = { overload = 2 },

-- 關鍵詞 hooks.on_play
local amount = ctx:keyword_param(self, "overload")
ctx:overload(ctx:controller(self), amount)
```

關鍵詞還可用 `required_card_fields` 強制卡牌提供任意 Lua 欄位。遊客模組據此要求
`deck_allowances`，牌組驗證器按宣告開放指定職業與卡包，並排除目標職業的遊客牌：

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

普通牌組預設執行本職業/中立限制和這些許可。僅用於規則沙箱的演示牌組可在 JSON 中顯式
設定 `"unrestricted": true`；正式牌組不應使用該開關。

## 同檔案衍生物

主卡可以在 `tokens` 中內嵌任意數量的專屬衍生物。token 的 `api_version` 預設繼承主卡。下面使用官方鬼靈爬行者及其衍生物 ID：

```lua
return {
    api_version = 1,
    id = "FP1_002",
    name = "鬼靈爬行者",
    set = "NAXX",
    type = "minion",
    cost = 2,
    attack = 1,
    health = 2,

    tokens = {
        {
            id = "FP1_002t",
            name = "鬼靈蜘蛛",
            set = "NAXX",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
        },
    },
}
```

主卡和 token 都進入同一個卡牌目錄，並進行全域性 ID 衝突檢查。內嵌 token 預設 `collectible = false`，但可以顯式覆蓋。初始牌組只接受 `collectible = true` 的隨從、法術、武器和地標；不可收集定義仍可由 Lua 建立、召喚或用於變形。因此具有專屬衍生物的新卡仍然只需要一個 Lua 檔案。

標準卡牌包還提供官方不可收集法術 `GAME_005`。雙方完成 Rust 排程階段後，引擎把一個該定義的例項交給 P2；它的正文完全由 Lua 編寫，並按普通法術產生出牌、施法、歷史和臨時法力事件。

卡牌可以提供以下函式：

```lua
targets(ctx, self) -> entity_id[]
on_play(ctx, self, target_or_nil)
on_battlecry(ctx, self, target_or_nil) -- 引用 battlecry 關鍵詞時必需
on_combo(ctx, self, target_or_nil)     -- 引用 combo 關鍵詞時必需
on_finale(ctx, self)                   -- 引用 finale 關鍵詞時必需
location_targets(ctx, self) -> entity_id[]
on_location_use(ctx, self, target_or_nil)
```

`targets/on_play` 用於法術、普通出牌效果與英雄技能；官方戰吼、連擊、壓軸牌分別引用 `battlecry` / `combo` / `finale`，並在對應回撥中實現獨有效果。地標出牌本身始終無目標，其啟用目標和效果分別由 `location_targets/on_location_use` 定義。`self` 和目標都是穩定的整數 `EntityId`，不是可以直接修改的 Rust 物件。

## 只讀查詢

```lua
ctx:controller(entity)          -- 返回 0 或 1
ctx:opponent(player)
ctx:player(player)              -- 返回玩家只讀快照
ctx:cards_played_this_turn(player)
ctx:cards_played_last_turn(player)
ctx:combo_active(entity)        -- 此實體本次從手牌打出時，之前是否已打過牌
ctx:outcast_active(entity)      -- 此實體離手前是否位於最左或最右
ctx:entered_hand_this_turn(entity)
ctx:turn()
ctx:active_player()             -- 目前行動玩家，返回 0 或 1
ctx:cards_played(player)        -- 本局從手牌打出的卡牌 ID；包含被反制的牌
ctx:spells_cast(player)         -- 本局成功施放的法術 ID
ctx:minions_played(player)      -- 本局成功打出的隨從 ID
ctx:minions_summoned(player)    -- 本局成功召喚的隨從 ID（包括打出與效果召喚）
ctx:minions_died(player)        -- 本局死亡隨從的凍結定義 ID
ctx:minions_died_this_turn(player) -- 當前回合死亡隨從的凍結定義 ID
ctx:minion_death_records(player) -- { card_id, turn, had_deathrattle, keywords }
ctx:discarded_cards(player)     -- 按棄牌事件順序記錄的原實體 ID
ctx:discarded_card_ids(player)  -- 按棄牌事件順序凍結的卡牌定義 ID
ctx:starting_deck(player)       -- 凍結的開局牌庫卡牌 ID 多重集
ctx:cards_added_to_hand(player) -- 開局後按事件順序進入手牌的卡牌 ID
ctx:overload_queued_total(player) -- 本局累計超載的水晶數
ctx:hero_was_healed_this_turn(player)
ctx:weapons_played(player)      -- 本局成功打出的武器 ID
ctx:locations_played(player)    -- 本局成功打出的地標 ID
ctx:last_spell_cast(player)     -- 上一張成功完成施放的法術 ID；沒有則為 nil
ctx:hero_power_uses(player)     -- 本局成功使用英雄技能的累計次數
ctx:hand(player)                -- 當前手牌順序
ctx:deck(player)                -- 牌庫頂到牌庫底
ctx:board(player)               -- 戰場從左到右
ctx:secrets(player)             -- 奧秘進入順序
ctx:graveyard(player)           -- 進入墓地順序
ctx:characters()                -- 雙方英雄及戰場隨從
ctx:minions()                   -- 雙方戰場隨從
ctx:enemy_characters(entity)
ctx:enemy_minions(entity)
ctx:friendly_minions(entity)
ctx:adjacent_minions(entity)    -- 按左、右順序返回最多兩個實體
ctx:board_position(entity)      -- 零基位置；不在戰場時返回 nil
ctx:entity(entity)              -- 返回只讀快照 table
ctx:keyword_param(entity, id)   -- 返回卡牌定義上的關鍵詞整數引數；沒有則為 nil
ctx:card_ids()                  -- 當前卡牌包的全部 ID，穩定排序
ctx:collectible_cards()         -- 僅返回 collectible=true 的 ID
ctx:card_definition(card_id)    -- 返回不可變卡牌定義快照
ctx:get_player_data(player, key)
ctx:resource(player, resource_id)
ctx:resource_spent(player, resource_id)
ctx:has_enchantment_from(entity, source)
```

`has_enchantment_from` 判斷實體是否仍保留由指定來源實體產生的附魔，適用於按來源獨立疊加，並在可沉默附魔被移除後失效的持續效果。

實體快照欄位包括：

```text
id, card_id, name, owner, controller, attack, health, max_health, damage, armor, spell_damage, zone, type,
keywords（字串陣列）, silenced, frozen, attacks_this_turn, attack_at_death, started_in_deck, location_cooldown, enchantments,
attached_cards, hook_attachments,
cards_played_before, combo_active
```

玩家快照欄位包括：

```text
id, class, hero, hero_power, hero_power_used, hero_power_uses, hero_power_uses_this_turn, weapon, keywords, resources, resources_spent, mana, max_mana, temporary_mana, overload_pending, overloaded_mana,
cards_played_this_turn, cards_played_this_game, spells_cast_this_game,
minions_played_this_game, minions_summoned_this_game, weapons_played_this_game, locations_played_this_game,
fatigue, deck_size, hand_size, board_size, secret_count, hero_power_used
```

`cards_played_this_turn` 在玩家回合開始時清零。引擎在牌離開手牌並支付費用後增加它，所以被反制的牌仍計入本回合出牌；但當前牌自己的連擊資格使用打出前快照 `cards_played_before`，不會把自己算作前置牌。`combo.lua` 在 lifecycle 階段呼叫 `ctx:combo_active(self)`。目標 selector 發生得更早，帶連擊目標的卡應在 `targets` 中查詢 `ctx:cards_played_this_turn(ctx:controller(self))`，未啟用時返回空陣列。兩種上下文都屬於 Rust 權威狀態，可隨等待選擇、snapshot 和 replay 恢復。

`starting_deck` 在開局後保持凍結，牌被抽取、變形或摧毀都不會改變它；`cards_added_to_hand` 記錄成功抽牌、生成牌和區域移動進入手牌，適合整局消耗或進度規則。

五個歷史查詢返回按發生順序凍結的卡牌定義 ID 陣列，而不是動態反查實體：隨從即使後來變形，歷史中仍保留它打出時的原定義。`cards_played` 在費用支付並離手時記錄，因此反制牌也存在；四個型別歷史在相應 `spell_cast/minion_played/weapon_played/location_played` 成功時記錄。法術自己的 `on_play` 結算期間尚未進入 `spells_cast`，完成正文後才成為 `last_spell_cast`。只有玩家實際施放的法術進入成功施法歷史；由 `cast_spell` 或 `cast_existing_spell` 效果施放的法術帶有 `generated_by`，既不進入該歷史，也不觸發「每當你施放法術」。這些陣列屬於 Rust 權威狀態並進入 snapshot/replay，不應由 Lua 全域性變數代替。

卡牌定義快照包含 `id, name, text, set, type, collectible, class, classes, rarity, spell_school, rune_cost, tags, cost, attack, health, secret, target_mode, requires_target, keywords, keyword_params`。`classes` 用於三職業等多職業牌；`rune_cost = { blood, frost, unholy }` 暴露印刷符文需求，無符文牌的三個值均為 0。其中 `requires_target` 只作為舊指令碼相容欄位；新邏輯應讀取 `target_mode`。例如可以完全在 Lua 中構造動態發現池：

```lua
local candidates = {}
for _, card_id in ipairs(ctx:collectible_cards()) do
    local card = ctx:card_definition(card_id)
    if card.type == "minion" and card.cost == 1 then
        table.insert(candidates, card_id)
    end
end
ctx:choose_cards(ctx:controller(self), "選擇一張1費隨從", candidates, "on_selected")
```

## 效果輸出

這些函式不會在 Lua 中直接改變狀態，只會把效果描述加入本次呼叫的輸出緩衝區。Lua 函式成功返回後，Rust 才會驗證和執行它們。

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
-- hook(ctx, self, spent) 在解析時執行；資源不足 minimum 時 spent 為 0
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
ctx:replace_trade_draw(event_id, replacement_entity)
ctx:continue_with(hook_name)
ctx:continue_with_entity(hook_name, entity)
ctx:continue_with_card(hook_name, card_id)
ctx:continue_with_number(hook_name, number)
ctx:continue_with_value(hook_name, serializable_value)
ctx:set_player_data(player, key, value)
ctx:increment_player_data(player, key, delta)
```

所有效果的 `source` 自動設為當前執行 hook 的卡牌實體。

`cardlib.effects` 是卡牌層的 Lua 便捷庫。每種 Rust 效果只保留一個參數化原語：`create_card`、`give_copy`、`summon`、`summon_existing`、`recruit`、`move`、`transform_into_copy`、`buff`、`summon_copy` 和 `summon_fresh_copy`；位置、目標區域、持續時間、複製狀態和屬性變體都由 Lua 語法糖組合。`summon` 支援 `position`、互斥的 `base_stats`/`final_stats` 和 `keywords`；`give_copy` 支援 `state = "preserve" | "definition"`、`final_stats` 和 `cost`；`move` 透過 `{ player = ... }` 指定目標玩家；`buff` 支援 `attack`、`health`、`keywords` 與 `duration = "permanent" | "end_of_turn"`。fresh-copy 的 `remaining_health` 與 `final_stats` 互斥。批次介面仍用於保證群組操作的原子性。

光環中的 `cost` 是加法層；卡牌文字寫「消耗為（1）」時使用 `cost_set = 1`（也可為函式），需要限制最終消耗時使用 `cost_cap`。消耗光環順序為 `Aura SET → Aura ADD → Aura CAP`。

`spell_damage` 光環可以指向手下或英雄。玩家的法術傷害加成為己方場上手下與英雄所承載數值之和，因此雙方玩家級效果無需任何卡牌特判。

`replace_hero` 要求目標定義為 Hero 且宣告有效的 `hero_power`：新英雄使用定義中的生命上限並回滿生命，保留原英雄的護甲、凍結狀態和本回合攻擊次數，同時替換英雄技能並釋出 `hero_replaced`/`hero_power_replaced`。`grant_player_keyword` 與 `disable_player_keyword` 只管理可執行的玩家級腳本機制；公開展示與規則執行正交，由 `grant_public_player_status` 和 `disable_public_player_status` 管理並投影到雙方視圖與 RL 觀察。

`destroy_all` 在同一個死亡檢查點摧毀所有目標，適用於「摧毀所有手下」一類同時結算；`move` 的目標區域包括 `hand`、`secret`、`deck_top`、`deck_bottom`、`deck_random`、`graveyard` 和 `removed`。移動到 `secret` 時會校驗該實體確實具有奧秘規則且奧秘區未滿。`shuffle_entity_into_deck` 使用 Rust 確定性隨機把原實體洗入指定玩家牌庫，同時轉移 owner/controller 並執行隱藏區重置。

`transform` 允許手牌和牌庫隱藏區中的實體跨卡牌類型原位替換，並保留實體身分與區域順序。`transform_preserving_scripts` 還會保留 `attached_cards` 與指令碼資料；需要跨自身變形持續的規則應先用 `attach_script` 附加可重用模組。`attach_hook` 可向任意命名 Lua 鉤子附加有序、可疊加的卡牌指令碼；沉默會移除手下已有的鉤子附件。

`cast_spell` 從定義建立法術，`cast_existing_spell` 施放隱藏區或終止區中的既有實體；兩者都接受 `{ target = entity, skip_if_invalid = true, random_target = true, choice_policy = "random" }`。隨機目標和自動抉擇現在是顯式策略，不再借用隱藏指令碼資料。連續隨機施法由 Lua 組合，公共庫 `cardlib.random_spell` 重用權威 `random_value` 與 `cast_spell`。

`create_card` 支援 `destination`、可選手牌 `position`、`attack`、`health`、`cost`、`spell_damage`、`keywords`、`attached_scripts`、`public_cards` 和 `started_in_deck`。`public_cards` 只公開描述合成牌，不會執行這些定義的指令碼；`ctx:add_public_card(entity, card_id)` 可追加後續元件。`consume_sideboard_card` 只移除指定身分；指令碼在同一個事務命令中把它與 `create_card(..., { started_in_deck = true })` 組合。屬性合成公式留在 Lua；`cardlib.fusion.create_minion` 是可重用的合成手下實作。宣告 `module_type = "library"` 的檔案會暴露為 `cardlib[id]`，參與校驗和確定性卡包雜湊，但不會註冊成卡牌。

`damage_ignoring_spell_damage` 仍走普通順序傷害與事件流程，但不疊加來源控制者的法術傷害。`spend_mana` 原子地花費玩家當前可用法力（優先臨時法力），並按實際正數花費發布 `mana_spent`。`increment_player_data` 對玩家指令碼資料執行原子有符號累加，發布帶 `old/new/delta` 的 `player_script_data_changed`，避免同一快照收集出的多個觸發器互相覆蓋。死亡記錄保存卡牌定義是否原生具有死亡之聲：沉默不清除此標記，附加死亡之聲也不會設定它。

`give_copy` 用於向前或同區域複製，保留來源實體的永久狀態；`give_copy_with_stats` 再附加最終攻擊、生命及可選消耗定值。`give_base_copy*` 用於戰場到手牌等向後區域複製，只從印刷定義建立無增益副本。

`draw_entity` 從指定玩家牌庫抽取該原實體，並走可取消的普通 CardDrawn/CardBurned 流程。`summon_existing` 把墓地或移除區的原手下送入完整可取消召喚流程，取消或滿場時恢復；`summon_existing_at` 還會使用記錄的原戰場位置。`move_to_hand` 可把原實體轉入指定玩家手牌，`shuffle_copy_into_deck` 會保留被複製實體的狀態。`summon_copy` 會保留牌庫、手牌或戰場中存活手下的執行期狀態；可選的 `final_stats` 會在同一原子操作內給新實體附加可沉默的最終攻血。墓地實體應使用 `summon_fresh_copy`，它預設從卡牌定義建立滿血、無增益的新實例；`remaining_health` 保留印刷生命上限但以受傷狀態進場，`final_stats` 則透過可沉默的最終定值替換顯示攻血。`summon_with_stats` 會附加可沉默的最終攻血；`summon_with_base_stats` 會替換印刷基礎屬性，因此翠玉魔像等動態衍生物不會因沉默恢復。`lose_weapon_durability` 扣除已裝備武器耐久，歸零時走普通可取消的 `weapon_destroyed` 生命週期。`add_attack_collateral` 為待結算攻擊加入同批戰鬥傷害。

`damage_batch` 對凍結目標集原子結算不同傷害值，其忽略法術傷害版本不疊加法強。`modify_all` 對凍結目標組套用相同屬性規格；`modify_batch` 接受逐實體規格，每個屬性操作不同時可傳 `modifiers` 陣列。兩者都支援 `reset_damage = true`。`force_attack` 無需攻擊者處於可攻擊狀態即可發起完整攻擊事件；`take_extra_turn` 為指定玩家排入可回放的額外回合。`grant_keyword_until_next_turn` 在該手下控制者的下回合開始時到期，且不依賴來源實體繼續存在。

區域查詢返回穩定順序的實體 ID 副本，指令碼不能修改 Rust 內部列表。`hand`、`deck` 等介面也允許查詢對手隱藏區；Lua 卡牌是服務端可信規則程式碼，UI 不會直接獲得這些結果。需要向玩家公開或選擇隱藏資訊時，應由卡牌顯式構造 `choose_*` 選項。

`discard` 只對引數玩家當前手牌中的實體生效；實體已被更早效果移走時安全地不做任何事。引擎先發布 `card_discarded/before`，此時目標仍在手牌，因此目標自己的 `active_zones = { "hand" }` 觸發器可以呼叫 `cancel_event`。成功提交後卡牌進入墓地，依次批次釋出 `card_discarded/after` 和 `zone_changed/after`；取消時不釋出二者。事件欄位為 `source`、`player`、`entity`。隨機棄牌應先從 `ctx:hand(player)` 過濾候選，再呼叫 `random_entity` 並在 resume hook 中 `discard`，以保持 Rust RNG、事件日誌和 replay 的確定性。

`cast_spell` 由 Rust 建立一個新的真實法術實體，並用它作為法術傷害、治療和後續事件的來源。普通法術進入墓地，滿足 `enters_secret_zone` Lua 規則的法術進入奧秘/任務區；不支付法力、不產生 `card_played`、不增加連擊計數，也不會觸發只反制手牌出牌的 `card_played/before` 奧秘。法術正文全部結算後釋出 `spell_cast/after` 並進入 `spells_cast` 歷史。該事件額外包含 `generated`、`generated_by` 和宣告的 `target`；直接從手牌施放時前兩者分別為 `false/nil`，效果施放時指向產生它的實體，無目標法術的 `target` 為 `nil`。抽到即施放使用 `cast_existing_spell(self, { skip_if_invalid = true })`，移動並施放同一個實體。

目標法術必須傳入目標，且目標必須透過該法術自身的 Lua `targets` 選擇器以及 Rust 的潛行/免疫過濾；缺失或非法目標會令當前玩家命令事務回滾。無目標法術可省略第四個引數。自動施放非 spell 定義同樣是指令碼錯誤。自動施放奧秘時若奧秘區已滿則不產生任何實體或事件。`CastSpell` 本身可序列化，因此可以位於等待選擇的剩餘結算佇列中，並由 snapshot/replay 恢復。

連續呼叫多次 `damage` 表示依次結算多個傷害效果。需要“同時造成傷害”時使用 `damage_all`：Rust 會去重目標，為每個目標分別釋出 `damaged/before`，然後一次性提交所有未取消的傷害，再以同一份提交後狀態批次釋出 `damaged/after`，最後進入死亡檢查點。

`summon` 把隨從放在戰場最右側；`summon_at` 使用從 `0` 開始的位置，允許值為 `0..當前隨從數`。位置越界會令本次玩家命令事務回滾。如果 `minion_summoned/before` 的效果令戰場縮短，提交時會把原位置收縮到新的最右邊界。

`entity_died` 事件包含控制者與該隨從被移除時的零基 `position`。同一死亡檢查點按入場順序移除隨從，後死者的位置在先死者移除後計算。`deathrattle.lua` 會把該值作為第三個引數傳給 `on_deathrattle(ctx, self, position)`；若要從死亡位置開始放置衍生物，應顯式呼叫 `summon_at(player, card_id, position)`。連續召喚到同一位置時，後一次會插在前一次左側。復生模組也使用該位置恢復實體。

`ctx:summon_copy` 複製效果結算時仍在牌庫、手牌或戰場中的存活手下狀態。副本獲得新的 `EntityId`，繼承當前卡牌定義、傷害、凍結、沉默、已消耗關鍵字、enchantment 和 `script_data`；每個 enchantment 獲得新 ID，原本以自身為來源的 enchantment 會重對映到副本。options 中的 `final_stats` 會在同一個效果內額外附加最終攻血並清除副本傷害，`position` 指定零基戰場位置。光環不作為永久狀態複製，而是在副本入場後按新位置重新計算。副本的擁有者和控制者是引數中的玩家，攻擊次數重置，並遵循普通新召喚手下的休眠/衝鋒規則。`cardlib.effects.summon_copy_at` 和 `summon_copy_with_stats` 只負責構造這些 options，不是額外的 Rust 操作。

複製仍釋出可取消的 `minion_summoned/before` 和成功後的 `after`，不會再次執行被複製手下的 `on_play`、關鍵詞 lifecycle hook 或戰吼。墓地實體不能作為狀態複製模板；復活或其他需要無增益、無傷害實例的效果應使用 `ctx:summon_fresh_copy`，常見參數組合由 `cardlib.effects.summon_fresh_copy*` 包裝。

`recruit` 和 `recruit_at` 從引數玩家的牌庫移動指定隨從實體到戰場，後者使用零基位置。它保留原 `EntityId`，不建立副本、不執行 `on_play`、關鍵詞 lifecycle hook 或戰吼，但會發布普通的 `minion_summoned/before/after`，因此召喚觸發器、取消和光環行為與其他效果召喚一致。常見的隨機招募應組合牌庫查詢、Rust RNG 和命名恢復：

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

目標已不在該玩家牌庫或戰場已滿時安全地不做任何事；牌庫中的非隨從目標屬於指令碼錯誤。引擎在 before 階段把原實體移入內部 `set_aside`，同時儲存原下標及前後實體錨點。取消、before 效果填滿戰場或終局截斷時會按仍存在的錨點恢復其相對順序；等待選擇時該預留資訊隨 snapshot/replay 序列化。

`gain_armor` 為指定玩家的英雄增加護甲併發布 `armor_gained/after`。英雄受到傷害時先扣護甲，再扣生命；`damaged` 的數值和吸血仍按本次實際結算的傷害量計算。治療只恢復生命，不恢復或消耗護甲。

`overload` 是通用的法力效果原語，把數值累加到玩家的 `overload_pending`，併發布 `overload_queued/after`。具體“過載”關鍵詞的出牌時機和引數讀取由 `data/keywords/overload.lua` 定義。該玩家下個回合開始時，待過載轉換為不超過當前最大法力的 `overloaded_mana`，可用法力設為 `max_mana - overloaded_mana`，釋出 `mana_locked/after`，隨後才釋出 `turn_started`。鎖定只持續該回合；新產生的過載繼續進入再下回合債務。

`unlock_mana` 只解鎖當前回合的 `overloaded_mana`，恢復等量可用法力併發布 `mana_unlocked/after`，不會清除 `overload_pending`。數值超過當前鎖定量時只解鎖實際存在的數量。

`clear_overload` 同時清零當前 `overloaded_mana` 和下回合 `overload_pending`，恢復當前被鎖定的可用法力，併發布 `overload_cleared/after`（包含 `locked` 與 `pending`）。熔岩震擊使用這個通用原語；它不會影響該效果結算之後新產生的過載。

`gain_temporary_mana` 增加本回合可用法力與 `temporary_mana`，釋出 `temporary_mana_gained/after`。支付卡牌或英雄技能費用時優先消耗臨時法力；`mana_spent/after` 包含 `player`、支付來源 `source`、總 `amount` 和其中的 `temporary`。0 費動作不釋出支付事件。剩餘臨時法力在 `turn_ended` 觸發及回合末 enchantment 結算後扣除，併發布 `temporary_mana_expired/after`。

交易是玩家行動而不是效果 API。具有 `can_trade = true` 規則的手牌可以花費 1 點法力交易；牌庫為空時不能交易。引擎先把該實體暫存，釋出可響應的 `trade_draw/before`，執行一次普通抽牌，再將原實體插入牌庫的確定性隨機位置，因此不會抽回同一個實體，滿手牌時也不會爆掉替換牌。交易不增加出牌計數或任何出牌歷史，保留原實體上的 enchantment，最終釋出 `card_traded/after`，並完整進入 replay。

`trade_draw/before` 的 `event_id` 可交給 `replace_trade_draw`，把預設牌庫頂抽牌替換成指定牌庫實體；指定實體在提交前離開牌庫時安全回退到普通抽牌。before hook 可以先把事件 ID 儲存到 `script_data`，呼叫 `discover_entities` 暫停結算，再在 resume hook 中選擇替代實體。抽牌完成後釋出 `trade_draw/after`，此時事件的 `replacement` 為實際請求的實體；隨後原交易牌進入牌庫併發布 `card_traded/after`。

`gain_mana_crystals(player, amount, filled)` 最多把 `max_mana` 增至 10；`filled = true` 時同時增加等量當前法力，否則獲得空水晶。`destroy_mana_crystals` 降低最大法力，並將當前永久法力與鎖定量收縮到新容量，不影響臨時法力。對應事件為 `mana_crystals_gained`（額外含 `filled`）和 `mana_crystals_destroyed`。

`move` 支援以下目標位置：

```text
hand, board, secret, deck_top, deck_bottom, deck_random, graveyard, removed
```

移動到手牌、牌庫或戰場時，實體回到擁有者控制，清除傷害、沉默、enchantment 和 `script_data`。`board` 只接受墓地中的手下且不會發布 `minion_summoned`，用於休眠復歸等明確不屬於召喚的效果。手牌已滿時，返回手牌會改為進入墓地。`deck_random` 只隨機選擇插入位置，不擾亂牌庫中其他卡牌的相對順序；隨機結果由 Rust RNG 決定並可透過 replay 精確重建。英雄、英雄技能、`set_aside` 和 `removed` 實體不能再次透過此 API 移動。

`trigger_hook(target, hook)` 在目標實體上呼叫指定的 Lua 生命週期鉤子，按普通效果佇列繼續結算。它適合「觸發一個手下的死亡之聲」這類不伴隨死亡事件的效果；呼叫方仍負責按卡牌規則篩選目標和處理重複次數。

`change_controller` 對戰場手下和秘密生效。目標玩家對應區域已滿、目標已經離場或控制權已經相同時不產生變化；手下成功時移動到新控制者戰場最右側並進入休眠，再由關鍵詞的 `ready_on_summon` 規則決定是否解除休眠；秘密則移動到新控制者的秘密區。該操作釋出可取消的 `controller_changed/before`，提交後釋出 `controller_changed/after`。實體的擁有者 `owner` 不變，之後返回手牌或牌庫仍回到擁有者一方。

`change_controller_until_end_of_turn` 記錄可逆的戰場手下控制權：沉默會立即把手下歸還原控制者，變形會清除歸還標記並讓目前控制權永久化；回合結束時若原方戰場已滿，該手下會被消滅。`refresh_mana_crystals` 只刷新現有且未被超載鎖定的永久水晶；可選數量省略時補滿，並始終保留暫時法力、目前超載和待結算超載。`summon_with_stats` 使用可沉默的最終屬性層，`summon_with_base_stats` 則直接設定召喚物基礎攻血，沉默不會把翠玉魔像等成長衍生物還原。

`transform` 只接受戰場隨從和另一張隨從定義。變形保留實體 ID、擁有者、控制者、戰場位置、休眠狀態和本回合攻擊次數；基礎屬性與卡牌指令碼替換為新定義，並清除傷害、凍結、沉默、enchantment、已消耗關鍵字狀態和 `script_data`。變形不算死亡或召喚，釋出可取消的 `transformed/before`，提交後釋出 `transformed/after`。`transform_all` 對整組套用同一定義，`transform_batch` 對每個實體套用各自定義，兩者都統一提交並只重算一次光環。`transform_into_copy` 複製模板實體完整狀態，再套用可沉默的最終攻血值。

`destroy` 可以消滅戰場隨從、摧毀戰場地標或已裝備武器。隨從進入統一死亡檢查點併發布 `entity_died`；地標立即移入其控制者墓地併發布 `location_destroyed`；武器走可取消的 `weapon_destroyed` 生命週期。其他區域及其他實體型別不會被該效果改變。

`set_health(target, amount)` 不發布治療事件，直接把目前生命值和生命值上限都設為指定數值。它會建立一個可沉默的永久 enchantment，適合表達生命值交換等卡牌敘述。

`buff` 和 `grant_keyword` 會建立可追蹤的永久 enchantment，而不是直接篡改基礎屬性。`silence` 會移除可沉默 enchantment、印刷關鍵字和指令碼觸發能力。

通用屬性修改使用 `modify`：

```lua
cardlib.effects.modify(ctx, target, {
    stat = "attack",             -- attack / health / cost / spell_damage
    operation = "set",           -- set / add / pre_final_add / multiply / final_set
    value = 5,
    duration = "end_of_turn",    -- permanent（預設）/ end_of_turn
    silenciable = true,           -- 預設 true；持續整局規則可設為 false
})
```

沒有 `final_set` 時，永久屬性按 `SET → ADD/PRE_FINAL_ADD → MULTIPLY` 分層。存在 `final_set` 時，最後一層定值成為基準，只繼續套用它之後建立的普通 Set/Add/Multiply；`pre_final_add` 永遠位於該定值之前。即時光環最後套用。回合末 enchantment 在 `turn_ended` 觸發全部結算後統一移除，然後再次進行光環、死亡和勝負檢查。

`ctx:remove_enchantments_from(target, source)` 刪除指定來源建立的全部 enchantment。
它與 `silenciable = false` 組合，可表達水晶核心這類不能被沉默、但隨控制權變化需要移除的持續規則。

## 標準關鍵字

關鍵詞由 Lua 在後設資料或 `grant_keyword` 中引用；具體規則由同 ID 的 Lua 關鍵詞模組執行。Rust 只摺疊 `attack_priority`、`can_be_attacked`、`can_be_targeted_by_enemy`、`can_attack_while_exhausted`、`ready_on_summon`、`max_attacks`、`can_trade` 等通用規則，並執行觸發器輸出的通用效果。

- `immune`：不能成為敵方定向效果或攻擊目標；`damaged/before` 由模組取消；
- `taunt`：存在敵方嘲諷隨從時，其他角色不能成為攻擊目標；
- `charge`：進入戰場的當回合即可攻擊任意合法目標；
- `rush`：進入戰場的當回合只能攻擊隨從，下個己方回合解除限制；
- `windfury`：每回合最多攻擊兩次；
- `divine_shield`：在 `damaged/before` 禁用自身並取消傷害，釋出通用 `keyword_disabled/after`；
- `poisonous`：對隨從造成正數實際傷害後，將其標記為致死；不能穿過聖盾。
- `lifesteal`：按實際正數傷害治療來源控制者的英雄，不超過其已損失生命；在死亡檢查點之前結算。
- `stealth`：敵方不能把潛行隨從選作定向卡牌目標或攻擊目標；隨機和群體效果仍可命中。該隨從發起攻擊時失去潛行。
- `reborn`：隨從第一次死亡後召喚一個全新的同卡牌實體，以 1 點生命回到戰場且不再具有復生；新實體會正常釋出 `minion_summoned`。
- `elusive`：拒絕敵方法術和英雄技能的定向目標查詢。
- `tradeable`：在手牌中透過通用 `can_trade` rule 開放交易行動；不依賴 Rust 關鍵詞名分支。

關鍵詞檔案結構見 [架構說明](ARCHITECTURE.md)。`disable_keyword` 是可用於任意關鍵詞的通用原語；返回手牌或牌庫時，已禁用狀態會隨其他戰場狀態一起重置。再次透過 `grant_keyword` 獲得同一關鍵詞會恢復它。沉默會移除印刷關鍵詞、可沉默 enchantment 提供的關鍵詞和卡牌/關鍵詞觸發能力。

凍結不是可沉默關鍵字，而是角色狀態。`ctx:freeze(target)` 可凍結戰場隨從或英雄併發布 `frozen/after`；被凍結角色不能攻擊，在其控制者結束自己的下一個回合時解凍。返回手牌或牌庫會清除凍結。

武器攻擊的傷害來源實體是英雄。設定 `weapon_inherits_to_hero = true` 的關鍵詞模組會參與當前出鞘武器的英雄規則查詢；關鍵詞事件模組也可在 `weapon` 區監聽英雄作為來源的傷害。

法術傷害的印刷值是引數化 Lua 關鍵詞，而不是 `CardDefinition` 特殊欄位：

```lua
keywords = { "spell_damage" },
keyword_params = { spell_damage = 1 },
```

`spell_damage.lua` 透過通用 `base_spell_damage` 數值規則提供基礎值。Rust 隨後應用 `SET → ADD → MULTIPLY → aura` 屬性分層，並把場上己方隨從最終的 `entity.spell_damage` 加到每個來源型別為 `spell` 的正數傷害效果上。隨從戰吼、英雄技能、攻擊、疲勞和武器傷害不會獲得該加成。`modify(... stat = "spell_damage")` 仍可修改該通用屬性；沉默會移除印刷法強和可沉默 enchantment 提供的法強，外部光環仍按普通光環規則重新計算。

## 地標

地標是 `type = "location"` 的普通可收集卡牌，`health` 表示耐久度。它與隨從共用七個戰場位置，但不是角色：不能攻擊、不能成為攻擊目標，也不會出現在 `characters`、`minions`、`friendly_minions`、`enemy_characters` 或 `adjacent_minions` 的結果中。`ctx:board(player)` 會返回該玩家戰場上的所有隨從和地標，因此指令碼可以檢查 `ctx:entity(id).type` 來篩選地標。

地標打出時不選擇目標，可照常實現 `on_play`。進入戰場後可以立刻免費使用；每次使用消耗 1 點耐久度，並在下一個己方回合保持冷卻，到再下一個己方回合恢復。實體快照中的 `location_cooldown` 在使用後為 `2`，兩個己方回合開始時依次變成 `1`、`0`。耐久耗盡後自動進入墓地；最後一次使用會先清出地標的戰場位置，再執行能力，因此滿場時仍可由該能力召喚隨從。普通傷害、治療、凍結、屬性增益和光環都不能改變地標耐久；需要移除地標時使用 `cardlib.effects.destroy(ctx, target)`。

```lua
return {
    api_version = 1,
    id = "CUSTOM_TRAINING_GROUNDS",
    name = "訓練場",
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

`location_used` 支援 before/after。使用時會在 before 之前預留本次耐久消耗與冷卻；取消事件會保留這些消耗，但不執行 `on_location_use`，也不釋出 after。成功打出與摧毀分別釋出 after-only 的 `location_played`、`location_destroyed`。

## 武器、英雄技能和奧秘

武器使用普通卡牌定義；`attack` 是英雄在自己回合獲得的攻擊力，`health` 是耐久度。對手回合武器處於收起狀態，不向英雄提供攻擊、劇毒或吸血等來源關鍵字，因此敵方隨從攻擊該英雄時不會受到武器反擊。回到持有者回合後武器重新生效。英雄每次主動完成一次攻擊後失去 1 點耐久度，耐久度歸零時武器進入墓地。裝備新武器會摧毀舊武器。

`weapon_equipped` 和 `weapon_destroyed` 均支援 before/after：

- 取消裝備：費用與卡牌仍被消耗，武器進入墓地，仍釋出 `card_played/after` 並執行該武器的 `on_play`；
- 取消替換舊武器的摧毀：舊武器保留，新武器進入墓地；
- 取消耐久歸零的摧毀：若指令碼沒有修復耐久，引擎將其恢復為 1。

英雄技能是獨立 Lua 模組：每個檔案宣告 `module_type = "hero_power"`，載入器自動賦予 `type = "hero_power"` 和 `collectible = false`。它可以實作 `targets` 和 `on_play`，費用、目標與效果複用統一執行階段介面。牌組 JSON 用 `hero_power` 指定技能；省略時使用官方 ID `HERO_08bp`。Rust 只負責目前回合限用一次和支付費用。

英雄牌仍是可收集卡牌模組，使用 `type = "hero"`，並宣告 `health`、`armor` 和替換技能的官方 ID `hero_power`。打出時保留目前生命值和傷害，獲得護甲，替換英雄實體與英雄技能，然後執行 Lua 生命週期鉤子；引擎會釋出 `hero_replaced` 和 `hero_power_replaced`。範例牌組設定：

```json
{
  "name": "自定義牌組",
  "class": "mage",
  "hero_power": "MY_HERO_POWER",
  "cards": ["MY_CARD"]
}
```

`class` 是 1 到 64 位元組的玩家職業標識，省略時為 `mage`。它進入 `PlayerState`、replay 和 snapshot，可由 Lua 透過 `ctx:player(player).class` 查詢。一般 `Game::new*` 建局會限制牌組只能包含本職業、中立、包含本職業的多職業卡，以及 Tourist 等卡牌宣告許可的跨職業卡；規則測試必須明確使用 `Game::new_unrestricted*` 才能混合職業。

`hero_power_used` 支援 before/after。費用與本回合次數會在 before 之前預留；取消事件會保留這些消耗，但不釋出 after，也不執行英雄技能 `on_play`。

奧秘是引用 `secret` 關鍵詞的法術。`secret.lua` 提供 `enters_secret_zone` 規則；打出後實體進入 `secret` 區域，觸發器必須把該區域列入 `active_zones`，觸發時顯式揭示：

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

`attack` 的 `before` 時點表示“攻擊已經宣告、戰鬥傷害尚未發生”。它產生的觸發效果和死亡會先結算；若攻擊者或防禦者已離場或瀕死，本次戰鬥取消。

## 實體指令碼資料

卡牌的計數器和任務進度必須儲存在實體上，不能放進共享的 Lua module table：

```lua
local count = ctx:get_data(self, "counter")
ctx:set_data(self, "counter", count + 1)
```

`script_data` 的值當前為有符號 64 位整數，鍵最長 64 位元組。它屬於 `GameState`，因此支援事務回滾、序列化和 replay。

## 持續光環

光環由 Lua 宣告來源生效區域、目標和修正值，Rust 在抽牌、出牌、區域、沉默或 enchantment 變化後重新計算。省略 `active_zones` 時預設只在戰場生效：

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

`attack`、`health`、`cost` 和 `spell_damage` 可以是固定整數，也可以是隻讀函式 `(ctx, self) -> integer`。例如一張在手牌中根據當前手牌數動態減費的隨從可以完全由 Lua 定義：

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

`active_zones` 可使用與觸發器相同的 `hero/hero_power/deck/hand/board/weapon/secret/graveyard`。來源進入內部 `set_aside` 或 `removed` 時永不生效；被沉默的來源也不產生光環。目標選擇器和動態數值函式都是隻讀的，嘗試輸出效果會被視為指令碼錯誤並回滾玩家命令。

一次重算會先移除全部舊光環，基於無光環的同一份穩定狀態收集所有 selector 和動態數值，再按目標聚合，最後統一應用並夾取屬性範圍。因此 `-2` 與 `+2` 不會因為來源建立順序不同而得到不同結果。費用在永久 enchantment 的 `SET → ADD → MULTIPLY` 之後加上光環總和並夾到 `0..255`；攻擊、生命和法強同樣在自身 enchantment 層之後應用。

## 玩家選擇和確定性隨機

需要暫停結算等待玩家輸入時，傳入候選實體和一個模組中的命名回撥：

```lua
local card = {
    api_version = 1,
    id = "CUSTOM_CHOICE",
    name = "自定義抉擇",
    type = "spell",
    cost = 1,
}

function card.on_play(ctx, self, target)
    ctx:choose_entities(
        ctx:controller(self),
        "選擇目標",
        ctx:enemy_characters(self),
        "on_target_chosen"
    )
end

function card.on_target_chosen(ctx, self, choice)
    cardlib.effects.damage(ctx, choice, 2)
end

return card
```

引擎會把選項、來源實體和 `on_target_chosen` 名稱儲存到 `GameState.pending_input`。玩家提交 `choose <編號>` 後，Rust 根據名稱重新呼叫函式；沒有儲存 Lua closure 或 coroutine。

隨機實體也使用同樣的命名 continuation：

```lua
ctx:random_entity(ctx:enemy_characters(self), "on_random_target")
```

選擇由 Rust 的種子 RNG 完成，並記錄 `random_choice_made` 事件。沙箱已刪除 `math.random` 和 `math.randomseed`。

卡牌選項和建立卡牌使用：

```lua
ctx:choose_cards(
    ctx:controller(self),
    "選擇一張牌",
    { "CS2_029", "CS2_120" },
    "on_card_chosen"
)

ctx:discover_cards(
    ctx:controller(self),
    "發現一張牌",
    candidates,
    3,
    "on_card_chosen"
)

-- 命名 resume hook
function card.on_card_chosen(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end
```

`choose_cards` 展示傳入的全部候選項。`discover_cards` 只負責對 Lua 給出的池執行確定性抽樣，不會暗中加入職業或型別規則。官方普通發現牌應在 Lua 中組合卡牌定義與玩家職業，例如：

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

隨後 `discover_cards` 由 Rust RNG 從去重後的候選池中無放回抽取至多指定數量，再建立 `ChoiceValue::Card` 玩家選擇。抽樣結果寫入 `random_cards_sampled`，包含 `source`、穩定順序的 `cards` 和去重後 `population`；玩家職業、隨機計數、選項和剩餘結算佇列均能進入 replay/snapshot。候選中的未知卡牌、空牌池或數量 `0` 會使當前命令回滾。

`discover_entities(player, prompt, candidates, count, resume_hook)` 使用實體副本數作為抽樣權重，但同一 `card_id` 最多展示一次；候選和回撥值是被抽中的穩定實體 ID，併發布 `random_entities_sampled`。它適用於從牌庫、手牌或戰場中的真實實體裡發現一個物件，不會建立定義副本。拍賣師亞克森用它從 `ctx:deck(player)` 發現實體，再呼叫 `replace_trade_draw`。

需要讓每個選項攜帶複合資料時使用通用選擇介面：

```lua
ctx:choose_options(ctx:controller(self), "選擇計劃", {
    {
        label = "進攻",
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

`value` 可為 `nil`、boolean、有符號整數、UTF-8 字串、稠密陣列或純字串鍵物件，並可遞迴組合。玩家看到 `label`，resume hook 收到對應 `value`。同類的 `ctx:random_value(values, resume_hook)` 讓 Rust RNG 從任意可序列化值陣列中選擇；它與 `random_entity` 一樣增加隨機計數併發布 `random_choice_made`。

官方「抉擇」類分支應把每個選項定義成非收藏卡，並透過 `card_id` 而不是不透明的 `value` 提交：

```lua
ctx:choose_options(player, "選擇一個", {
    { card_id = "EX1_164a", label = "獲得兩個法力水晶" },
    { card_id = "EX1_164b", label = "抽三張牌" },
}, "chosen")
```

這樣公開觀察會得到 `ChoiceValue::Card`，客戶端和訓練策略都能讀取選項牌定義；resume hook 收到卡牌 ID 字串。選擇選項不會建立、打出或施放該選項牌，仍由母牌的 continuation 結算對應分支。

選項也可以同時提供 `card_id` 和 `value`：卡牌定義負責公開語義，resume hook 仍收到任意結構 payload。可選的 `card_ids = { ... }` 會補充更多公開語義卡，例如同時描述「回溯」按鈕以及將被替換的手下。

## 狀態穩定後的命名序列

同一個 Lua hook 看到的是呼叫開始時的只讀快照。需要“先執行效果，再根據新狀態繼續”時，不能在一次 hook 中發出傷害後立刻讀取生命值，而應使用命名 continuation：

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

continuation 本身是可序列化 `EffectSpec`。到達它之前，排在前面的效果、事件觸發器、光環和死亡檢查點會全部結算；隨後引擎按名稱重新查詢模組函式。continuation 可以繼續發出另一個 continuation，hook 名必須為 1–64 位元組。

無引數時使用 `continue_with`。相容介面 `continue_with_entity/card/number` 分別保留實體、卡牌和數值型別；`continue_with_value(hook, value)` 可以儲存與 `choose_options` 相同的遞迴結構，恢復 hook 時作為第三個引數傳入。

結構化值會立即複製成 Rust 權威資料，不保留 Lua table、closure 或 coroutine。為保證 snapshot 大小和指令碼複雜度有界，單個值最多 16 層、512 個節點和 16 KiB 字串資料；玩家選擇最多 256 項，prompt 最多 4 KiB，單項 label 最多 1 KiB。浮點數、函式、執行緒、userdata、迴圈/共享 table、稀疏陣列、混合陣列/物件或其他鍵型別會令整條玩家命令事務回滾。空 table 解釋為物件；確需複用相同內容時應建立兩個獨立 table。

## 事件觸發器

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

`timing` 可為 `before` 或 `after`，省略時預設 `after`。傳給 condition/effect 的事件 table 包含：

```text
name, event_id, timing，以及該事件自己的 player/source/target/amount 等欄位
```

出牌、棄牌、抽牌、效果召喚、通用區域移動、控制權變化、變形、疲勞、攻擊、傷害和治療擁有前置提交階段。在 `before` trigger 中可以輸出普通效果，也可以取消或替換正在等待提交的事件：

```lua
{
    event = "damaged",
    timing = "before",
    condition = function(ctx, self, event)
        return event.target == self and event.amount > 1
    end,
    effect = function(ctx, self, event)
        -- 二選一：完全取消，或把最終傷害改為 1。
        -- ctx:cancel_event(event)
        cardlib.effects.set_event_amount(ctx, event, 1)
    end,
}
```

`cancel_event` 適用於上述所有 before 時點；不同事件的取消語義由 Rust 統一定義：

- `card_played`：費用已經預留，卡牌進入墓地，不執行 `on_play`，釋出 `card_countered`；
- `card_drawn` / `card_burned`：預留卡牌精確放回牌庫頂，不釋出 after 事件；
- 由 `ctx:summon` 產生的 `minion_summoned`：預留 token 進入 `removed` 區域，不釋出 after 事件；
- `zone_changed`：實體留在原區域；若另一個巢狀效果已先移動該實體，較舊的移動自動失效；
- `fatigue`：取消本次疲勞通知和傷害，但 Rust 的疲勞計數仍增長；
- `location_used`：耐久與冷卻已經預留；取消能力效果，但不返還這些消耗；
- `attack`、`damaged`、`healed`：取消對應的攻擊、傷害或治療提交。

`cardlib.effects.set_event_amount`、`add_event_amount` 和 `multiply_event_amount` 適用於 `damaged`、`healed` 和 `fatigue`，並按 EffectSpec 佇列順序組合。修改疲勞數值隻影響本次傷害，不改寫下次疲勞計數。在 after trigger、事件已經提交後或不支援數值替換的事件上呼叫，會令當前玩家命令失敗並事務回滾。

before trigger 也可以呼叫 `choose_entities`/`choose_cards`。Rust 會把尚未執行的事件提交和戰鬥動作一起序列化進 `PendingInput`；玩家選擇後才繼續，而不是讓攻擊在選擇前偷跑。

若省略 `active_zones`，觸發器只在該實體位於 `board` 時有效。區域名為：

```text
hero, hero_power, deck, hand, board, weapon, secret, graveyard
```

當前事件名：

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

事件 table 始終有 `name`，並按事件包含 `player`、`entity`、`source`、`target`、`amount`、`attacker` 或 `defender` 等欄位。`card_created` 同時包含新實體 `entity` 與建立它的效果來源 `source`。`card_drawn` 和 `card_burned` 的 `entity` 是被抽取的牌，`source` 是造成這次抽牌的效果實體；自然回合抽牌與起手抽牌為 nil，指令碼、英雄技能和交易替換抽牌保留實際來源。`spell_targeted` 在成功通過反制後、法術正文前釋出，包含法術 `entity`、宣告的 `target`、`generated` 與 `generated_by`；`spell_cast` 在正文後包含相同欄位，無目標時 `target` 為 nil。`keyword_disabled` 包含 `keyword`；`entity_died` 包含 `player` 和該實體被移除時的零基 `position`。`trade_draw` 包含 `player/entity/replacement`；`card_traded` 包含 `player/entity`，在替換抽牌完成且原實體進入牌庫後釋出。

`damage_prevented` 包含 `source`、`target` 和當前的 `reason = "immune"`，它替代該次傷害的 `damaged/after`。地標不是傷害目標，因此對地標輸出傷害效果時不會建立傷害事件。

`game_ended` 的勝者事件包含 `outcome = "winner"` 和 `winner`；雙方英雄在同一死亡檢查點死亡時，包含 `outcome = "draw"`，沒有 `winner`。

`card_played.cost` 是提交出牌命令時凍結的實際卡牌費用，不會被離開費用光環或牌面效果後續消耗法力所改變。

目前 `card_played`、`card_discarded`、`trade_draw`、正常對局中的 `card_drawn/card_burned`、由 `ctx:summon` 產生的 `minion_summoned`、`zone_changed`、`controller_changed`、`transformed`、`fatigue`、`weapon_equipped`、`weapon_destroyed`、`location_used`、`hero_power_used`、`attack`、`damaged` 和普通 `healed` 會發布 `before`；成功提交後釋出 `after`。棄牌引起的附帶 `zone_changed` 目前只發布 after。`spell_targeted` 是通過反制後的正文前通知；`spell_cast`、`minion_played`、`weapon_played`、`location_played` 是成功出牌後的細分 after 通知，其中 `minion_played` 與效果召喚產生的 `minion_summoned` 不同。被反制的法術只發布 `card_countered`，不會發布 `spell_targeted`、`card_played/after` 或型別細分事件，但仍消耗費用並啟用後續連擊。初始化起手不會發布 before，直接打出的隨從和地標目前只在 `card_played` 階段可被反制。`card_traded`、`location_destroyed` 與其他列出的事件當前只發布 `after`。

成功提交手牌出牌後，引擎先完整結算卡牌 `on_play` 和關鍵詞 lifecycle hook，再發布 `card_played/after`、型別細分事件及隨從的 `minion_summoned/after`。因此戰吼效果和戰吼內部召喚會先完成，隨後才觸發原卡牌的 After Play / After Summon 監聽器。法術正文同樣先於 `spell_cast/after`，英雄技能正文先於 `hero_power_used/after`。

同一事件的監聽實體使用 APNAP 順序：當前玩家控制的實體先觸發，非當前玩家隨後觸發；每組內部按實體 timestamp 排序。每個實體指令碼中的多個 trigger 保持 Lua 陣列順序。

同一死亡檢查點按實體入場順序逐一移除致死隨從並記錄位置；全部移除後，所有 `entity_died` 事件會先寫入日誌並在同一份穩定狀態上收集監聽器，然後才執行亡語效果。復生排在該批次的亡語效果之後，因此亡語召出的實體和復生實體都不會倒過來監聽本批次尚未釋出的死亡。

事件前置階段可能把卡牌暫存在內部 `set_aside` 區域；被阻止且不應進入墓地的衍生物進入 `removed`。這兩個區域會出現在 `ctx:entity` 快照中，但不能作為普通 trigger 的 `active_zones`。

## 指令碼約束

- 不要把對局狀態儲存在 Lua 全域性變數或 module table 中；同一份定義會服務多個實體。
- 不要使用 Lua 隨機數；使用 `ctx:random_entity(..., "resume_hook")`，由 Rust RNG 選擇。
- 不要依賴 table/檔案遍歷的偶然順序。
- 一個 hook 最多執行約 200,000 條 VM 指令。
- 單次玩家命令最多解析 10,000 個效果，超過後整條命令回滾。
