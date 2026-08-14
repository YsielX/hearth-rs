local function is_beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1, id = "AT_010", name = "Ram Wrangler",
    text = "<b>Battlecry:</b> If you have a Beast, summon a\nrandom Beast.",
    set = "TGT", type = "minion", class = "hunter", rarity = "rare",
    cost = 5, attack = 3, health = 3, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local has_beast = false
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self and is_beast(ctx, minion) then has_beast = true break end
    end
    if not has_beast or #ctx:board(ctx:controller(self)) >= 7 then return end
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.type == "minion" then
            for _, tag in ipairs(definition.tags) do
                if tag == "beast" or tag == "all" then pool[#pool + 1] = id break end
            end
        end
    end
    if #pool > 0 then ctx:random_value(pool, "summon_beast") end
end

function card.summon_beast(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
