# 架构说明

[English](../ARCHITECTURE.md) | [简体中文](ARCHITECTURE.md) | [繁體中文](../zhTW/ARCHITECTURE.md)

## 设计约束

1. Rust 是唯一权威状态，不允许 Lua 持有可变游戏对象。
2. 卡牌 ID、卡牌逻辑和具体关键词语义不进入 Rust 分支。
3. Lua 只能读取快照并输出效果描述；Rust 验证后原子提交。
4. 同一种子、卡牌包指纹和命令序列必须产生相同状态。
5. 新卡能组合已有规则钩子与效果原语时，只增加 Lua 文件。

## 分层

```text
CLI / 未来网络协议
        │ PlayerCommand / legal_actions
        ▼
Rust Game 状态机
  ├─ 区域与实体不变量
  ├─ 法力、回合、攻击、伤害和死亡检查点
  ├─ PendingEvent / ResolutionItem 队列
  ├─ 确定性 RNG、事务回滚、replay、snapshot
  └─ 通用 keyword rule 查询
        │ 只读 GameState + hook 参数
        ▼
LuaCardRuntime
  ├─ 卡牌模块：targets / on_play / on_battlecry / triggers / auras
  ├─ 英雄技能模块：每个技能一个独立加载模块
  ├─ 关键词模块：rules / hooks / triggers
  └─ 通用 ctx 查询与 EffectSpec 输出
        │
        ▼
data/sets/**/*.lua + data/hero_powers/**/*.lua + data/keywords/*.lua + data/libraries/*.lua
```

## 三类 Lua 模块

卡牌模块默认 `module_type = "card"`，也可以省略该字段。它包含官方元数据和卡牌钩子。

英雄技能模块声明 `module_type = "hero_power"`。加载器自动赋予不可收集的 `hero_power` 类型，模块负责费用、目标、`on_play`、触发器、衍生物和关键词引用。英雄牌仍是 `type = "hero"` 的卡牌模块，并声明 `armor` 和经过校验的 `hero_power` ID。

共享 Lua 库声明 `module_type = "library"`、`api_version = 1` 和 `id`，并暴露为 `cardlib[id]`。它参与卡包哈希但不会注册成卡牌，用来组合通用 ctx 操作，而不是增加卡牌专属 Rust effect。

关键词模块显式声明：

```lua
return {
    api_version = 1,
    module_type = "keyword",
    id = "taunt",
    name = "嘲讽",
    rules = {
        attack_priority = function(ctx, self, current, attacker)
            return math.max(current, 1)
        end,
    },
}
```

加载结束后，运行时会验证每张卡引用的关键词 ID 都存在。关键词源文件和卡牌源文件共同进入卡牌包哈希，所以修改任一规则都会令旧 replay 拒绝加载。

关键词的 `hooks` 可以接入通用生命周期入口，目前支持 `on_play` 和 `on_location_use`。关键词还可通过 `actions` 声明手牌或战场上的命名玩家动作，例如锻造、预备与三项泰坦能力。`battlecry.lua` 的 `hooks.on_play` 输出一个命名 continuation，转到卡牌的 `on_battlecry`；`combo.lua` 先查询冻结的出牌前上下文，再按条件转到 `on_combo`；`finale.lua` 在付费后检查剩余法力，再转到 `on_finale`。Rust 只遍历实体当前的关键词模块并调用通用入口，不检查这些关键词字符串。`required_card_hooks` 和 `required_card_actions` 会在加载期验证卡牌侧契约。

关键词模块可用 `requires_param = true` 声明数值契约。卡牌以 `keyword_params = { keyword_id = value }` 提供静态整数，Lua 通过通用 `ctx:keyword_param` 查询。加载器只验证引用关系和必需参数，数值含义仍属于关键词 Lua：例如 `overload.lua` 决定在 `on_play` 读取参数并输出 `Overload` 效果，Rust 没有 `overload` 关键词分支。

## 规则折叠而不是关键词分支

Rust 在需要做规则决策时询问通用规则名：

| rule | 初始值 | 用途 |
| --- | ---: | --- |
| `attack_priority` | `0` | 高优先级目标屏蔽低优先级目标 |
| `can_be_attacked` | `true` | 目标能否被攻击 |
| `can_be_targeted_by_enemy` | `true` | 能否成为敌方定向效果目标 |
| `can_attack_while_exhausted` | `false` | 新入场且休眠时能否攻击某目标 |
| `ready_on_summon` | `false` | 入场时是否解除休眠 |
| `max_attacks` | `1` | 每回合最大攻击次数 |
| `can_trade` | `false` | 手牌实体是否开放“交易”玩家行动 |
| `can_play` | `true` | 当前实体是否可从手牌打出 |
| `can_attack` | `true` | 当前角色是否可主动攻击 |
| `can_be_targeted` | `true` | 是否可成为任一方定向效果目标 |
| `enters_secret_zone` | `false` | 法术结算后是否进入持久任务/奥秘区 |
| `starts_in_opening_hand` | `false` | 是否强制进入起手 |
| `hero_power_is_passive` | `false` | 英雄技能是否禁止主动使用 |
| `can_magnetize` | `false` | 手牌随从是否开放相邻机械合体放置 |
| `base_spell_damage` | `0` | 为通用法强属性分层提供印刷基础值 |

实体的所有有效关键词模块按稳定顺序折叠当前值。这样 Rust 只认识规则接口，不认识 `taunt` 或 `rush`。未来的关键词只要能组合这些规则和事件触发器，就不需要 Rust 改动。

武器的关键词模块仍属于武器实体。需要修改英雄攻击规则的模块显式设置 `weapon_inherits_to_hero = true`（例如风怒），规则查询才会组合到当前出鞘武器；伤害后的关键词触发器则由武器监听事件并检查伤害来源是否为其英雄。

## 事件关键词

圣盾、免疫、潜行、剧毒、吸血、亡语和复生不是伤害或死亡函数中的硬编码：

- 圣盾监听 `damaged/before`，输出 `disable_keyword` 和 `cancel_event`；
- 免疫监听 `damaged/before` 并取消事件，同时通过规则钩子阻止敌方选中；
- 潜行在自身 `attack/after` 时禁用；
- 剧毒监听自身造成的 `damaged/after` 并输出通用 `destroy`；
- 吸血监听伤害并输出通用 `heal`；
- 亡语监听墓地中的自身 `entity_died/after`，通过 continuation 调用卡牌的 `on_deathrattle`；
- 复生监听墓地中的 `entity_died/after`，调用 `summon_fresh_copy`，指定 1 点生命并排除 `reborn`。

战吼、连击与压轴由 lifecycle keyword 驱动：`battlecry.lua` 在出牌阶段把已声明目标传给卡牌的 `on_battlecry`；`combo.lua` 仅在当前牌不是本回合第一张手牌时调用 `on_combo`；`finale.lua` 仅在本次付费后剩余法力为零时调用 `on_finale`。法术迸发监听己方的 `spell_cast/after`，先禁用自身关键词，再通过可序列化 continuation 调用 `on_spellburst`；亡语用同一机制把死亡位置传给 `on_deathrattle`。关键词模块用 `required_card_hooks` 声明契约，加载卡包时会拒绝只引用关键词却没有实现效果函数的卡牌。卡牌文件因此只写该牌独有的效果，不再重复触发条件、时序与一次性状态。

数值型 lifecycle 关键词使用同一模型：`overload.lua` 读取卡牌的 `keyword_params.overload` 并输出通用法力效果。土元素本身没有 `on_play`，召唤效果也不会误触发过载；只有从手牌成功打出时才进入 lifecycle hook。熔岩震击则组合伤害和通用 `clear_overload` 原语，同时清除当前与待生效过载。

持续数值关键词也使用规则折叠：`spell_damage.lua` 读取 `keyword_params.spell_damage` 并提供 `base_spell_damage`。Rust 不解析该关键词 ID，只把规则结果作为通用法强属性的基础值，再应用 enchantment 与光环层。沉默移除关键词后下一次重算自然回到零。

Rust 为此提供的都是通用原语：禁用任意关键词、取消任意待提交事件、召唤定义的新鲜副本并排除任意关键词列表。没有原语检查具体关键词名。

## 事件与结算

一次会被响应的动作先创建 `PendingEvent`：

```text
创建 before 事件
  → APNAP 收集卡牌触发器和关键词触发器
  → 依次执行其 EffectSpec
  → 提交或取消事件
  → 写入日志
  → 发布 after
  → Death Checkpoint
```

Lua hook 不直接改变 `GameState`。例如 `cardlib.effects.damage(ctx, target, 3)` 只输出 `EffectSpec::Damage`。这样每个成功玩家命令可以在临时状态中完成；任何 Lua 错误、非法目标或不变量失败都会回滚整个命令。

同时伤害使用一组待提交事件：先收集全部 `before`，再提交未取消项，然后基于同一提交后状态发布 `after`，最后统一进入死亡检查点。

## 目标策略

目标候选始终由 Lua 的 `targets` 或 `location_targets` 生成，再经过 Rust 的通用合法性过滤。卡牌用 `target_mode` 声明选择策略：

- `required`：没有合法目标就不能使用，适合法术和英雄技能；
- `required_if_available`：有目标必须选择、无目标仍可打出，适合带目标战吼；
- `optional`：允许无目标，也允许从候选中选择一个。

因此 Rust 不需要知道哪张牌是战吼或连击牌，也没有卡牌 ID 特判。`required_if_available` 的 `on_battlecry` / `on_combo` 必须处理 `target == nil`。合法性只在玩家声明目标时检查；随后触发器、控制权或位置光环即使改变目标属性，也不会重新运行 Lua selector。结算仍使用已声明的稳定 `EntityId`，具体效果原语再根据实体结算时所在区域决定是否生效。

手牌出牌进入战场或墓地后，先完整结算 `on_play` 与关键词 lifecycle 效果，再依次发布 `card_played`、类型细分事件以及随从的 `minion_summoned` after 通知。因此战吼召出的衍生物会先完成自己的召唤序列，随后才进入原随从的 After Play / After Summon 阶段。

## 关键词开放玩家行动

`tradeable.lua` 不在 Rust 中注册关键词名，而是把通用布尔规则 `can_trade` 折叠为 `true`。`legal_actions` 因此为手牌实体增加 `TradeCard`：花费 1 点法力，先抽取一个不同实体，再把原实体插入确定性随机牌库位置。交易不算出牌，保留原实体及其 enchantment，并作为 replay 命令序列的一部分。

默认抽牌被建模为 `trade_draw/before → CardDrawn → trade_draw/after` 子流程。Lua 触发器可以暂停 before 阶段，用 `discover_entities` 从真实牌库实体中抽样，再用 `replace_trade_draw` 修改仍在队列中的通用事件。拍卖师亚克森完全由这三个接口组合，核心没有其卡牌 ID 或“发现”业务分支。

## 死亡与复生位置

Rust 死亡检查点先按入场顺序识别所有致死随从，再逐一移除。每个随从的 `position` 在它实际移除时记录，因此同批中较早死亡的随从不会继续占据较晚死亡随从的位置。全部移入墓地后再批量发布 `entity_died`；Rust 不检查复生或任何具体亡语。

`deathrattle.lua` 把事件的 `position` 传给卡牌的 `on_deathrattle`，卡牌可据此调用 `summon_at`；`reborn.lua` 则输出通用新鲜副本召唤效果。两者仍经过可取消的 `minion_summoned/before/after`，战场已满时安全失败。

## 数据和状态

卡牌静态数据来自 Lua `CardDefinition`，包括关键词引用和通用数值参数；运行中每一个实例是 Rust `Entity`。实体保存基础属性、伤害、区域、控制者、enchantment、已禁用关键词、冻结状态、攻击次数和可序列化 `script_data`。玩家职业同样属于 `PlayerState`，Lua 只能读取，不能在全局变量中伪造。

奥秘、任务和任务线分别引用 `secret.lua`、`quest.lua`、`questline.lua`；这些模块提供通用 `enters_secret_zone` 规则，核心只询问规则名。`CardDefinition.secret` 仅为旧卡包兼容字段，新 Lua 文件不应再使用它。

## 沙箱和确定性

- 删除 `dofile/loadfile/require/package/io/os/debug`；
- 删除 `math.random/randomseed`；
- Lua 内存上限 16 MiB；
- 单次 hook 指令预算 200,000；
- 随机与发现都由 Rust 种子 RNG 执行；
- 源文件相对路径和内容进入稳定卡牌包哈希；
- replay 保存初始牌组、玩家职业、英雄技能、种子、卡牌包哈希和成功命令；
- snapshot 内嵌 replay，并通过重放逐字段验证权威状态。

## 扩展原则

新增卡牌时优先组合现有 `ctx` API。新增关键词时优先组合通用 rules 与 triggers。只有出现无法由现有边界表达、且确实适用于多张卡的基础规则时，才向 Rust 增加新的通用 rule 或原子 EffectSpec；禁止添加 `if card_id == ...` 或 `if keyword == ...` 的业务分支。
