return {
    api_version = 1,
    id = "AT_034",
    name = "Poisoned Blade",
    text = "Your Hero Power gives this +1 Attack instead\nof replacing it.",
    set = "TGT",
    type = "weapon",
    class = "rogue",
    rarity = "epic",
    cost = 2,
    attack = 1,
    health = 3,
    triggers = {
        {
            event = "hero_power_used",
            timing = "before",
            active_zones = { "weapon" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self) ctx:set_data(self, "hero_power_pending", 1) end,
        },
        {
            event = "weapon_destroyed",
            timing = "before",
            active_zones = { "weapon" },
            condition = function(ctx, self, event)
                return event.entity == self and ctx:get_data(self, "hero_power_pending") == 1
            end,
            effect = function(ctx, self, event)
                ctx:set_data(self, "hero_power_pending", 0)
                ctx:cancel_event(event)
                ctx:buff(self, 1, 0)
            end,
        },
        {
            event = "hero_power_used",
            timing = "after",
            active_zones = { "weapon" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self) ctx:set_data(self, "hero_power_pending", 0) end,
        },
    },
}
