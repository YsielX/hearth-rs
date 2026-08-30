local function opponent_cards(ctx, self)
    local class = ctx:player(ctx:opponent(ctx:controller(self))).class
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(card_id).class == class then result[#result + 1] = card_id end
    end
    return result
end

local card = {
    api_version = 1,
    id = "AT_033",
    name = "Burgle",
    text = "Get 3 random\ncards <i>(from your\nopponent's class)</i>.",
    set = "TGT",
    type = "spell",
    class = "rogue",
    rarity = "rare",
    cost = 3,
}

function card.on_play(ctx, self)
    ctx:set_data(self, "cards_added", 0)
    local pool = opponent_cards(ctx, self)
    if #pool > 0 then ctx:random_value(pool, "add_opponent_card") end
end

function card.add_opponent_card(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
    local added = ctx:get_data(self, "cards_added") + 1
    ctx:set_data(self, "cards_added", added)
    if added < 3 then
        local pool = opponent_cards(ctx, self)
        if #pool > 0 then ctx:random_value(pool, "add_opponent_card") end
    end
end

return card
