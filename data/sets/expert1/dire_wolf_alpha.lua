return {
    api_version = 1,
    id = "EX1_162",
    name = "Dire Wolf Alpha",
    text = "Adjacent minions have +1 Attack.",
    set = "EXPERT1",
    type = "minion",
    cost = 2,
    attack = 2,
    health = 2,
    tags = { "beast" },

    auras = {
        {
            attack = 1,
            targets = function(ctx, self)
                return ctx:adjacent_minions(self)
            end,
        },
    },
}
