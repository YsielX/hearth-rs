return {
    api_version = 1,
    id = "KAR_035",
    name = "Priest of the Feast",
    text = "Whenever you cast a spell, restore #3 Health to\nyour hero.",
    set = "KARA",
    type = "minion",
    class = "priest",
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 6,
    triggers = {{
        event = "spell_cast",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self)
            local player = ctx:controller(self)
            cardlib.effects.heal(ctx, ctx:player(player).hero, 3)
        end,
    }},
}
