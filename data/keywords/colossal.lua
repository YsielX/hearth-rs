return {
    api_version = 1, module_type = "keyword", id = "colossal", name = "Colossal",
    requires_param = true,
    required_card_hooks = { "on_colossal" },
    triggers = {
        {
            event = "minion_summoned", timing = "after", active_zones = { "board" },
            condition = function(ctx, self, event) return event.entity == self end,
            effect = function(ctx, self, event) ctx:continue_with("on_colossal") end,
        },
    },
}
