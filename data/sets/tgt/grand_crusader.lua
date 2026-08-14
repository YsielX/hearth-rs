local card = {
    api_version = 1, id = "AT_118", name = "Grand Crusader",
    text = "<b>Battlecry:</b> Add a random Paladin card to your hand.", set = "TGT", type = "minion",
    rarity = "epic", cost = 6, attack = 5, health = 5, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(card_id).class == "paladin" then candidates[#candidates + 1] = card_id end
    end
    if #candidates > 0 then ctx:random_value(candidates, "add_paladin_card") end
end

function card.add_paladin_card(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
