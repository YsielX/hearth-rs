return {
    api_version = 1, module_type = "keyword", id = "twinspell", name = "Twinspell",
    required_card_hooks = { "on_twinspell" },
    hooks = { on_play = function(ctx, self) ctx:continue_with("on_twinspell") end },
}
