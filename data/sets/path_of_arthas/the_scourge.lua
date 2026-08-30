local function is_undead(definition)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == "undead" or tag == "all" then return true end
    end
    return false
end

local function undead_pool(ctx)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and is_undead(definition) then
            pool[#pool + 1] = card_id
        end
    end
    return pool
end

local card = {
    api_version = 1,
    id = "RLK_122",
    name = "The Scourge",
    text = "Fill your board with random Undead.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "legendary",
    spell_school = "shadow",
    cost = 9,
    rune_cost = { unholy = 2 },
}

function card.on_play(ctx, self)
    ctx:continue_with("summon_next_undead")
end

function card.summon_next_undead(ctx, self)
    local player = ctx:controller(self)
    if #ctx:board(player) >= 7 then return end
    local pool = undead_pool(ctx)
    if #pool > 0 then ctx:random_value(pool, "undead_selected") end
end

function card.undead_selected(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
    ctx:continue_with("summon_next_undead")
end

return card
