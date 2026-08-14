return {
    api_version = 1,
    id = "EX1_001",
    name = "Lightwarden",
    text = "Whenever a character is healed, gain +2 Attack.",
    set = "EXPERT1",
    type = "minion",
    rarity = "rare",
    cost = 1,
    attack = 1,
    health = 2,
    tags = { "draenei" },
    triggers = {
        {
            event = "healed",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.amount > 0
            end,
            effect = function(ctx, self)
                ctx:buff(self, 2, 0)
            end,
        },
    },
}
