local function is_beast(definition)
    if definition.type ~= "minion" or not definition.collectible then return false end
    for _, tag in ipairs(definition.tags) do
        if tag == "beast" then return true end
    end
    return false
end

local function beast_pool(ctx, maximum_cost, excluded)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if card_id ~= excluded and is_beast(definition) and definition.cost <= maximum_cost then
            result[#result + 1] = card_id
        end
    end
    return result
end

local power = {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_828p",
    name = "Build-A-Beast",
    text = "Craft a custom Zombeast.",
    set = "ICECROWN",
    class = "neutral",
    cost = 2,
}

function power.on_play(ctx, self)
    local pool = beast_pool(ctx, 5, nil)
    if #pool > 0 then
        ctx:discover_cards(ctx:controller(self), "Choose the first Beast", pool, 3, "first_beast")
    end
end

function power.first_beast(ctx, self, first)
    local remaining = 10 - ctx:card_definition(first).cost
    local pool = beast_pool(ctx, remaining, first)
    if #pool > 0 then
        local choices = {}
        for _, second in ipairs(pool) do
            choices[#choices + 1] = { first = first, second = second }
        end
        ctx:choose_options(
            ctx:controller(self),
            "Choose the second Beast",
            (function()
                local options = {}
                for _, choice in ipairs(choices) do
                    local definition = ctx:card_definition(choice.second)
                    options[#options + 1] = { label = definition.name, value = choice }
                end
                return options
            end)(),
            "second_beast"
        )
    end
end

function power.second_beast(ctx, self, choice)
    ctx:give_merged_minion(ctx:controller(self), "ICC_828t", choice.first, choice.second)
end

power.tokens = {
    {
        id = "ICC_828t", name = "Zombeast", text = "{0}\n{1}",
        set = "ICECROWN", type = "minion", class = "hunter",
        cost = 0, attack = 1, health = 1, tags = { "undead", "beast" },
    },
}

return power
