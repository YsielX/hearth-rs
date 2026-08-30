return {
    api_version = 1,
    id = "NEW1_026", rarity = "rare",
    name = "Violet Teacher",
    text = "Whenever you cast a spell, summon a 1/1 Violet Apprentice.",
    set = "EXPERT1",
    type = "minion",
    cost = 4,
    attack = 3,
    health = 5,
    triggers = {
        {
            event = "spell_cast",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self) and event.player_cast
            end,
            effect = function(ctx, self)
                ctx:summon(ctx:controller(self), "NEW1_026t")
            end,
        },
    },
    tokens = {
        {
            id = "NEW1_026t",
            name = "Violet Apprentice",
            set = "EXPERT1",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
        },
    },
}
