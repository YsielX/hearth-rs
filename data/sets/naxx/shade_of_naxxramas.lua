return {
    api_version = 1,
    id = "FP1_005",
    name = "Shade of Naxxramas",
    text = "<b><b>Stealth</b>.</b> At the start of your turn, gain +1/+1.",
    set = "NAXX",
    type = "minion",
    rarity = "epic",
    cost = 3,
    attack = 2,
    health = 2,
    tags = { "undead" },
    keywords = { "stealth" },
    triggers = {
        {
            event = "turn_started",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                ctx:buff(self, 1, 1)
            end,
        },
    },
}
