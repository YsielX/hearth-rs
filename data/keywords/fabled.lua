return {
    api_version = 1, module_type = "keyword", id = "fabled", name = "Fabled",
    required_card_hooks = { "on_fabled" },
    triggers = {
        {
            event = "game_started", timing = "after", active_zones = { "deck" },
            condition = function(ctx, self, event) return true end,
            effect = function(ctx, self, event) ctx:continue_with("on_fabled") end,
        },
    },
}
