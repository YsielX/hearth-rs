return {
    api_version = 1,
    id = "EX1_029",
    name = "Leper Gnome",
    text = "<b>Deathrattle:</b> Deal 2 damage to the enemy hero.",
    set = "EXPERT1",
    type = "minion",
    rarity = "common",
    cost = 1,
    attack = 2,
    health = 1,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local opponent = ctx:opponent(ctx:controller(self))
        cardlib.effects.damage(ctx, ctx:player(opponent).hero, 2)
    end,
}
