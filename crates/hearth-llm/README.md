# hearth-llm

Rust LLM 玩家模块，使用兼容 Chat Completions 的 HTTP 接口。客户端陪玩与训练教师共用同一套请求和标签协议；不依赖 Bevy、Python 或训练框架。

## 陪玩

图形客户端：进入「设置 → LLM 设置」，填写 URL、模型名和 API Key，点击「应用设置」。在选牌页面选择「对战 LLM」后开始对局。

- URL 支持 `https://example.com/v1` 或完整的 `https://example.com/v1/chat/completions`；基础路径不会自动补 `/v1`。
- 本地服务支持 `http://localhost:1234/v1`，无需认证时 Key 可留空。
- URL、模型、JSON 模式和超时保存在客户端设置中。Key 只保留在进程内，不写入设置、存档、回放或 `Debug` 输出。
- 服务不接受 `response_format` 时关闭「JSON 模式」；仍要求回答 JSON，仍校验标签。
- 后台线程进行请求，游戏/Lua 状态留在主线程。对局菜单可暂停、改设置或投降。客户端计时沿用仅对人类输入计时的规则。
- 失败会显示错误，可重试或让启发式 bot 代走一步。重试、换局等操作会丢弃旧回复；已经发出的 HTTP 请求可能继续到完成或超时，不能撤回服务端计费。
- 恢复 LLM 对局后使用当前连接设置；重启后需要重新输入 Key 或通过环境变量提供。

CLI 支持任一方选择 `llm`，例如 LLM 对战现有启发式 bot：

```bash
export HEARTH_LLM_URL='http://localhost:1234/v1'
export HEARTH_LLM_MODEL='your-model-name'
# 需要认证时设置 HEARTH_LLM_API_KEY。
cargo run -p hearth-cli -- play --player-one llm --player-two bot

# 图形客户端也读取相同环境变量
cargo run -p hearth-client-bevy -- --llm
```

两种客户端均支持 `--llm-url URL`、`--llm-model MODEL`、`--llm-key KEY`、`--llm-timeout N`、`--llm-no-json`。优先级为命令行 > 环境变量 > 已保存设置（图形客户端）。默认请求超时 120 秒，可设 1–600 秒。CLI 请求失败会返回错误并结束，不会自动换成启发式动作。

也可以显式导入 `key.env`，无需把 Key 放到命令行：

```bash
cargo run -p hearth-client-bevy -- --llm --llm-env key.env --llm-model YOUR_MODEL
cargo run -p hearth-cli -- play --player-one llm --player-two bot --llm-env key.env --llm-model YOUR_MODEL
```

文件支持 `OPENAI_API_URL`（或 `OPENAI_BASE_URL`）、`OPENAI_API_KEY`、`OPENAI_MODEL`，也支持对应的 `HEARTH_LLM_*` 名称；同一文件内优先使用 `HEARTH_LLM_*`。支持空行、注释、`export NAME=value` 和单/双引号；内容按字面读取，不执行 shell 命令、变量替换或反斜杠转义。文件中的值覆盖已有配置，后续命令行参数可以再覆盖文件，因此把 `--llm-env` 放在覆盖参数之前。不会自动扫描或载入其他 `.env` 文件。

## Rust 接入与训练复用

`DecisionRequest::prepare` 接收当前输入玩家的 `PlayerView`、完整 `LegalAction` 列表、己方初始牌组、卡包 hash 和卡牌定义查询函数。提供可见实体、资源、选项语义、完整的该玩家可见历史及相关卡牌定义；不会读取对手隐藏区域或引擎 RNG。初始牌组转成计数表，不透露未来抽牌顺序。调用方必须保证传入的 view 和合法动作属于同一局面，初始牌组属于该玩家。

`hearth-app::MatchSession` 提供便利接口：

```rust,no_run
# fn example(session: &mut hearth_app::MatchSession) -> Result<(), Box<dyn std::error::Error>> {
let bot = hearth_app::LlmBot::new(hearth_app::LlmConfig::from_env())?;
let request = session.llm_request()?; // 在引擎线程上构造
let decision = bot.decide(&request)?; // 阻塞；可移至线程或自己的任务池
session.dispatch_llm(&decision)?;    // 在引擎线程上再次核对局面后执行
# Ok(())
# }
```

对于 UI，`GameSession::prepare_llm_turn` 返回可移入工作线程的 `(LlmBot, DecisionRequest)`；收到结果后调用 `apply_llm_decision`。请求与游戏状态独立，因此训练调度器可以等待教师时推进其他对局。

返回的 `LlmDecision` 包含：

- `label.request_id`：可见局面、动作映射、卡包及提示版本的指纹。
- `label.action_index`、`acceptable_actions`、`reason`：首选动作、可接受动作集合（包含首选）、简短理由。
- `prompt_version`、请求/返回模型名、响应 ID、服务返回的原始 `usage` 和 `elapsed_ms`。

`DecisionRequest` 与 `LlmDecision` 可序列化后一起作为教师样本保存。请求包含己方私有信息，应当作为训练数据，而非公开观战日志。`request_id` 用来核对局面，不代替训练系统自己的 episode ID、策略版本和样本 ID；相同局面的指纹可以相同。

模型每次只选择一个合法动作；出牌后重新构造请求，因此换牌、发现、选项、目标和站位均经过同一通道。模块只校验响应格式、动作范围和局面对应关系，不用启发式评分否定教师策略。不会自动重试、静默降级或训练学生。异步请求池、预算控制、标签存储及启发式对手调度由后续训练层实现。

协议采用 [Chat Completions JSON 模式](https://developers.openai.com/api/docs/guides/structured-outputs)：JSON 模式不保证业务 schema，因此无论服务是否启用该模式，本地都会执行上述校验。

## 验证

```bash
cargo test -p hearth-llm -p hearth-app -p hearth-cli -p hearth-client-bevy
```

HTTP 测试只使用回环地址上的模拟服务，不需要真实 Key，也不会调用付费模型；运行环境需允许绑定本地临时端口。

真实连接检查需要显式运行下面的示例，会发送两次模型请求（换牌和正常回合），并通过引擎执行返回的合法动作。只输出动作索引、耗时和数值 token 统计，不输出 Key 或原始响应：

```bash
cargo run -p hearth-app --example llm_smoke -- key.env YOUR_MODEL
```
