return {
    api_version = 1, module_type = "keyword", id = "rewind", name = "Rewind",
    required_card_hooks = { "on_rewind" },
    hooks = { on_play = function(ctx, self) ctx:continue_with("on_rewind") end },
}
