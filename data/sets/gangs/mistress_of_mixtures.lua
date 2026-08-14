return {
    api_version = 1,
    id = "CFM_120",
    name = "Mistress of Mixtures",
    text = "<b>Deathrattle:</b> Restore #4 Health to each hero.",
    set = "GANGS",
    type = "minion",
    cost = 1,
    attack = 2,
    health = 2,
    tags = { "undead" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        ctx:heal(ctx:player(0).hero, 4)
        ctx:heal(ctx:player(1).hero, 4)
    end,
}
