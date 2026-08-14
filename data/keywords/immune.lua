return {
    api_version = 1,
    module_type = "keyword",
    id = "immune",
    name = "Immune",
    rules = {
        can_be_attacked = function(ctx, self, current, attacker)
            return false
        end,
        can_be_targeted_by_enemy = function(ctx, self, current, source)
            return false
        end,
    },
    triggers = {
        {
            event = "damaged",
            timing = "before",
            active_zones = { "hero", "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > 0
            end,
            effect = function(ctx, self, event)
                ctx:cancel_event(event)
            end,
        },
    },
}
