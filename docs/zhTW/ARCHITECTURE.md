# 架構說明

[English](../ARCHITECTURE.md) | [簡體中文](../zhCN/ARCHITECTURE.md) | [繁體中文](ARCHITECTURE.md)

## 設計約束

1. Rust 是唯一權威狀態，不允許 Lua 持有可變遊戲物件。
2. 卡牌 ID、卡牌邏輯和具體關鍵詞語義不進入 Rust 分支。
3. Lua 只能讀取快照並輸出效果描述；Rust 驗證後原子提交。
4. 同一種子、卡牌包指紋和命令序列必須產生相同狀態。
5. 新卡能組合已有規則鉤子與效果原語時，只增加 Lua 檔案。

## 分層

```text
CLI / 未來網路協議
        │ PlayerCommand / legal_actions
        ▼
Rust Game 狀態機
  ├─ 區域與實體不變數
  ├─ 法力、回合、攻擊、傷害和死亡檢查點
  ├─ PendingEvent / ResolutionItem 佇列
  ├─ 確定性 RNG、事務回滾、replay、snapshot
  └─ 通用 keyword rule 查詢
        │ 只讀 GameState + hook 引數
        ▼
LuaCardRuntime
  ├─ 卡牌模組：targets / on_play / on_battlecry / triggers / auras
  ├─ 英雄技能模組：每個技能一個獨立載入模組
  ├─ 關鍵詞模組：rules / hooks / triggers
  └─ 通用 ctx 查詢與 EffectSpec 輸出
        │
        ▼
data/sets/**/*.lua + data/hero_powers/**/*.lua + data/keywords/*.lua
```

## 三類 Lua 模組

卡牌模組預設 `module_type = "card"`，也可以省略該欄位。它包含官方後設資料和卡牌鉤子。

英雄技能模組宣告 `module_type = "hero_power"`。載入器自動賦予不可收集的 `hero_power` 型別，模組負責費用、目標、`on_play`、觸發器、衍生物和關鍵詞引用。英雄牌仍是 `type = "hero"` 的卡牌模組，並宣告 `armor` 和經過校驗的 `hero_power` ID。

關鍵詞模組顯式宣告：

```lua
return {
    api_version = 1,
    module_type = "keyword",
    id = "taunt",
    name = "嘲諷",
    rules = {
        attack_priority = function(ctx, self, current, attacker)
            return math.max(current, 1)
        end,
    },
}
```

載入結束後，執行時會驗證每張卡引用的關鍵詞 ID 都存在。關鍵詞原始檔和卡牌原始檔共同進入卡牌包雜湊，所以修改任一規則都會令舊 replay 拒絕載入。

關鍵詞的 `hooks` 可以接入通用生命週期入口，目前支援 `on_play` 和 `on_location_use`。關鍵詞還可透過 `actions` 宣告手牌或戰場上的命名玩家動作，例如鍛造、預備與三項泰坦能力。`battlecry.lua` 的 `hooks.on_play` 輸出一個命名 continuation，轉到卡牌的 `on_battlecry`；`combo.lua` 先查詢凍結的出牌前上下文，再按條件轉到 `on_combo`；`finale.lua` 在付費後檢查剩餘法力，再轉到 `on_finale`。Rust 只遍歷實體當前的關鍵詞模組並呼叫通用入口，不檢查這些關鍵詞字串。`required_card_hooks` 和 `required_card_actions` 會在載入期驗證卡牌側契約。

關鍵詞模組可用 `requires_param = true` 宣告數值契約。卡牌以 `keyword_params = { keyword_id = value }` 提供靜態整數，Lua 透過通用 `ctx:keyword_param` 查詢。載入器只驗證引用關係和必需引數，數值含義仍屬於關鍵詞 Lua：例如 `overload.lua` 決定在 `on_play` 讀取引數並輸出 `Overload` 效果，Rust 沒有 `overload` 關鍵詞分支。

## 規則摺疊而不是關鍵詞分支

Rust 在需要做規則決策時詢問通用規則名：

| rule | 初始值 | 用途 |
| --- | ---: | --- |
| `attack_priority` | `0` | 高優先順序目標遮蔽低優先順序目標 |
| `can_be_attacked` | `true` | 目標能否被攻擊 |
| `can_be_targeted_by_enemy` | `true` | 能否成為敵方定向效果目標 |
| `can_attack_while_exhausted` | `false` | 新入場且休眠時能否攻擊某目標 |
| `ready_on_summon` | `false` | 入場時是否解除休眠 |
| `max_attacks` | `1` | 每回合最大攻擊次數 |
| `can_trade` | `false` | 手牌實體是否開放“交易”玩家行動 |
| `can_play` | `true` | 當前實體是否可從手牌打出 |
| `can_attack` | `true` | 當前角色是否可主動攻擊 |
| `can_be_targeted` | `true` | 是否可成為任一方定向效果目標 |
| `enters_secret_zone` | `false` | 法術結算後是否進入持久任務/奧秘區 |
| `starts_in_opening_hand` | `false` | 是否強制進入起手 |
| `hero_power_is_passive` | `false` | 英雄技能是否禁止主動使用 |
| `can_magnetize` | `false` | 手牌隨從是否開放相鄰機械合體放置 |
| `base_spell_damage` | `0` | 為通用法強屬性分層提供印刷基礎值 |

實體的所有有效關鍵詞模組按穩定順序摺疊當前值。這樣 Rust 只認識規則介面，不認識 `taunt` 或 `rush`。未來的關鍵詞只要能組合這些規則和事件觸發器，就不需要 Rust 改動。

武器的關鍵詞模組仍屬於武器實體。需要修改英雄攻擊規則的模組顯式設定 `weapon_inherits_to_hero = true`（例如風怒），規則查詢才會組合到當前出鞘武器；傷害後的關鍵詞觸發器則由武器監聽事件並檢查傷害來源是否為其英雄。

## 事件關鍵詞

聖盾、免疫、潛行、劇毒、吸血、亡語和復生不是傷害或死亡函式中的硬編碼：

- 聖盾監聽 `damaged/before`，輸出 `disable_keyword` 和 `cancel_event`；
- 免疫監聽 `damaged/before` 並取消事件，同時透過規則鉤子阻止敵方選中；
- 潛行在自身 `attack/after` 時禁用；
- 劇毒監聽自身造成的 `damaged/after` 並輸出通用 `destroy`；
- 吸血監聽傷害並輸出通用 `heal`；
- 亡語監聽墓地中的自身 `entity_died/after`，透過 continuation 呼叫卡牌的 `on_deathrattle`；
- 復生監聽墓地中的 `entity_died/after`，呼叫 `summon_fresh_copy`，指定 1 點生命並排除 `reborn`。

戰吼、連擊與壓軸由 lifecycle keyword 驅動：`battlecry.lua` 在出牌階段把已宣告目標傳給卡牌的 `on_battlecry`；`combo.lua` 僅在當前牌不是本回合第一張手牌時呼叫 `on_combo`；`finale.lua` 僅在本次付費後剩餘法力為零時呼叫 `on_finale`。法術迸發監聽己方的 `spell_cast/after`，先禁用自身關鍵詞，再透過可序列化 continuation 呼叫 `on_spellburst`；亡語用同一機制把死亡位置傳給 `on_deathrattle`。關鍵詞模組用 `required_card_hooks` 宣告契約，載入卡包時會拒絕只引用關鍵詞卻沒有實現效果函式的卡牌。卡牌檔案因此只寫該牌獨有的效果，不再重複觸發條件、時序與一次性狀態。

數值型 lifecycle 關鍵詞使用同一模型：`overload.lua` 讀取卡牌的 `keyword_params.overload` 並輸出通用法力效果。土元素本身沒有 `on_play`，召喚效果也不會誤觸發過載；只有從手牌成功打出時才進入 lifecycle hook。熔岩震擊則組合傷害和通用 `clear_overload` 原語，同時清除當前與待生效過載。

持續數值關鍵詞也使用規則摺疊：`spell_damage.lua` 讀取 `keyword_params.spell_damage` 並提供 `base_spell_damage`。Rust 不解析該關鍵詞 ID，只把規則結果作為通用法強屬性的基礎值，再應用 enchantment 與光環層。沉默移除關鍵詞後下一次重算自然回到零。

Rust 為此提供的都是通用原語：禁用任意關鍵詞、取消任意待提交事件、召喚定義的新鮮副本並排除任意關鍵詞列表。沒有原語檢查具體關鍵詞名。

## 事件與結算

一次會被響應的動作先建立 `PendingEvent`：

```text
建立 before 事件
  → APNAP 收集卡牌觸發器和關鍵詞觸發器
  → 依次執行其 EffectSpec
  → 提交或取消事件
  → 寫入日誌
  → 釋出 after
  → Death Checkpoint
```

Lua hook 不直接改變 `GameState`。例如 `ctx:damage(target, 3)` 只輸出 `EffectSpec::Damage`。這樣每個成功玩家命令可以在臨時狀態中完成；任何 Lua 錯誤、非法目標或不變數失敗都會回滾整個命令。

同時傷害使用一組待提交事件：先收集全部 `before`，再提交未取消項，然後基於同一提交後狀態釋出 `after`，最後統一進入死亡檢查點。

## 目標策略

目標候選始終由 Lua 的 `targets` 或 `location_targets` 生成，再經過 Rust 的通用合法性過濾。卡牌用 `target_mode` 宣告選擇策略：

- `required`：沒有合法目標就不能使用，適合法術和英雄技能；
- `required_if_available`：有目標必須選擇、無目標仍可打出，適合帶目標戰吼；
- `optional`：允許無目標，也允許從候選中選擇一個。

因此 Rust 不需要知道哪張牌是戰吼或連擊牌，也沒有卡牌 ID 特判。`required_if_available` 的 `on_battlecry` / `on_combo` 必須處理 `target == nil`。合法性只在玩家宣告目標時檢查；隨後觸發器、控制權或位置光環即使改變目標屬性，也不會重新執行 Lua selector。結算仍使用已宣告的穩定 `EntityId`，具體效果原語再根據實體結算時所在區域決定是否生效。

手牌出牌進入戰場或墓地後，先完整結算 `on_play` 與關鍵詞 lifecycle 效果，再依次釋出 `card_played`、型別細分事件以及隨從的 `minion_summoned` after 通知。因此戰吼召出的衍生物會先完成自己的召喚序列，隨後才進入原隨從的 After Play / After Summon 階段。

## 關鍵詞開放玩家行動

`tradeable.lua` 不在 Rust 中註冊關鍵詞名，而是把通用布林規則 `can_trade` 摺疊為 `true`。`legal_actions` 因此為手牌實體增加 `TradeCard`：花費 1 點法力，先抽取一個不同實體，再把原實體插入確定性隨機牌庫位置。交易不算出牌，保留原實體及其 enchantment，並作為 replay 命令序列的一部分。

預設抽牌被建模為 `trade_draw/before → CardDrawn → trade_draw/after` 子流程。Lua 觸發器可以暫停 before 階段，用 `discover_entities` 從真實牌庫實體中抽樣，再用 `replace_trade_draw` 修改仍在佇列中的通用事件。拍賣師亞克森完全由這三個介面組合，核心沒有其卡牌 ID 或“發現”業務分支。

## 死亡與復生位置

Rust 死亡檢查點先按入場順序識別所有致死隨從，再逐一移除。每個隨從的 `position` 在它實際移除時記錄，因此同批中較早死亡的隨從不會繼續佔據較晚死亡隨從的位置。全部移入墓地後再批次釋出 `entity_died`；Rust 不檢查復生或任何具體亡語。

`deathrattle.lua` 把事件的 `position` 傳給卡牌的 `on_deathrattle`，卡牌可據此呼叫 `summon_at`；`reborn.lua` 則輸出通用新鮮副本召喚效果。兩者仍經過可取消的 `minion_summoned/before/after`，戰場已滿時安全失敗。

## 資料和狀態

卡牌靜態資料來自 Lua `CardDefinition`，包括關鍵詞引用和通用數值引數；執行中每一個例項是 Rust `Entity`。實體儲存基礎屬性、傷害、區域、控制者、enchantment、已禁用關鍵詞、凍結狀態、攻擊次數和可序列化 `script_data`。玩家職業同樣屬於 `PlayerState`，Lua 只能讀取，不能在全域性變數中偽造。

奧秘、任務和任務線分別引用 `secret.lua`、`quest.lua`、`questline.lua`；這些模組提供通用 `enters_secret_zone` 規則，核心只詢問規則名。`CardDefinition.secret` 僅為舊卡包相容欄位，新 Lua 檔案不應再使用它。

## 沙箱和確定性

- 刪除 `dofile/loadfile/require/package/io/os/debug`；
- 刪除 `math.random/randomseed`；
- Lua 記憶體上限 16 MiB；
- 單次 hook 指令預算 200,000；
- 隨機與發現都由 Rust 種子 RNG 執行；
- 原始檔相對路徑和內容進入穩定卡牌包雜湊；
- replay 儲存初始牌組、玩家職業、英雄技能、種子、卡牌包雜湊和成功命令；
- snapshot 內嵌 replay，並透過重放逐欄位驗證權威狀態。

## 擴充套件原則

新增卡牌時優先組合現有 `ctx` API。新增關鍵詞時優先組合通用 rules 與 triggers。只有出現無法由現有邊界表達、且確實適用於多張卡的基礎規則時，才向 Rust 增加新的通用 rule 或原子 EffectSpec；禁止新增 `if card_id == ...` 或 `if keyword == ...` 的業務分支。
