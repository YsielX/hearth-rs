return {
    api_version = 1,
    id = "GIL_622",
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
        ctx:damage(ctx:player(ctx:opponent(player)).hero, 3)
        ctx:heal(ctx:player(player).hero, 3)
    end,
}
