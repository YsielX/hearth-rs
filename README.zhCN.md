# hearth-rs

[English](README.md) | [简体中文](README.zhCN.md) | [繁體中文](README.zhTW.md)

一个 Rust 权威规则内核 + Lua 卡牌/关键词规则层的炉石命令行原型。

核心目标是：新增卡牌只增加 Lua 文件，不注册 Rust ID，也不在 Rust 中为卡名或关键词写分支。

## 当前结构

```text
data/
├── hero_powers/           # 每个英雄技能一个独立 Lua 模块
├── hearthstonejson/       # 已实现定义的官方元数据来源快照
├── keywords/              # 嘲讽、圣盾、突袭等独立 Lua 模块
├── locales/               # enUS / zhCN / zhTW 官方名称与正文
└── sets/                  # 按 HearthstoneJSON set 存放的官方卡牌
crates/
├── hearth-core/           # 状态机、区域、事件队列、确定性 RNG、replay
├── hearth-script/         # Lua 沙箱、模块加载、规则钩子与效果桥接
├── hearth-cli/            # `play` 和 `fuzz` 子命令
├── hearth-bot/            # 不读取隐藏信息的确定性基础 Bot
└── hearth-fuzz/           # 状态机 Fuzzer 库（无独立二进制）
decks/demo.json            # 官方卡演示牌组
decks/quest_rogue.json     # Dog 2017 经典洞穴任务贼
```

Rust 负责不能交给脚本随意修改的原子能力：实体身份、区域容器、法力支付、攻击/伤害提交、死亡检查点、效果队列、输入暂停、确定性随机、事务回滚与 replay。

Lua 负责卡牌语义和关键词语义：目标选择、战吼、亡语、奥秘、发现、触发条件、触发效果，以及攻击规则修饰。Rust 引擎里不再按 `"taunt"`、`"divine_shield"`、`"reborn"` 等字符串执行具体规则。

## 官方卡牌数据

当前仓库保留 1386 个官方卡牌、衍生物、英雄和英雄技能定义，覆盖 45 个 set；其中包括完整的 30 张“纳克萨玛斯的诅咒”、123 张“地精大战侏儒”、31 张“黑石山的火焰”、132 张“冠军的试炼”、45 张“探险者协会”、134 张“上古之神的低语”、45 张“卡拉赞之夜”、132 张“龙争虎斗加基森”、135 张“勇闯安戈洛”和 135 张“冰封王座的骑士”，以及 11 个基础职业英雄技能、每个已审计构筑关键词至少一张完整实现的官方代表牌，并完整实现一套经典洞穴任务贼。这仍是代表性规则语料，不是完整官方卡池。

名称、正文、数值、官方 ID 和 set 来自 HearthstoneJSON 的客户端数据。默认英文来源快照位于 [data/hearthstonejson/selected.enUS.json](data/hearthstonejson/selected.enUS.json)，三语显示文本位于 `data/locales/`，取数说明见[简体中文取数文档](data/hearthstonejson/zhCN/README.md)。不可收集衍生物也使用官方 ID，例如鬼灵蜘蛛 `FP1_002t`，不再使用自造的 `TOKEN_*` ID。

## 三语文本

CLI 的 `--locale` 接受 `enUS`、`zhCN` 和 `zhTW`，未指定时默认英文。它会切换卡牌名称、正文、帮助、状态标签、事件、错误以及 Lua 动态选项提示；命令本身保持稳定的英文关键字，便于 replay 和脚本复用。牌组名是用户随意填写的元数据，始终原样显示唯一的 `name` 值，不参与 locale。

```bash
cargo run -p hearth-cli -- play --locale zhTW
cargo run -p hearth-cli -- play --locale enUS
```

卡牌 Lua 中的英文名称和正文是默认后备值。正式卡包通过官方 ID 从 `data/locales/<locale>.json` 合并显示文本，缺少任一支持语言会被测试拒绝。动态提示使用 `ctx:localize(enUS, zhCN, zhTW)`。

## 关键词也是 Lua

卡牌只引用关键词模块：

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

`taunt` 对应 [data/keywords/taunt.lua](data/keywords/taunt.lua)，它通过通用 `attack_priority` 规则钩子提高攻击目标优先级。`divine_shield` 对应独立模块，通过 `damaged/before` 触发器禁用自身并取消伤害事件。

构筑模式的 68 个功能性关键词均有独立 Lua 模块。规则型关键词由模块直接折叠规则或监听事件；效果词由模块统一触发条件和时序，并以加载期强制的卡牌 hook 承载该牌独有的数值、目标或衍生物。完整口径、覆盖矩阵和不计入项见 [关键词覆盖表](docs/KEYWORDS.md)。组合这些模块不需要按卡牌 ID 修改 Rust。可交易模块通过通用 `can_trade` 规则开放玩家行动；锻造、预备和泰坦能力通过通用 `card action` 接口开放玩家行动；CLI 分别使用 `trade <实体ID>` 和 `action <实体ID> <动作ID> [目标ID]`。

带数值的关键词也由 Lua 模块实现。闪电箭和土元素只声明 `keywords = { "overload" }` 以及 `keyword_params = { overload = 1/2 }`；[data/keywords/overload.lua](data/keywords/overload.lua) 自己读取参数并调用通用法力原语。被污染的狂热者同样以 `spell_damage = 1` 参数引用 [data/keywords/spell_damage.lua](data/keywords/spell_damage.lua)，由 Lua 提供印刷法强基础值，再进入 Rust 通用属性分层和法术伤害结算。

卡牌的隐藏官方规则也留在 Lua。例如野性成长在未满 10 个水晶时调用通用加水晶原语，达到上限时改为生成官方 `CS2_013t`“法力过剩”；该衍生法术自己的抽牌逻辑也与主卡放在同一个 Lua 文件中。

玩家职业是 Rust 权威对局状态，但发现池仍由卡牌 Lua 构造。幽灵写手、剧毒魔蝎和甲虫钥匙链读取 `ctx:player(player).class`，只把该职业或中立的合格定义交给通用确定性发现原语；职业会写入 replay 和 snapshot。

## 新增卡牌

在 `data/sets/<set>/` 增加 Lua 文件即可。比如一张目标伤害法术：

```lua
return {
    api_version = 1,
    id = "MY_SET_001",
    name = "示例法术",
    text = "造成3点伤害。",
    set = "MY_SET",
    type = "spell",
    cost = 2,
    target_mode = "required",

    targets = function(ctx, self)
        return ctx:enemy_characters(self)
    end,

    on_play = function(ctx, self, target)
        cardlib.effects.damage(ctx, target, 3)
    end,
}
```

`target_mode = "required"` 用于没有合法目标便不能打出的法术/英雄技能；带目标战吼使用
`"required_if_available"`，有合法目标时必须选择，没有时仍可打出且 `on_battlecry` 收到 `nil`。
`"optional"`（默认值）允许省略目标。目标规则本身仍由 Lua 的 `targets` 函数定义。

重启程序后会自动递归发现。只要效果能由现有通用查询、事件钩子和效果原语表达，新增卡牌无需修改任何 Rust 代码。完整接口见[简体中文 Lua 卡牌 API](docs/zhCN/CARD_API.md)。

## 运行

需要 Rust 1.88 或更新版本。Lua 5.4 由 `mlua` 的 `vendored` 功能构建。

```bash
cargo run -p hearth-cli -- play \
  --deck-one decks/demo.json \
  --deck-two decks/demo.json \
  --seed 42
```

运行经典洞穴任务贼对局：

```bash
cargo run -p hearth-cli -- play \
  --deck-one decks/quest_rogue.json \
  --deck-two decks/quest_rogue.json \
  --locale zhCN \
  --seed 42
```

牌表采用 Dog 在 2017 年公开使用的 30 张构筑，包含探索地下洞穴、水晶核心、回手组件、帕奇斯、莫罗斯和紫罗兰教师。任务进度、回手减费、下一个法术减费、牌库招募、随机异职业牌和 5/5 持续效果均由 Lua 卡牌/关键词模块实现；Rust 只新增了通用起手规则和装备生成武器原语。

普通牌组会校验本职业/中立卡。游客牌在 Lua 中声明 `deck_allowances`，可开放指定职业与
卡包并排除目标职业的游客牌；规则展示用的 `demo.json` 显式设置 `unrestricted: true`，
因为它有意混合多个职业来演示机制。

双方依次输入 `keep` 完成调度。常用命令：

```text
state                       查看场面
hand                        查看当前玩家手牌
cards                       查看卡牌包
legal                       列出所有合法行动
targets <手牌实体ID>         查看目标
play <手牌实体ID> [目标ID]   出牌
trade <手牌实体ID>           花费1点法力交易可交易牌
action <实体ID> <动作ID> [目标ID] 执行锻造、预备或泰坦能力
attack <攻击者ID> <目标ID>   攻击
power [目标ID]              使用英雄技能
choose <编号>               完成发现/选择
end                         结束回合
save <文件>                 保存 replay
snapshot <文件>             保存状态快照
```

## 玩家控制器与隐藏信息

两个玩家位置都可以独立选择 `interactive`、`bot` 或 `fuzzer`：

```bash
cargo run -p hearth-cli --release -- play \
  --deck-one decks/quest_rogue.json \
  --deck-two decks/quest_rogue.json \
  --player-one interactive \
  --player-two bot
```

控制器只能接收玩家视角投影和引擎给出的合法操作元数据，不能读取原始 `GameState`。玩家视角会排除双方牌库顺序（也包括自己的牌库顺序）、对手手牌和普通奥秘身份、脚本数据、隐藏光环来源、RNG 状态以及 replay；任务、任务线和支线任务则按官方规则保持公开。CLI 事件输出会隐藏对手抽牌、生成到手牌的卡、未揭示奥秘名称、隐藏选择和隐藏随机抽样。双交互玩家热座模式使用清屏交接；普通对局中禁止导出包含权威隐藏状态的 replay/snapshot，只有显式传入 `--debug-state` 才会开启该调试能力。

基础 [`hearth-bot`](crates/hearth-bot/README.md) 按“场攻斩杀、规划当前合法操作以尽量打满费用、优势交换、踢脸”的顺序行动。嘲讽等攻击限制仍完全由引擎决定，因为 Bot 只会从引擎枚举的合法攻击中选择。

## 验证

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

端到端测试会加载真实 Lua 卡包，将每个 Lua 卡牌 ID 与来源快照逐一比对，并验证 68 项关键词目录、set、关键词 Lua 规则、战吼、亡语、奥秘、磁力、锻造、预备、发现、衍生物、replay 和 snapshot。

## 状态机 Fuzz 测试

确定性状态机 Fuzzer 实现在 [`hearth-fuzz` 库](crates/hearth-fuzz/README.md)中，并且只通过 `hearth-cli` 子命令提供；普通 `cargo test` 不会启动 fuzz campaign。它会生成职业合法套牌、抽取引擎枚举的合法操作、在每一步校验状态不变量，并将最终状态与 replay 对比：

```bash
cargo run -p hearth-cli --release -- fuzz --seeds 100 --steps 180
```

## 边界

这仍是规则原型，不是完整炉石服务端。关键词层已经覆盖构筑模式词表，但卡牌库仍是 1386 个官方定义，并不等于完整官方卡池；酒馆战棋、佣兵战纪的模式专属关键词也不在本 CLI 对战规则范围内。新增一种现有钩子无法描述的基础规则时，应优先增加通用规则钩子或原子效果，而不是在 Rust 中判断具体卡牌或关键词名。
