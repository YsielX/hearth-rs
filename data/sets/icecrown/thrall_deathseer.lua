local function minions_costing(ctx, cost)
    local result = {}
    for index, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == cost then
            result[#result + 1] = { card = card_id, index = index }
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
    rarity = "legendary",
    cost = 5,
    health = 30,
    armor = 5,
    hero_power = "ICC_481p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local pools = {}
    for _, target in ipairs(ctx:friendly_minions(self)) do
        local candidates = minions_costing(ctx, ctx:entity(target).cost + 2)
        local choices = {}
        for _, replacement in ipairs(candidates) do
            choices[#choices + 1] = {
                target = target,
                card = replacement.card,
                card_index = replacement.index,
            }
        end
        if #choices > 0 then
            pools[#pools + 1] = choices
        end
    end
    ctx:set_data(self, "evolution_count", #pools)
    ctx:set_data(self, "evolution_chosen", 0)
    for _, choices in ipairs(pools) do
        ctx:random_value(choices, "choose_evolution")
    end
end

function card.choose_evolution(ctx, self, choice)
    local chosen = ctx:get_data(self, "evolution_chosen") + 1
    ctx:set_data(self, "evolution_chosen", chosen)
    ctx:set_data(self, "evolution_target_" .. chosen, choice.target)
    ctx:set_data(self, "evolution_card_" .. chosen, choice.card_index)
    if chosen == ctx:get_data(self, "evolution_count") then
        local transforms = {}
        for index = 1, chosen - 1 do
            transforms[#transforms + 1] = {
                ctx:get_data(self, "evolution_target_" .. index),
                ctx:collectible_cards()[ctx:get_data(self, "evolution_card_" .. index)],
            }
        end
        transforms[#transforms + 1] = { choice.target, choice.card }
        ctx:transform_batch(transforms)
    end
end

return card
