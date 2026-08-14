local fusion = {
    api_version = 1,
    module_type = "library",
    id = "fusion",
}

local function append_unique(values, value)
    for _, existing in ipairs(values) do
        if existing == value then return end
    end
    values[#values + 1] = value
end

function fusion.create_minion(ctx, player, template, components, limits)
    limits = limits or {}
    local attack = 0
    local health = 0
    local cost = 0
    local keywords = {}
    for _, card_id in ipairs(components) do
        local definition = ctx:card_definition(card_id)
        if definition.type ~= "minion" then
            error("fusion components must be minions: " .. card_id)
        end
        attack = attack + definition.attack
        health = health + definition.health
        cost = cost + definition.cost
        for _, keyword in ipairs(definition.keywords) do
            append_unique(keywords, keyword)
        end
    end
    ctx:create_card(player, template, {
        attack = attack,
        health = math.max(limits.minimum_health or 1, health),
        cost = math.min(limits.maximum_cost or 10, cost),
        keywords = keywords,
        attached_scripts = components,
    })
end

return fusion
