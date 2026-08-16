return {
    api_version = 1, id = "OG_147", name = "Corrupted Healbot",
    text = "<b>Deathrattle:</b> Restore #8 Health to the enemy hero.",
    set = "OG", type = "minion", rarity = "rare", cost = 5, attack = 6, health = 6,
    tags = { "mech" }, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local enemy = ctx:opponent(ctx:controller(self))
        cardlib.effects.heal(ctx, ctx:player(enemy).hero, 8)
    end,
}
