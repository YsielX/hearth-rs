local card = {
    api_version = 1,
    id = "KAR_069",
    name = "Swashburglar",
    text = "<b>Battlecry:</b> Add a random card from another class to your hand.",
    set = "KARA",
    type = "minion",
    class = "rogue",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 2,
    tags = { "pirate" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local own_class = ctx:player(ctx:controller(self)).class
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.class ~= "neutral" and definition.class ~= own_class then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "on_card") end
end

function card.on_card(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
