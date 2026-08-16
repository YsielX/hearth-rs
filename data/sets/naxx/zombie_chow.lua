return {
    api_version = 1,
    id = "FP1_001",
    name = "Zombie Chow",
    text = "<b>Deathrattle:</b> Restore #5 Health to the enemy hero.",
    set = "NAXX",
    type = "minion",
    rarity = "common",
    cost = 1,
    attack = 2,
    health = 3,
    tags = { "undead" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local opponent = ctx:opponent(ctx:controller(self))
        cardlib.effects.heal(ctx, ctx:player(opponent).hero, 5)
    end,
}
