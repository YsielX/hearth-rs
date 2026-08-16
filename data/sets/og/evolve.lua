local function minion_pool(ctx, cost)
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
    api_version = 1, id = "OG_027", name = "Evolve",
    text = "Transform your minions into random minions that cost (1) more.", set = "OG",
    type = "spell", class = "shaman", rarity = "rare", cost = 1, spell_school = "nature",
}

function card.on_play(ctx, self)
    ctx:set_data(self, "evolve_index", 1)
    ctx:continue_with("evolve_next")
end

function card.evolve_next(ctx, self)
    local index = ctx:get_data(self, "evolve_index") or 1
    local board = ctx:board(ctx:controller(self))
    if index > #board then return end
    local target = board[index]
    for _, keyword in ipairs(ctx:entity(target).keywords) do
        if keyword == "dormant" then
            ctx:set_data(self, "evolve_index", index + 1)
            ctx:continue_with("evolve_next")
            return
        end
    end
    ctx:set_data(self, "evolve_target", target)
    local pool = minion_pool(ctx, evolved_cost(ctx:entity(target).cost))
    if #pool > 0 then ctx:random_value(pool, "finish_evolve")
    else
        ctx:set_data(self, "evolve_index", index + 1)
        ctx:continue_with("evolve_next")
    end
end

function card.finish_evolve(ctx, self, card_id)
    cardlib.effects.transform(ctx, ctx:get_data(self, "evolve_target"), card_id)
    ctx:set_data(self, "evolve_index", (ctx:get_data(self, "evolve_index") or 1) + 1)
    ctx:continue_with("evolve_next")
end

return card
