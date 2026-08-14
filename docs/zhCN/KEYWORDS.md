# 构筑模式关键词覆盖表

[English](../KEYWORDS.md) | [简体中文](KEYWORDS.md) | [繁體中文](../zhTW/KEYWORDS.md)

审计日期：2026-08-14。

## 口径

本项目以 Hearthstone Wiki 的构筑模式 Ability 表为基础，并用暴雪近三个版本的官方公告核对新增词：

- [Ability / Keyword 汇总](https://hearthstone.wiki.gg/wiki/Ability)
- [逃离紫罗兰监狱：预备（Prepare）](https://hearthstone.blizzard.com/en-us/news/24276664)
- [大灾变：先驱（Herald）、碎裂（Shatter）与巨型回归](https://hearthstone.blizzard.com/en-gb/news/24250357/cataclysm-is-now-live)
- [穿越时间流：回溯（Rewind）、传奇（Fabled）](https://hearthstone.blizzard.com/en-us/news/24226328/)
- [漫游翡翠梦境：灌注（Imbue）](https://hearthstone.blizzard.com/en-us/news/24179067/step-into-the-emerald-dream-hearthstone-s-next-expansion)

统计结果是 **68 个构筑模式功能性关键词，68 个均有 Lua 模块**。仓库另有
`conditional_charge.lua`，它是南海船工官方卡牌隐藏规则的内部复用模块，不计入官方
关键词数。目录精确集合由 `keyword_catalog_matches_the_constructed_hearthstone_glossary`
测试锁定，增加、删除或误拼模块都会失败。

下列内容不计入 68：

- 酒馆战棋、佣兵战纪及其他模式专属能力；
- `Corpse`、`Dark Gift`、`Jade Golem`、`Lackey`、`Spare Part` 等资源名、生成池名或卡牌类别；
- “Bonus Effect”等只用于解释卡牌文字、没有独立对局时序的术语。

这些术语由具体卡牌 Lua 使用 `player_data`、动态牌池和普通效果原语表达，不应伪装成一个
空的战斗关键词。

## 68 项清单

### 常驻关键词（27/27）

| Lua ID | 中文 | 实现边界 |
| --- | --- | --- |
| `battlecry` | 战吼 | 统一出牌时序，强制卡牌实现 `on_battlecry` |
| `casts_when_drawn` | 抽到时施放 | 移动同一实体、施放并补抽 |
| `charge` | 冲锋 | 入场立即就绪规则 |
| `counter` | 反制 | 通用 before 事件取消；奥秘 Lua 决定触发条件 |
| `deathrattle` | 亡语 | 死亡位置、延迟续算与 `on_deathrattle` 契约 |
| `discover` | 发现 | 卡牌 Lua 构造池；Rust RNG 负责无放回抽样与选择续算 |
| `divine_shield` | 圣盾 | 伤害前禁用自身并取消该次伤害 |
| `dormant` | 休眠 | 禁止攻击、被攻击和定向选中；卡牌脚本决定苏醒条件 |
| `elusive` | 扰魔 | 双方的法术与英雄技能均不可定向选中 |
| `freeze` | 冻结 | 卡牌输出通用冻结原语；核心维护跨回合解冻时点 |
| `immune` | 免疫 | 取消伤害并禁止敌方攻击/定向选中 |
| `lifesteal` | 吸血 | 按实际伤害治疗，支持随从与武器继承 |
| `mega_windfury` | 超级风怒 | 每回合四次攻击，支持武器继承 |
| `passive` | 被动 | 禁止主动使用英雄技能 |
| `poisonous` | 剧毒 | 实际造成正数伤害后消灭随从 |
| `reborn` | 复生 | 死亡位置召唤 1 生命新实体并移除复生 |
| `rush` | 突袭 | 入场回合只允许攻击随从 |
| `secret` | 奥秘 | Lua `enters_secret_zone` 规则与卡牌触发器 |
| `silence` | 沉默 | 卡牌选择目标；通用原语移除可沉默层和脚本能力 |
| `spell_damage` | 法术伤害 | 参数化基础法强规则和通用属性分层 |
| `start_of_game` | 对战开始时 | 起手前的 `game_started` 事件及卡牌回调 |
| `stealth` | 潜行 | 目标过滤；攻击或造成伤害后失去潜行 |
| `summoned_when_drawn` | 抽到时召唤 | 保留同一实体召唤并补抽 |
| `taunt` | 嘲讽 | 通用攻击优先级规则 |
| `temporary` | 临时 | 控制者回合结束移入 removed，不触发弃牌 |
| `tradeable` | 可交易 | 开放 1 费交易动作、确定性插回牌库及 replay |
| `windfury` | 风怒 | 每回合两次攻击，支持武器继承 |

### 职业常驻关键词（6/6）

| Lua ID | 中文 | 实现边界 |
| --- | --- | --- |
| `choose_one` | 抉择 | 统一生命周期并强制 `on_choose_one` 卡牌回调 |
| `choose_multiple` | 多选 | 统一生命周期并强制 `on_choose_multiple` 回调 |
| `combo` | 连击 | 使用离手前冻结的本回合出牌数 |
| `outcast` | 流放 | 使用离手前冻结的手牌左右端位置 |
| `overheal` | 过量治疗 | 治疗事件与溢出量判断，卡牌回调承载独有效果 |
| `overload` | 过载 | 参数化欠债、下回合锁定以及解锁/清除事件 |

### 版本关键词（35/35）

| Lua ID | 中文 | 共享实现 |
| --- | --- | --- |
| `adapt` | 进化 | 选择/效果由卡牌回调，模块统一出牌入口 |
| `colossal` | 巨型 | 任意方式召唤后调用组件召唤回调 |
| `corrupt` | 腐蚀 | 手牌监听更高费用出牌，一次性转换 |
| `dredge` | 探底 | 模块统一入口；卡牌回调使用牌库实体选择与置顶原语 |
| `echo` | 回响 | 模块统一入口；卡牌回调创建本回合临时副本 |
| `excavate` | 发掘 | 玩家级四阶循环计数并把阶级交给奖励回调 |
| `fabled` | 传奇 | 起手前从牌库触发伙伴加入回调 |
| `finale` | 压轴 | 支付后剩余法力为零才触发 |
| `forge` | 锻造 | 手牌 2 费通用动作；该牌只写 `action_effects.forge` |
| `frenzy` | 暴怒 | 存活伤害后一次性触发 |
| `gigantify` | 扩大 | 统一入口；卡牌回调创建其官方巨大衍生物 |
| `herald` | 先驱 | 参数化推进、2/4 次强化档位，并把次数/总进度/档位交给士兵回调 |
| `honorable_kill` | 荣誉消灭 | 精确伤害致 0，支持武器伤害来源 |
| `imbue` | 灌注 | 玩家级永久次数并回调替换/强化英雄技能 |
| `infuse` | 注能 | 手牌统计友方随从死亡，达到参数后一次性转换 |
| `inspire` | 激励 | 己方英雄技能成功使用后触发 |
| `invoke` | 祈求 | 玩家级祈求次数并把次数交给卡牌回调 |
| `kindred` | 同类 | 比较上回合出牌的种族标签 |
| `magnetic` | 磁力 | 相邻机械合法位置、属性/关键词/脚本合并及沉默 |
| `manathirst` | 法力渴求 | 最大法力达到参数后触发卡牌回调 |
| `miniaturize` | 微缩 | 统一入口；卡牌回调创建对应官方 1/1 衍生物 |
| `overkill` | 超杀 | 伤害令生命低于 0，支持武器来源 |
| `prepare` | 预备 | 花光法力、减费 `已花费+1`、本回合不可打出 |
| `quest` | 任务 | 强制起手并进入持久任务区 |
| `questline` | 任务线 | 强制起手、持久区域与分阶段卡牌回调 |
| `quickdraw` | 快枪 | 仅进入手牌的同一回合触发 |
| `recruit` | 招募 | 从牌库移动原实体、预留/取消/位置与召唤事件 |
| `rewind` | 回溯 | 统一入口；卡牌 Lua 保存可重掷结果并决定接受时点 |
| `shatter` | 碎裂 | 抽到或创建后触发卡牌的左右半张生成逻辑 |
| `sidequest` | 支线任务 | 进入持久任务区但不强制起手 |
| `spellburst` | 法术迸发 | 己方法术成功施放后一次性触发 |
| `starship` | 星舰 | 舰船组件死亡回调；生成舰体以通用战场动作发射 |
| `titan` | 泰坦 | 三个一次性能力、每回合一次、冻结限制与攻击解锁 |
| `tourist` | 游客 | Lua `deck_allowances` 开放指定职业/卡包并排除目标职业游客；对局中无触发器 |
| `twinspell` | 双生法术 | 统一入口；卡牌回调生成无双生法术的官方副本 |

## “实现”在此架构中的含义

一个关键词不一定等于一段固定数值效果。嘲讽、圣盾、磁力等规则由关键词模块完整执行；
战吼、发现、进化、微缩等词的触发时机是共享的，但目标池、数值、选项或官方衍生物 ID
属于具体卡牌正文。模块通过 `required_card_hooks`、`required_card_actions` 和
`requires_param` 在加载时强制卡牌补齐这些内容。缺少回调、动作或参数会直接拒绝加载，
不会退化成只显示一个关键词字符串。

因此，增加一张使用现有 68 项关键词的新卡仍然只需新增 Lua 文件；只有未来出现当前通用
规则、事件、选择或效果原语无法表达的基础机制时，才需要扩展 Rust 的通用边界。
