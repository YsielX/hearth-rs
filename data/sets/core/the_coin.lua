return {
    api_version = 1,
    id = "GAME_005",
    name = "The Coin",
    text = "Gain 1 Mana Crystal this turn only.",
    set = "CORE",
    type = "spell",
    collectible = false,
    cost = 0,
    on_play = function(ctx, self)
        ctx:gain_temporary_mana(ctx:controller(self), 1)
    end,
}
