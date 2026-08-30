return {
    api_version = 1,
    id = "DAL_378",
    name = "Unleash the Beast",
    text = "<b>Twinspell</b>\nSummon a 5/5 Wyvern with <b>Rush</b>.",
    set = "DALARAN",
    type = "spell",
    rarity = "rare",
    class = "hunter",
    cost = 6,
    keywords = { "twinspell" },
    on_play = function(ctx, self)
        ctx:summon(ctx:controller(self), "DAL_378t1")
    end,
    on_twinspell = function(ctx, self)
        cardlib.effects.give_card(ctx, ctx:controller(self), "DAL_378ts")
    end,
    tokens = {
        {
            id = "DAL_378t1",
            name = "Wyvern",
            text = "<b>Rush</b>",
            set = "DALARAN",
            type = "minion",
            class = "hunter",
            tags = { "beast" },
            cost = 5,
            attack = 5,
            health = 5,
            keywords = { "rush" },
        },
        {
            id = "DAL_378ts",
            name = "Unleash the Beast",
            text = "Summon a 5/5 Wyvern with <b>Rush</b>.",
            set = "DALARAN",
            type = "spell",
            rarity = "rare",
            class = "hunter",
            cost = 6,
            on_play = function(ctx, self)
                ctx:summon(ctx:controller(self), "DAL_378t1")
            end,
        },
    },
}
