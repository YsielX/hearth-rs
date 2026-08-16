return {
    api_version = 1, id = "GVG_069", name = "Antique Healbot",
    text = "<b>Battlecry:</b> Restore #8 Health to your hero.", set = "GVG", type = "minion",
    rarity = "common", cost = 5, attack = 3, health = 3, tags = { "mech" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        cardlib.effects.heal(ctx, ctx:player(player).hero, 8)
    end,
}
