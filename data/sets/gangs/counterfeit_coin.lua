return {
    api_version = 1, id = "CFM_630", name = "Counterfeit Coin",
    text = "Gain 1 Mana Crystal this turn only.",
    set = "GANGS", type = "spell", class = "rogue", rarity = "rare", cost = 0,
    on_play = function(ctx, self) ctx:gain_temporary_mana(ctx:controller(self), 1) end,
}
