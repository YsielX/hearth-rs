return {
    api_version = 1, module_type = "keyword", id = "dormant", name = "Dormant",
    rules = {
        can_attack = function(ctx, self, current) return false end,
        can_be_attacked = function(ctx, self, current) return false end,
        can_be_targeted = function(ctx, self, current) return false end,
        can_be_randomly_selected = function(ctx, self, current) return false end,
        can_be_destroyed = function(ctx, self, current) return false end,
        can_be_silenced = function(ctx, self, current) return false end,
        can_be_frozen = function(ctx, self, current) return false end,
    },
    triggers = {
        {
            event = "damaged",
            timing = "before",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self
            end,
            effect = function(ctx, self, event)
                ctx:cancel_event(event)
            end,
        },
    },
}
