return {
    api_version = 1,
    module_type = "keyword",
    id = "stealth",
    name = "Stealth",
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
            event = "attack",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.attacker == self
            end,
            effect = function(ctx, self, event)
                ctx:disable_keyword(self, "stealth")
            end,
        },
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.source == self and event.amount > 0
            end,
            effect = function(ctx, self, event)
                ctx:disable_keyword(self, "stealth")
            end,
        },
    },
}
