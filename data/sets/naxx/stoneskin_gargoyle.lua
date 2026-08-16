return {
    api_version = 1,
    id = "FP1_027",
    name = "Stoneskin Gargoyle",
    text = "At the start of your turn, restore this minion to full Health.",
    set = "NAXX",
    type = "minion",
    rarity = "common",
    cost = 3,
    attack = 1,
    health = 4,
    tags = { "undead" },
    triggers = {
        {
            event = "turn_started",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:entity(self).damage > 0
            end,
            effect = function(ctx, self, event)
                cardlib.effects.heal(ctx, self, ctx:entity(self).damage)
            end,
        },
    },
}
