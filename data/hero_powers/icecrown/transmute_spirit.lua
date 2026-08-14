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

local power = {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_481p",
    name = "Transmute Spirit",
    text = "[x]Transform a friendly\nminion into a random one\nthat costs (1) more.",
    set = "ICECROWN",
    class = "neutral",
    cost = 2,
    target_mode = "required",
    targets = function(ctx, self) return ctx:friendly_minions(self) end,
}

function power.on_play(ctx, self, target)
    local candidates = minions_costing(ctx, ctx:entity(target).cost + 1)
    local choices = {}
    for _, card_id in ipairs(candidates) do
        choices[#choices + 1] = { target = target, card = card_id }
    end
    if #choices > 0 then ctx:random_value(choices, "transmute") end
end

function power.transmute(ctx, self, choice)
    ctx:transform(choice.target, choice.card)
end

return power
