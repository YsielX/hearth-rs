return {
    api_version = 1, module_type = "keyword", id = "corrupt", name = "Corrupt",
    required_card_hooks = { "on_corrupt" },
    triggers = {
        {
            event = "card_played", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and event.entity ~= self
                    and ctx:entity(event.entity).cost > ctx:entity(self).cost
            end,
            effect = function(ctx, self, event)
                ctx:disable_keyword(self, "corrupt")
                ctx:continue_with("on_corrupt")
            end,
        },
    },
}
