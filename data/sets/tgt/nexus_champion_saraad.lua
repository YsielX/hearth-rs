local card = {
    api_version = 1, id = "AT_127", name = "Nexus-Champion Saraad",
    text = "<b>Inspire:</b> Add a random spell to your hand.", set = "TGT", type = "minion",
    rarity = "legendary", cost = 5, attack = 4, health = 5, keywords = { "inspire" },
}

function card.on_inspire(ctx, self)
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(card_id).type == "spell" then candidates[#candidates + 1] = card_id end
    end
    if #candidates > 0 then ctx:random_value(candidates, "add_random_spell") end
end

function card.add_random_spell(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
