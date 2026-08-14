return {
    api_version = 1, module_type = "keyword", id = "gigantify", name = "Gigantify",
    required_card_hooks = { "on_gigantify" },
    hooks = { on_play = function(ctx, self) ctx:continue_with("on_gigantify") end },
}
