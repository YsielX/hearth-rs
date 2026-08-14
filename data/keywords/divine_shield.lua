return {
    api_version = 1,
    module_type = "keyword",
    id = "divine_shield",
    name = "Divine Shield",
    triggers = {
        {
            event = "damaged",
            timing = "before",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > 0
            end,
            effect = function(ctx, self, event)
                ctx:disable_keyword(self, "divine_shield")
                ctx:cancel_event(event)
            end,
        },
    },
}
