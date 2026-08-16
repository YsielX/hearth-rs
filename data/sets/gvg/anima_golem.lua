return {
    api_version = 1,
    id = "GVG_077",
    name = "Anima Golem",
    text = "At the end of each turn, destroy this minion if it's your only one.",
    set = "GVG",
    type = "minion",
    class = "warlock",
    rarity = "epic",
    cost = 6,
    attack = 9,
    health = 9,
    tags = { "mech" },
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self)
                return #ctx:friendly_minions(self) == 1
            end,
            effect = function(ctx, self)
                cardlib.effects.destroy(ctx, self)
            end,
        },
    },
}
