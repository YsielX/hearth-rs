return {
    api_version = 1,
    id = "GIL_622", rarity = "rare",
    name = "Lifedrinker",
    text = "[x]<b>Battlecry:</b> Deal 3 damage to\nthe enemy hero. Restore\n#3 Health to your hero.",
    set = "GILNEAS",
    type = "minion",
    cost = 4,
    attack = 3,
    health = 3,
    tags = { "beast" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        cardlib.effects.damage(ctx, ctx:player(ctx:opponent(player)).hero, 3)
        cardlib.effects.heal(ctx, ctx:player(player).hero, 3)
    end,
}
