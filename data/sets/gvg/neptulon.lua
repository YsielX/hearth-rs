local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags) do
        if tag == wanted then return true end
    end
    return false
end

local function murloc_pool(ctx)
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if has_tag(ctx:card_definition(card_id), "murloc") then
            candidates[#candidates + 1] = card_id
        end
    end
    return candidates
end

local card = {
    api_version = 1,
    id = "GVG_042",
    name = "Neptulon",
    text = "<b>Battlecry:</b> Add 4 random Murlocs to your hand. <b>Overload:</b> (3)",
    set = "GVG",
    type = "minion",
    class = "shaman",
    rarity = "legendary",
    cost = 7,
    attack = 7,
    health = 7,
    tags = { "elemental" },
    keywords = { "battlecry", "overload" },
    keyword_params = { overload = 3 },
}

function card.on_battlecry(ctx, self)
    ctx:set_data(self, "murlocs_added", 0)
    local candidates = murloc_pool(ctx)
    if #candidates > 0 then ctx:random_value(candidates, "add_random_murloc") end
end

function card.add_random_murloc(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
    local added = ctx:get_data(self, "murlocs_added") + 1
    ctx:set_data(self, "murlocs_added", added)
    if added < 4 then
        local candidates = murloc_pool(ctx)
        if #candidates > 0 then ctx:random_value(candidates, "add_random_murloc") end
    end
end

return card
