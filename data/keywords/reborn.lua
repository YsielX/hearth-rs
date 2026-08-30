return {
    api_version = 1,
    module_type = "keyword",
    id = "reborn",
    name = "Reborn",
    triggers = {
        {
            event = "entity_died",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return event.entity == self
            end,
            effect = function(ctx, self, event)
                cardlib.effects.summon_fresh_copy(ctx, self, event.position, 1, { "reborn" })
            end,
        },
    },
}
