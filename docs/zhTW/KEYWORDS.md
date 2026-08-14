# 構築模式關鍵詞覆蓋表

[English](../KEYWORDS.md) | [簡體中文](../zhCN/KEYWORDS.md) | [繁體中文](KEYWORDS.md)

審計日期：2026-08-14。

## 口徑

本專案以 Hearthstone Wiki 的構築模式 Ability 表為基礎，並用暴雪近三個版本的官方公告核對新增詞：

- [Ability / Keyword 彙總](https://hearthstone.wiki.gg/wiki/Ability)
- [逃離紫羅蘭監獄：預備（Prepare）](https://hearthstone.blizzard.com/en-us/news/24276664)
- [大災變：先驅（Herald）、碎裂（Shatter）與巨型迴歸](https://hearthstone.blizzard.com/en-gb/news/24250357/cataclysm-is-now-live)
- [穿越時間流：回溯（Rewind）、傳奇（Fabled）](https://hearthstone.blizzard.com/en-us/news/24226328/)
- [漫遊翡翠夢境：灌注（Imbue）](https://hearthstone.blizzard.com/en-us/news/24179067/step-into-the-emerald-dream-hearthstone-s-next-expansion)

統計結果是 **68 個構築模式功能性關鍵詞，68 個均有 Lua 模組**。倉庫另有
`conditional_charge.lua`，它是南海船工官方卡牌隱藏規則的內部複用模組，不計入官方
關鍵詞數。目錄精確集合由 `keyword_catalog_matches_the_constructed_hearthstone_glossary`
測試鎖定，增加、刪除或誤拼模組都會失敗。

下列內容不計入 68：

- 酒館戰棋、傭兵戰紀及其他模式專屬能力；
- `Corpse`、`Dark Gift`、`Jade Golem`、`Lackey`、`Spare Part` 等資源名、生成池名或卡牌類別；
- “Bonus Effect”等只用於解釋卡牌文字、沒有獨立對局時序的術語。

這些術語由具體卡牌 Lua 使用 `player_data`、動態牌池和普通效果原語表達，不應偽裝成一個
空的戰鬥關鍵詞。

## 68 項清單

### 常駐關鍵詞（27/27）

| Lua ID | 中文 | 實現邊界 |
| --- | --- | --- |
| `battlecry` | 戰吼 | 統一出牌時序，強制卡牌實現 `on_battlecry` |
| `casts_when_drawn` | 抽到時施放 | 移動同一實體、施放並補抽 |
| `charge` | 衝鋒 | 入場立即就緒規則 |
| `counter` | 反制 | 通用 before 事件取消；奧秘 Lua 決定觸發條件 |
| `deathrattle` | 亡語 | 死亡位置、延遲續算與 `on_deathrattle` 契約 |
| `discover` | 發現 | 卡牌 Lua 構造池；Rust RNG 負責無放回抽樣與選擇續算 |
| `divine_shield` | 聖盾 | 傷害前禁用自身並取消該次傷害 |
| `dormant` | 休眠 | 禁止攻擊、被攻擊和定向選中；卡牌指令碼決定甦醒條件 |
| `elusive` | 擾魔 | 雙方的法術與英雄技能均不可定向選中 |
| `freeze` | 凍結 | 卡牌輸出通用凍結原語；核心維護跨回合解凍時點 |
| `immune` | 免疫 | 取消傷害並禁止敵方攻擊/定向選中 |
| `lifesteal` | 吸血 | 按實際傷害治療，支援隨從與武器繼承 |
| `mega_windfury` | 超級風怒 | 每回合四次攻擊，支援武器繼承 |
| `passive` | 被動 | 禁止主動使用英雄技能 |
| `poisonous` | 劇毒 | 實際造成正數傷害後消滅隨從 |
| `reborn` | 復生 | 死亡位置召喚 1 生命新實體並移除復生 |
| `rush` | 突襲 | 入場回合只允許攻擊隨從 |
| `secret` | 奧秘 | Lua `enters_secret_zone` 規則與卡牌觸發器 |
| `silence` | 沉默 | 卡牌選擇目標；通用原語移除可沉默層和指令碼能力 |
| `spell_damage` | 法術傷害 | 引數化基礎法強規則和通用屬性分層 |
| `start_of_game` | 對戰開始時 | 起手前的 `game_started` 事件及卡牌回撥 |
| `stealth` | 潛行 | 目標過濾；攻擊或造成傷害後失去潛行 |
| `summoned_when_drawn` | 抽到時召喚 | 保留同一實體召喚並補抽 |
| `taunt` | 嘲諷 | 通用攻擊優先順序規則 |
| `temporary` | 臨時 | 控制者回合結束移入 removed，不觸發棄牌 |
| `tradeable` | 可交易 | 開放 1 費交易動作、確定性插回牌庫及 replay |
| `windfury` | 風怒 | 每回合兩次攻擊，支援武器繼承 |

### 職業常駐關鍵詞（6/6）

| Lua ID | 中文 | 實現邊界 |
| --- | --- | --- |
| `choose_one` | 抉擇 | 統一生命週期並強制 `on_choose_one` 卡牌回撥 |
| `choose_multiple` | 多選 | 統一生命週期並強制 `on_choose_multiple` 回撥 |
| `combo` | 連擊 | 使用離手前凍結的本回合出牌數 |
| `outcast` | 流放 | 使用離手前凍結的手牌左右端位置 |
| `overheal` | 過量治療 | 治療事件與溢位量判斷，卡牌回撥承載獨有效果 |
| `overload` | 過載 | 引數化欠債、下回合鎖定以及解鎖/清除事件 |

### 版本關鍵詞（35/35）

| Lua ID | 中文 | 共享實現 |
| --- | --- | --- |
| `adapt` | 進化 | 選擇/效果由卡牌回撥，模組統一出牌入口 |
| `colossal` | 巨型 | 任意方式召喚後呼叫元件召喚回撥 |
| `corrupt` | 腐蝕 | 手牌監聽更高費用出牌，一次性轉換 |
| `dredge` | 探底 | 模組統一入口；卡牌回撥使用牌庫實體選擇與置頂原語 |
| `echo` | 迴響 | 模組統一入口；卡牌回撥建立本回合臨時副本 |
| `excavate` | 發掘 | 玩家級四階迴圈計數並把階級交給獎勵回撥 |
| `fabled` | 傳奇 | 起手前從牌庫觸發夥伴加入回撥 |
| `finale` | 壓軸 | 支付後剩餘法力為零才觸發 |
| `forge` | 鍛造 | 手牌 2 費通用動作；該牌只寫 `action_effects.forge` |
| `frenzy` | 暴怒 | 存活傷害後一次性觸發 |
| `gigantify` | 擴大 | 統一入口；卡牌回撥建立其官方巨大衍生物 |
| `herald` | 先驅 | 引數化推進、2/4 次強化檔位，並把次數/總進度/檔位交給士兵回撥 |
| `honorable_kill` | 榮譽消滅 | 精確傷害致 0，支援武器傷害來源 |
| `imbue` | 灌注 | 玩家級永久次數並回撥替換/強化英雄技能 |
| `infuse` | 注能 | 手牌統計友方隨從死亡，達到引數後一次性轉換 |
| `inspire` | 激勵 | 己方英雄技能成功使用後觸發 |
| `invoke` | 祈求 | 玩家級祈求次數並把次數交給卡牌回撥 |
| `kindred` | 同類 | 比較上回合出牌的種族標籤 |
| `magnetic` | 磁力 | 相鄰機械合法位置、屬性/關鍵詞/指令碼合併及沉默 |
| `manathirst` | 法力渴求 | 最大法力達到引數後觸發卡牌回撥 |
| `miniaturize` | 微縮 | 統一入口；卡牌回撥建立對應官方 1/1 衍生物 |
| `overkill` | 超殺 | 傷害令生命低於 0，支援武器來源 |
| `prepare` | 預備 | 花光法力、減費 `已花費+1`、本回合不可打出 |
| `quest` | 任務 | 強制起手並進入持久任務區 |
| `questline` | 任務線 | 強制起手、持久區域與分階段卡牌回撥 |
| `quickdraw` | 快槍 | 僅進入手牌的同一回合觸發 |
| `recruit` | 招募 | 從牌庫移動原實體、預留/取消/位置與召喚事件 |
| `rewind` | 回溯 | 統一入口；卡牌 Lua 儲存可重擲結果並決定接受時點 |
| `shatter` | 碎裂 | 抽到或建立後觸發卡牌的左右半張生成邏輯 |
| `sidequest` | 支線任務 | 進入持久任務區但不強制起手 |
| `spellburst` | 法術迸發 | 己方法術成功施放後一次性觸發 |
| `starship` | 星艦 | 艦船元件死亡回撥；生成艦體以通用戰場動作發射 |
| `titan` | 泰坦 | 三個一次效能力、每回合一次、凍結限制與攻擊解鎖 |
| `tourist` | 遊客 | Lua `deck_allowances` 開放指定職業/卡包並排除目標職業遊客；對局中無觸發器 |
| `twinspell` | 雙生法術 | 統一入口；卡牌回撥生成無雙生法術的官方副本 |

## “實現”在此架構中的含義

一個關鍵詞不一定等於一段固定數值效果。嘲諷、聖盾、磁力等規則由關鍵詞模組完整執行；
戰吼、發現、進化、微縮等詞的觸發時機是共享的，但目標池、數值、選項或官方衍生物 ID
屬於具體卡牌正文。模組透過 `required_card_hooks`、`required_card_actions` 和
`requires_param` 在載入時強制卡牌補齊這些內容。缺少回撥、動作或引數會直接拒絕載入，
不會退化成只顯示一個關鍵詞字串。

因此，增加一張使用現有 68 項關鍵詞的新卡仍然只需新增 Lua 檔案；只有未來出現當前通用
規則、事件、選擇或效果原語無法表達的基礎機制時，才需要擴充套件 Rust 的通用邊界。
