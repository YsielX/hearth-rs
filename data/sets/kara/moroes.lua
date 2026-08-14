return {
    api_version = 1,
    id = "KAR_044",
    name = "Moroes",
    text = "<b>Stealth</b>\nAt the end of your turn, summon a 1/1 Steward.",
    set = "KARA",
    type = "minion",
    rarity = "legendary",
    cost = 3,
    attack = 1,
    health = 1,
    keywords = { "stealth" },
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                ctx:summon(ctx:controller(self), "KAR_044a")
            end,
        },
    },
    tokens = {
        {
            id = "KAR_044a",
            name = "Steward",
            text = "",
            set = "KARA",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
        },
    },
}
