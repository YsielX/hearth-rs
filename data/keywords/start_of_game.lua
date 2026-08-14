return {
    api_version = 1, module_type = "keyword", id = "start_of_game", name = "Start of Game",
    required_card_hooks = { "on_start_of_game" },
    triggers = {
        {
            event = "game_started", timing = "after", active_zones = { "deck" },
            condition = function(ctx, self, event) return true end,
            effect = function(ctx, self, event) ctx:continue_with("on_start_of_game") end,
        },
    },
}
