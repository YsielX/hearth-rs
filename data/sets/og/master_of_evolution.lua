local function pool(ctx, cost)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == cost then result[#result + 1] = card_id end
    end
    return result
end

local function evolved_cost(cost)
    if cost < 10 then return cost + 1 end
    return cost
end

local card = {
    api_version = 1, id = "OG_328", name = "Master of Evolution",
    text = "<b>Battlecry:</b> Transform a friendly minion into a random one that costs (1) more.",
    set = "OG", type = "minion", class = "shaman", rarity = "rare", cost = 4,
    attack = 4, health = 5, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            local dormant = false
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if not dormant then result[#result + 1] = minion end
        end
        return result
    end,
}
function card.on_battlecry(ctx, self, target)
    if not target then return end
    local candidates = pool(ctx, evolved_cost(ctx:entity(target).cost))
    if #candidates > 0 then
        ctx:set_data(self, "evolution_target", target)
        ctx:random_value(candidates, "finish_evolution")
    end
end
function card.finish_evolution(ctx, self, card_id)
    ctx:transform(ctx:get_data(self, "evolution_target"), card_id)
end
return card
