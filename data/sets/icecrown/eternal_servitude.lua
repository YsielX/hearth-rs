local function death_pool(ctx, self)
    local seen, pool = {}, {}
    for _, id in ipairs(ctx:minions_died(ctx:controller(self))) do
        if not seen[id] then seen[id] = true; pool[#pool + 1] = id end
    end
    return pool
end

local card = {
    api_version = 1, id = "ICC_213", name = "Eternal Servitude",
    text = "<b>Discover</b> a friendly minion that died this game. Summon it.",
    set = "ICECROWN", type = "spell", class = "priest", rarity = "rare",
    spell_school = "shadow", cost = 4, keywords = { "discover" },
    rules = { can_play = function(ctx, self, current)
        return current and #ctx:board(ctx:controller(self)) < 7 and #death_pool(ctx, self) > 0
    end },
}

function card.on_play(ctx, self)
    local pool = death_pool(ctx, self)
    if #pool > 0 then ctx:discover_cards(ctx:controller(self), "Discover a minion to summon", pool, 3, "serve_eternally") end
end

function card.serve_eternally(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
