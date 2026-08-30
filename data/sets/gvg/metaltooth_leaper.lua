local function is_mech(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "mech" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_048",
    name = "Metaltooth Leaper",
    text = "<b>Battlecry:</b> Give your other Mechs +2 Attack.",
    set = "GVG",
    type = "minion",
    class = "hunter",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "mech", "beast" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self and is_mech(ctx, minion) then cardlib.effects.buff(ctx, minion, 2, 0) end
    end
end

return card
