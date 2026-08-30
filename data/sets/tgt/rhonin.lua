local card = {
    api_version = 1, id = "AT_009", name = "Rhonin",
    text = "<b>Deathrattle:</b> Add 3 copies of Arcane Missiles to your hand.",
    set = "TGT", type = "minion", class = "mage", rarity = "legendary",
    cost = 8, attack = 7, health = 7, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    for _ = 1, 3 do cardlib.effects.give_card(ctx, player, "EX1_277") end
end
return card
