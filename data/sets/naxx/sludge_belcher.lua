return {
    api_version = 1,
    id = "FP1_012",
    name = "Sludge Belcher",
    text = "[x]<b>Taunt</b>\n<b>Deathrattle:</b> Summon a\n1/2 Slime with <b>Taunt</b>.",
    set = "NAXX",
    type = "minion",
    rarity = "rare",
    cost = 5,
    attack = 3,
    health = 6,
    tags = { "undead" },
    keywords = { "taunt", "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        cardlib.effects.summon_at(ctx, ctx:controller(self), "FP1_012t", position)
    end,
    tokens = {
        {
            id = "FP1_012t",
            name = "Putrid Slime",
            text = "<b>Taunt</b>",
            set = "NAXX",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 2,
            keywords = { "taunt" },
        },
    },
}
