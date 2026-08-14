return {
    api_version = 1, module_type = "keyword", id = "dredge", name = "Dredge",
    required_card_hooks = { "on_dredge" },
    hooks = { on_play = function(ctx, self) ctx:continue_with("on_dredge") end },
}
