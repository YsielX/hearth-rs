return {
    api_version = 1, module_type = "keyword", id = "echo", name = "Echo",
    required_card_hooks = { "on_echo" },
    hooks = { on_play = function(ctx, self) ctx:continue_with("on_echo") end },
}
