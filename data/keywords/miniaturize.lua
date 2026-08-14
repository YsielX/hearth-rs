return {
    api_version = 1, module_type = "keyword", id = "miniaturize", name = "Miniaturize",
    required_card_hooks = { "on_miniaturize" },
    hooks = { on_play = function(ctx, self) ctx:continue_with("on_miniaturize") end },
}
