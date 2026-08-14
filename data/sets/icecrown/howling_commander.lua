local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
        if keyword == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1, id = "ICC_801", name = "Howling Commander",
    text = "<b>Battlecry:</b> Draw a <b>Divine Shield</b> minion from your deck.",
    set = "ICECROWN", type = "minion", class = "paladin", rarity = "rare",
    cost = 3, attack = 2, health = 2, tags = { "undead" }, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local pool = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" and has_keyword(ctx, entity, "divine_shield") then
            pool[#pool + 1] = entity
        end
    end
    if #pool > 0 then ctx:random_entity(pool, "draw_shielded_minion") end
end

function card.draw_shielded_minion(ctx, self, entity)
    ctx:draw_entity(ctx:controller(self), entity)
end

return card
