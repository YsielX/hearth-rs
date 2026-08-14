local function one_cost_minions(ctx)
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 1 then
            candidates[#candidates + 1] = card_id
        end
    end
    return candidates
end

local card = {
    api_version = 1,
    id = "BRM_026",
    name = "Hungry Dragon",
    text = "<b>Battlecry:</b> Summon a random 1-Cost minion for your opponent.",
    set = "BRM",
    type = "minion",
    rarity = "common",
    cost = 4,
    attack = 5,
    health = 6,
    tags = { "dragon" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = one_cost_minions(ctx)
    if #candidates > 0 then ctx:random_value(candidates, "summon_for_opponent") end
end

function card.summon_for_opponent(ctx, self, card_id)
    ctx:summon(ctx:opponent(ctx:controller(self)), card_id)
end

return card
