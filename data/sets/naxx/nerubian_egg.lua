return {
    api_version = 1,
    id = "FP1_007",
    name = "Nerubian Egg",
    text = "<b>Deathrattle:</b> Summon a 4/4 Nerubian.",
    set = "NAXX",
    type = "minion",
    rarity = "rare",
    cost = 2,
    attack = 0,
    health = 2,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        cardlib.effects.summon_at(ctx, ctx:controller(self), "FP1_007t", position)
    end,
    tokens = {
        {
            id = "FP1_007t",
            name = "Nerubian",
            text = "",
            set = "EXPERT1",
            type = "minion",
            cost = 4,
            attack = 4,
            health = 4,
            tags = { "undead" },
        },
    },
}
