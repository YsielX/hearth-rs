return {
    api_version = 1,
    id = "GVG_039",
    name = "Vitality Totem",
    text = "At the end of your turn, restore #4 Health to your hero.",
    set = "GVG",
    type = "minion",
    class = "shaman",
    rarity = "rare",
    cost = 2,
    attack = 0,
    health = 3,
    tags = { "totem" },
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                local player = ctx:controller(self)
                cardlib.effects.heal(ctx, ctx:player(player).hero, 4)
            end,
        },
    },
}
