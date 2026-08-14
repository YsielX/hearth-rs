return {
    api_version = 1, module_type = "keyword", id = "overheal", name = "Overheal",
    required_card_hooks = { "on_overheal" },
    triggers = {
        {
            event = "healed", timing = "before", active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > ctx:entity(self).damage
            end,
            effect = function(ctx, self, event)
                ctx:continue_with_number("on_overheal", event.amount - ctx:entity(self).damage)
            end,
        },
    },
}
