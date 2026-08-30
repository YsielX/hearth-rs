return {
    api_version = 1,
    id = "LOE_012",
    name = "Tomb Pillager",
    text = "<b>Deathrattle:</b> Get a Coin.",
    set = "LOE",
    type = "minion",
    class = "rogue",
    rarity = "common",
    cost = 4,
    attack = 6,
    health = 4,
    tags = { "undead" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        cardlib.effects.give_card(ctx, ctx:controller(self), "GAME_005")
    end,
}
