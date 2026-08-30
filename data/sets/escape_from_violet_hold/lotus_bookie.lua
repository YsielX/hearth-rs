return {
    api_version = 1,
    id = "JAIL_720", rarity = "common",
    name = "Lotus Bookie",
    text = "<b>Deathrattle:</b> Get a Coin.",
    set = "ESCAPEFROM_VIOLET_HOLD",
    type = "minion",
    class = "rogue",
    cost = 2,
    attack = 2,
    health = 2,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        cardlib.effects.give_card(ctx, ctx:controller(self), "GAME_005")
    end,
}
