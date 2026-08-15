# 炉石强化学习训练

这套训练代码的边界是：`hearth-core` 只负责规则和公开事件，`hearth-script`
负责加载并执行 Lua 卡牌，`hearth-env` 把合法动作和玩家可见信息转换成稳定协议；张量、
奖励回填、经验池、模型和联赛全部位于 `python/hearth_env/training`。模型不会读取
`GameState`、对手手牌、牌库顺序、RNG 状态或 replay。

## 模型为什么同时使用定义、Lua 和文本

卡 ID embedding 能精确记忆见过的卡，但单独使用它无法迁移到新卡。当前编码器因此使用：

1. `CardDefinition` 的费用、攻击、生命、类型、职业、keyword 等显式字段；
2. 英文描述和结构字段的词法特征；
3. 定义该卡（包括同文件 token）的原始 Lua 源码词法及相邻 token 特征；
4. 一个可增量扩展的卡 ID embedding，作为已知卡的残差记忆。

这些词法特征使用固定维度、稳定的 signed hashing，不需要先训练一个代码语言模型，也不会
在加入新卡时改变输入形状。Lua 字节码不作为持久输入：它依赖 Lua 版本/编译选项，并丢失
一部分对泛化有用的标识符信息。原始 Lua 是卡包的一部分，checkpoint 同时记录 card-pack
hash；增加新卡后加载旧 checkpoint，会按卡 ID 搬移旧 embedding，并为新卡保留结构/Lua
语义通路，再继续自博弈。

这并不保证模型“零训练就精通新卡”。Lua 的静态词法特征只能给合理先验，复杂交互最终仍需
通过实际对局学习。不过，同一模型可训练多个职业和任意已有卡组成的套牌，不需要每套牌一个
模型；开局套牌多重集本身也是模型上下文。

## 训练阶段

完整入口按以下顺序执行：

1. 用外部启发式策略生成行为克隆数据，学会基本合法节奏，避免从纯随机策略冷启动；
2. 行为克隆（BC）用所有合法动作上的交叉熵训练动作评分；
3. Deep Monte Carlo（DMC）自博弈，把终局胜负 `+1/0/-1` 回填到每个行动者当时的动作；
4. 定期冻结 checkpoint 进入 league，当前模型会与当前/历史/启发式策略混合对战；
5. 用成对换边评估，按套牌和 matchup 统计，而不是只看训练 loss；
6. 新卡加入后直接加载旧 checkpoint，以包含新卡的套牌继续 BC/DMC。旧卡 embedding 会保留。

所谓 10% “specialist” 默认是自动启发式/冻结策略提供的行为多样性，不要求人工打 10% 的
对局。你的高质量人类对局很珍贵，可以以后作为小比例加权 BC 数据，而不是训练成立的前提。

## 安装和最短验证

```bash
python3 -m venv .venv
.venv/bin/pip install -e '.[train]'
.venv/bin/hearth-train smoke
```

所有全局参数应放在子命令前，例如：

```bash
.venv/bin/hearth-train --workers 16 --device auto pipeline \
  --run-dir runs/first \
  --bc-episodes 10000 \
  --iterations 1000
```

也可以逐阶段运行：

```bash
.venv/bin/hearth-train --workers 16 collect-bc \
  --episodes 10000 --output training-data/bc-000.jsonl.gz

.venv/bin/hearth-train --device cuda train-bc \
  --input training-data/bc-000.jsonl.gz --output runs/bc.pt --epochs 3

.venv/bin/hearth-train --workers 24 --device cuda train-dmc \
  --init runs/bc.pt --run-dir runs/dmc \
  --iterations 1000 --episodes-per-iteration 64 --updates-per-iteration 128

.venv/bin/hearth-train --device cuda evaluate \
  --checkpoint runs/dmc/latest.pt --matches 200
```

每个 rollout worker 是一个独立 OS 进程，拥有一个 Lua runtime，并用 `reset_match` 连续重开
不同套牌的多局；Lua 对象不跨线程共享。learner 可用一张 GPU，actor 默认在 CPU 推理。原始
episode 以 gzip JSONL 保存，便于审计；大规模运行若 I/O 成为瓶颈，下一步应只替换 Python
数据传输/存储为紧凑二进制，不需要改 core。

## 配置和数据量起点

默认 128 hidden、2 层 encoder 在当前 1537 张定义上约 127 万参数：FP32 权重约 4.8 MiB，
卡牌语义表约 1.6 MiB。8 GB 显存足以进行默认 batch 训练；16–24 GB 能提高 batch 和模型宽度，
但不是起步条件。本机无 GPU 时也能完成 smoke、
少量 BC/DMC 更新和主要的并行对局生成，只是神经网络 learner 会慢很多。

建议先用 5–20 万个启发式 decision（通常是数千局）验证 BC，再逐步累计百万到千万级自博弈
decision。无需人工标注数据。实际磁盘量依赖 history 长度和对局复杂度，应先跑 100–1000 局，
按生成的 `.jsonl.gz` 实测外推；不要一开始承诺固定 TB 级预算。本机 smoke 在 history=96 时
258 个 decision 占 98 KiB（约 0.38 KiB/decision，即百万 decision 约 0.4 GiB）；复杂卡组会更大，
所以这只是起始量级，不是上限。默认经验池最多保留 50 万个
内存样本，373 GiB 内存的当前机器足够，但 Python 原始 dict 并不紧凑，正式长跑应根据实测
降低容量或后续切换二进制 replay。当前 Python 3.14 的默认 Torch wheel 连 CUDA 运行库使虚拟环境
约 4.6 GiB；这是安装空间而非训练数据或显存占用。

## 防止“只会一套牌”的评估

训练 DeckPool 同时抽样原始套牌、随机替换约 20% 卡的扰动套牌，以及职业合法池随机套牌。
推荐继续补充真实强度套牌列表，并把训练/验证/测试按“完整套牌”拆分：验证和测试套牌不能只是
训练套牌换一个随机种子。还应额外留出一组卡，在训练期间不进入套牌，用它们构成探针套牌，
衡量结构/Lua 特征是否真的带来卡牌迁移，而不只是记住 card ID。
