local card = {
    api_version = 1, id = "AT_038", name = "Darnassus Aspirant",
    text = "<b>Battlecry:</b> Gain an empty Mana Crystal.\n<b>Deathrattle:</b> Lose a Mana Crystal.",
    set = "TGT", type = "minion", class = "druid", rarity = "rare",
    cost = 2, attack = 2, health = 3, keywords = { "battlecry", "deathrattle" },
}

function card.on_battlecry(ctx, self)
    ctx:gain_mana_crystals(ctx:controller(self), 1, false)
end

function card.on_deathrattle(ctx, self)
    ctx:destroy_mana_crystals(ctx:controller(self), 1)
end

return card
