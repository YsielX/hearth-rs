# hearth-rs

[English](README.md) | [簡體中文](README.zhCN.md) | [繁體中文](README.zhTW.md)

一個 Rust 權威規則核心 + Lua 卡牌/關鍵詞規則層的爐石命令列原型。

核心目標是：新增卡牌只增加 Lua 檔案，不註冊 Rust ID，也不在 Rust 中為卡名或關鍵詞寫分支。

## 當前結構

```text
data/
├── hero_powers/           # 每個英雄技能一個獨立 Lua 模組
├── hearthstonejson/       # 已實現定義的官方後設資料來源快照
├── keywords/              # 嘲諷、聖盾、突襲等獨立 Lua 模組
├── locales/               # enUS / zhCN / zhTW 官方名稱與正文
└── sets/                  # 按 HearthstoneJSON set 存放的官方卡牌
crates/
├── hearth-core/           # 狀態機、區域、事件佇列、確定性 RNG、replay
├── hearth-script/         # Lua 沙箱、模組載入、規則鉤子與效果橋接
└── hearth-cli/            # 雙人熱座命令列
decks/demo.json            # 官方卡演示牌組
decks/quest_rogue.json     # Dog 2017 經典洞穴任務賊
```

Rust 負責不能交給指令碼隨意修改的原子能力：實體身份、區域容器、法力支付、攻擊/傷害提交、死亡檢查點、效果佇列、輸入暫停、確定性隨機、事務回滾與 replay。

Lua 負責卡牌語義和關鍵詞語義：目標選擇、戰吼、亡語、奧秘、發現、觸發條件、觸發效果，以及攻擊規則修飾。Rust 引擎裡不再按 `"taunt"`、`"divine_shield"`、`"reborn"` 等字串執行具體規則。

## 官方卡牌資料

目前倉庫保留 150 個官方卡牌、衍生物和英雄技能定義，覆蓋 43 個 set；其中包括 11 個基礎職業英雄技能，以及冰封王座九張英雄牌和它們替換出的英雄技能，並完整實作一套經典洞穴任務賊。這仍是代表性規則語料，不是完整官方卡池。

名稱、正文、數值、官方 ID 和 set 來自 HearthstoneJSON 的客戶端資料。預設英文來源快照位於 [data/hearthstonejson/selected.enUS.json](data/hearthstonejson/selected.enUS.json)，三語顯示文字位於 `data/locales/`，取數說明見[繁體中文取數文件](data/hearthstonejson/zhTW/README.md)。不可收集衍生物也使用官方 ID，例如鬼靈蜘蛛 `FP1_002t`，不再使用自造的 `TOKEN_*` ID。

## 三語文字

CLI 的 `--locale` 接受 `enUS`、`zhCN` 和 `zhTW`，未指定時預設英文。它會切換卡牌名稱、正文、幫助、狀態標籤、事件、錯誤以及 Lua 動態選項提示；命令本身保持穩定的英文關鍵字，便於 replay 和指令碼複用。牌組名是使用者隨意填寫的後設資料，始終原樣顯示唯一的 `name` 值，不參與 locale。

```bash
cargo run -p hearth-cli -- --locale zhTW
cargo run -p hearth-cli -- --locale enUS
```

卡牌 Lua 中的英文名稱和正文是預設後備值。正式卡包透過官方 ID 從 `data/locales/<locale>.json` 合併顯示文字，缺少任一支援語言會被測試拒絕。動態提示使用 `ctx:localize(enUS, zhCN, zhTW)`。

## 關鍵詞也是 Lua

卡牌只引用關鍵詞模組：

```lua
return {
    api_version = 1,
    id = "GVG_085",
    name = "Annoy-o-Tron",
    text = "<b>Taunt</b>\n<b>Divine Shield</b>",
    set = "GVG",
    type = "minion",
    cost = 2,
    attack = 1,
    health = 2,
    keywords = { "taunt", "divine_shield" },
}
```

`taunt` 對應 [data/keywords/taunt.lua](data/keywords/taunt.lua)，它透過通用 `attack_priority` 規則鉤子提高攻擊目標優先順序。`divine_shield` 對應獨立模組，透過 `damaged/before` 觸發器禁用自身並取消傷害事件。

構築模式的 68 個功能性關鍵詞均有獨立 Lua 模組。規則型關鍵詞由模組直接摺疊規則或監聽事件；效果詞由模組統一觸發條件和時序，並以載入期強制的卡牌 hook 承載該牌獨有的數值、目標或衍生物。完整口徑、覆蓋矩陣和不計入項見 [關鍵詞覆蓋表](docs/KEYWORDS.md)。組合這些模組不需要按卡牌 ID 修改 Rust。可交易模組透過通用 `can_trade` 規則開放玩家行動；鍛造、預備和泰坦能力透過通用 `card action` 介面開放玩家行動；CLI 分別使用 `trade <實體ID>` 和 `action <實體ID> <動作ID> [目標ID]`。

帶數值的關鍵詞也由 Lua 模組實現。閃電箭和土元素只宣告 `keywords = { "overload" }` 以及 `keyword_params = { overload = 1/2 }`；[data/keywords/overload.lua](data/keywords/overload.lua) 自己讀取引數並呼叫通用法力原語。被汙染的狂熱者同樣以 `spell_damage = 1` 引數引用 [data/keywords/spell_damage.lua](data/keywords/spell_damage.lua)，由 Lua 提供印刷法強基礎值，再進入 Rust 通用屬性分層和法術傷害結算。

卡牌的隱藏官方規則也留在 Lua。例如野性成長在未滿 10 個水晶時呼叫通用加水晶原語，達到上限時改為生成官方 `CS2_013t`“法力過剩”；該衍生法術自己的抽牌邏輯也與主卡放在同一個 Lua 檔案中。

玩家職業是 Rust 權威對局狀態，但發現池仍由卡牌 Lua 構造。幽靈寫手、劇毒魔蠍和甲蟲鑰匙鏈讀取 `ctx:player(player).class`，只把該職業或中立的合格定義交給通用確定性發現原語；職業會寫入 replay 和 snapshot。

## 新增卡牌

在 `data/sets/<set>/` 增加 Lua 檔案即可。比如一張目標傷害法術：

```lua
return {
    api_version = 1,
    id = "MY_SET_001",
    name = "示例法術",
    text = "造成3點傷害。",
    set = "MY_SET",
    type = "spell",
    cost = 2,
    target_mode = "required",

    targets = function(ctx, self)
        return ctx:enemy_characters(self)
    end,

    on_play = function(ctx, self, target)
        ctx:damage(target, 3)
    end,
}
```

`target_mode = "required"` 用於沒有合法目標便不能打出的法術/英雄技能；帶目標戰吼使用
`"required_if_available"`，有合法目標時必須選擇，沒有時仍可打出且 `on_battlecry` 收到 `nil`。
`"optional"`（預設值）允許省略目標。目標規則本身仍由 Lua 的 `targets` 函式定義。

重啟程式後會自動遞迴發現。只要效果能由現有通用查詢、事件鉤子和效果原語表達，新增卡牌無需修改任何 Rust 程式碼。完整介面見[繁體中文 Lua 卡牌 API](docs/zhTW/CARD_API.md)。

## 執行

需要 Rust 1.88 或更新版本。Lua 5.4 由 `mlua` 的 `vendored` 功能構建。

```bash
cargo run -p hearth-cli -- \
  --deck-one decks/demo.json \
  --deck-two decks/demo.json \
  --seed 42
```

執行經典洞穴任務賊對局：

```bash
cargo run -p hearth-cli -- \
  --deck-one decks/quest_rogue.json \
  --deck-two decks/quest_rogue.json \
  --locale zhCN \
  --seed 42
```

牌表採用 Dog 在 2017 年公開使用的 30 張構築，包含探索地下洞穴、水晶核心、回手元件、帕奇斯、莫羅斯和紫羅蘭教師。任務進度、回手減費、下一個法術減費、牌庫招募、隨機異職業牌和 5/5 持續效果均由 Lua 卡牌/關鍵詞模組實現；Rust 只新增了通用起手規則和裝備生成武器原語。

普通牌組會校驗本職業/中立卡。遊客牌在 Lua 中宣告 `deck_allowances`，可開放指定職業與
卡包並排除目標職業的遊客牌；規則展示用的 `demo.json` 顯式設定 `unrestricted: true`，
因為它有意混合多個職業來演示機制。

雙方依次輸入 `keep` 完成排程。常用命令：

```text
state                       檢視場面
hand                        檢視當前玩家手牌
cards                       檢視卡牌包
legal                       列出所有合法行動
targets <手牌實體ID>         檢視目標
play <手牌實體ID> [目標ID]   出牌
trade <手牌實體ID>           花費1點法力交易可交易牌
action <實體ID> <動作ID> [目標ID] 執行鍛造、預備或泰坦能力
attack <攻擊者ID> <目標ID>   攻擊
power [目標ID]              使用英雄技能
choose <編號>               完成發現/選擇
end                         結束回合
save <檔案>                 儲存 replay
snapshot <檔案>             儲存狀態快照
```

## 驗證

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

端到端測試會載入真實 Lua 卡包，將每個 Lua 卡牌 ID 與來源快照逐一比對，並驗證 68 項關鍵詞目錄、set、關鍵詞 Lua 規則、戰吼、亡語、奧秘、磁力、鍛造、預備、發現、衍生物、隨機狀態遍歷、replay 和 snapshot。

## 邊界

這仍是規則原型，不是完整爐石服務端。關鍵詞層已經覆蓋構築模式詞表，但卡牌庫仍是 150 個代表性官方定義，並不等於完整官方卡池；酒館戰棋、傭兵戰紀的模式專屬關鍵詞也不在本 CLI 對戰規則範圍內。新增一種現有鉤子無法描述的基礎規則時，應優先增加通用規則鉤子或原子效果，而不是在 Rust 中判斷具體卡牌或關鍵詞名。
