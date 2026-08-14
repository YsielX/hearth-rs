return {
    api_version = 1, module_type = "keyword", id = "frenzy", name = "Frenzy",
    required_card_hooks = { "on_frenzy" },
    triggers = {
        {
            event = "damaged", timing = "after", active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > 0 and ctx:entity(self).health > 0
            end,
            effect = function(ctx, self, event)
                ctx:disable_keyword(self, "frenzy")
                ctx:continue_with_entity("on_frenzy", event.source)
            end,
        },
    },
}
