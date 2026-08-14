return {
    api_version = 1, id = "UNG_950", name = "Vinecleaver",
    text = "[x]After your hero attacks,\nsummon two {0} Silver\nHand Recruits.",
    set = "UNGORO", type = "weapon", class = "paladin", rarity = "rare",
    cost = 6, attack = 4, health = 3,
    triggers = {
        {
            event = "attack", timing = "before", active_zones = { "weapon" },
            condition = function(ctx, self, event)
                return event.attacker == ctx:player(ctx:controller(self)).hero
                    and ctx:player(ctx:controller(self)).weapon == self
            end,
            effect = function(ctx, self, event) ctx:set_data(self, "vinecleaver_attack", event.event_id) end,
        },
        {
            event = "attack", timing = "after", active_zones = { "weapon", "graveyard" },
            condition = function(ctx, self, event)
                return ctx:get_data(self, "vinecleaver_attack") == event.event_id
            end,
            effect = function(ctx, self)
                ctx:set_data(self, "vinecleaver_attack", 0)
                local player = ctx:controller(self)
                ctx:summon(player, "CS2_101t")
                ctx:summon(player, "CS2_101t")
            end,
        },
    },
}
