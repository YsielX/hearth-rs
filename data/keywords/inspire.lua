return {
    api_version = 1, module_type = "keyword", id = "inspire", name = "Inspire",
    required_card_hooks = { "on_inspire" },
    triggers = {
        {
            event = "hero_power_used", timing = "after", active_zones = { "board", "weapon" },
            condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
            effect = function(ctx, self, event)
                ctx:continue_with_entity("on_inspire", event.entity)
            end,
        },
    },
}
