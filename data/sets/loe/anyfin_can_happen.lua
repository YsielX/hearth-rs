local function is_murloc(ctx, card_id)
    for _, tag in ipairs(ctx:card_definition(card_id).tags or {}) do
        if tag == "murloc" or tag == "all" then return true end
    end
    return false
end

local function dead_murlocs(ctx, self)
    local player = ctx:controller(self)
    local result = {}
    for _, owner in ipairs({ player, ctx:opponent(player) }) do
        for _, card_id in ipairs(ctx:minions_died(owner)) do
            if is_murloc(ctx, card_id) then result[#result + 1] = card_id end
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "LOE_026",
    name = "Anyfin Can Happen",
    text = "Summon 7 Murlocs that died this game.",
    set = "LOE",
    type = "spell",
    class = "paladin",
    rarity = "rare",
    cost = 10,
    rules = {
        can_play = function(ctx, self, current)
            return current and #ctx:board(ctx:controller(self)) < 7
        end,
    },
}

function card.on_play(ctx, self)
    ctx:set_data(self, "summoned", 0)
    local counts = {}
    for _, card_id in ipairs(dead_murlocs(ctx, self)) do
        counts[card_id] = (counts[card_id] or 0) + 1
    end
    for card_id, count in pairs(counts) do
        ctx:set_data(self, "eligible_" .. card_id, count)
        ctx:set_data(self, "selected_" .. card_id, 0)
    end
    ctx:continue_with("choose_next_murloc")
end

function card.choose_next_murloc(ctx, self)
    local player = ctx:controller(self)
    if ctx:get_data(self, "summoned") >= 7 or #ctx:board(player) >= 7 then return end

    local skipped, available = {}, {}
    for _, card_id in ipairs(dead_murlocs(ctx, self)) do
        local used = ctx:get_data(self, "selected_" .. card_id)
        local eligible = ctx:get_data(self, "eligible_" .. card_id)
        local seen = (skipped[card_id] or 0) + 1
        skipped[card_id] = seen
        if seen > used and seen <= eligible then available[#available + 1] = card_id end
    end
    if #available > 0 then ctx:random_value(available, "summon_selected_murloc") end
end

function card.summon_selected_murloc(ctx, self, card_id)
    ctx:set_data(self, "selected_" .. card_id, ctx:get_data(self, "selected_" .. card_id) + 1)
    ctx:set_data(self, "summoned", ctx:get_data(self, "summoned") + 1)
    ctx:summon(ctx:controller(self), card_id)
    ctx:continue_with("choose_next_murloc")
end

return card
