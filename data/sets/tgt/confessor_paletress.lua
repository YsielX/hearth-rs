local function legendary_minions(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.rarity == "legendary" then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "AT_018",
    name = "Confessor Paletress",
    text = "<b>Battlecry and Inspire:</b> Summon a random <b>Legendary</b> minion.",
    set = "TGT",
    type = "minion",
    class = "priest",
    rarity = "legendary",
    cost = 7,
    attack = 5,
    health = 4,
    keywords = { "battlecry", "inspire" },
}

local function summon_legendary(ctx)
    local pool = legendary_minions(ctx)
    if #pool > 0 then ctx:random_value(pool, "summon_legendary") end
end

card.on_battlecry = summon_legendary
card.on_inspire = summon_legendary

function card.summon_legendary(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
