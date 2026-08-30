# 卡牌資料來源

[English](../README.md) | [簡體中文](../zhCN/README.md) | [繁體中文](README.md)

`selected.enUS.json` 是截至 2026-08-30 的 1921 個已實現官方定義的預設英文來源快照，資料來自：

```text
https://api.hearthstonejson.com/v1/latest/enUS/cards.json
```

`selected.zhCN.json` 保留對應的簡體中文來源記錄。原始資料由 HearthstoneJSON 提供；倉庫只保留已實現卡牌及衍生物，以及 ID、名稱、正文、set、型別、職業、數值、種族、法術派系和 mechanics 等相關欄位。

`data/locales/enUS.json`、`zhCN.json` 和 `zhTW.json` 來自同一客戶端資料版本。測試要求每個已實現 ID 在三份目錄中都有非空名稱，並將每個 Lua 定義與英文來源快照逐項比較。

重新整理 locale 目錄和簡中來源記錄後執行：

```bash
python3 scripts/sync_lua_fallbacks.py
```

指令碼會把所有 Lua `name`/`text` 後備文字同步為英文、規範關鍵詞英文名，並重新生成 `selected.enUS.json`。衍生物必須從完整 `cards.json` 取數，不能使用只包含可收集牌的 `cards.collectible.json`。
