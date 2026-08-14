return {
    api_version = 1, module_type = "keyword", id = "shatter", name = "Shatter",
    required_card_hooks = { "on_shatter" },
    triggers = {
        {
            event = "card_drawn", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event) return event.entity == self end,
            effect = function(ctx, self, event) ctx:continue_with("on_shatter") end,
        },
        {
            event = "card_created", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event) return event.entity == self end,
            effect = function(ctx, self, event) ctx:continue_with("on_shatter") end,
        },
    },
}
