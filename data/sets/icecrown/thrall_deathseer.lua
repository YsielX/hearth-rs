local function minions_costing(ctx, cost)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == cost then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "ICC_481",
    name = "Thrall, Deathseer",
    text = "<b>Battlecry:</b> Transform your minions into random ones that cost (2) more.",
    set = "ICECROWN",
    type = "hero",
    class = "shaman",
    cost = 5,
    health = 30,
    armor = 5,
    hero_power = "ICC_481p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    for _, target in ipairs(ctx:friendly_minions(self)) do
        local candidates = minions_costing(ctx, ctx:entity(target).cost + 2)
        local choices = {}
        for _, replacement in ipairs(candidates) do
            choices[#choices + 1] = { target = target, card = replacement }
        end
        if #choices > 0 then ctx:random_value(choices, "evolve") end
    end
end

function card.evolve(ctx, self, choice)
    ctx:transform(choice.target, choice.card)
end

return card
