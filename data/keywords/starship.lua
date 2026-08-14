return {
    api_version = 1, module_type = "keyword", id = "starship", name = "Starship",
    required_card_hooks = { "on_starship_piece" },
    triggers = {
        {
            event = "entity_died", timing = "after", active_zones = { "graveyard" },
            condition = function(ctx, self, event) return event.entity == self end,
            effect = function(ctx, self, event) ctx:continue_with("on_starship_piece") end,
        },
    },
}
