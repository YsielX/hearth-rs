# 卡牌数据来源

[English](../README.md) | [简体中文](README.md) | [繁體中文](../zhTW/README.md)

`selected.enUS.json` 是截至 2026-08-14 的 150 个已实现官方定义的默认英文来源快照，数据来自：

```text
https://api.hearthstonejson.com/v1/latest/enUS/cards.json
```

`selected.zhCN.json` 保留对应的简体中文来源记录。原始数据由 HearthstoneJSON 提供；仓库只保留已实现卡牌及衍生物，以及 ID、名称、正文、set、类型、职业、数值、种族、法术派系和 mechanics 等相关字段。

`data/locales/enUS.json`、`zhCN.json` 和 `zhTW.json` 来自同一客户端数据版本。测试要求每个已实现 ID 在三份目录中都有非空名称，并将每个 Lua 定义与英文来源快照逐项比较。

刷新 locale 目录和简中来源记录后运行：

```bash
python3 scripts/sync_lua_fallbacks.py
```

脚本会把所有 Lua `name`/`text` 后备文本同步为英文、规范关键词英文名，并重新生成 `selected.enUS.json`。衍生物必须从完整 `cards.json` 取数，不能使用只包含可收集牌的 `cards.collectible.json`。
