return {
    api_version = 1, id = "CFM_646", name = "Backstreet Leper",
    text = "[x]<b>Deathrattle:</b> Deal 2 damage\nto the enemy hero.",
    set = "GANGS", type = "minion", rarity = "common", cost = 3, attack = 3, health = 1,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        cardlib.effects.damage(ctx, ctx:player(ctx:opponent(ctx:controller(self))).hero, 2)
    end,
}
