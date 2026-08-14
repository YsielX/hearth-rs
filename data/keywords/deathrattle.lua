return {
    api_version = 1,
    module_type = "keyword",
    id = "deathrattle",
    name = "Deathrattle",

    required_card_hooks = { "on_deathrattle" },
    triggers = {
        {
            event = "entity_died",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return event.entity == self
            end,
            effect = function(ctx, self, event)
                for _ = 1, event.repetitions do
                    ctx:continue_with_number("on_deathrattle", event.position)
                end
            end,
        },
        {
            event = "weapon_destroyed",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return event.entity == self
            end,
            effect = function(ctx, self)
                ctx:continue_with("on_deathrattle")
            end,
        },
    },
}
