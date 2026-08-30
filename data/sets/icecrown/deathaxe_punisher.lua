local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
        if keyword == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "ICC_810",
    name = "Deathaxe Punisher",
    text = "<b>Battlecry:</b> Give a random <b>Lifesteal</b> minion in your hand +2/+2.",
    set = "ICECROWN",
    type = "minion",
    rarity = "epic",
    cost = 4,
    attack = 3,
    health = 3,
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" and has_keyword(ctx, entity, "lifesteal") then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_lifesteal_minion") end
end

function card.buff_lifesteal_minion(ctx, self, target)
    cardlib.effects.buff(ctx, target, 2, 2)
end

return card
