local function is_mech(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "mech" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_036",
    name = "Powermace",
    text = "<b>Deathrattle:</b> Give a random friendly Mech +2/+2.",
    set = "GVG",
    type = "weapon",
    class = "shaman",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 2,
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local candidates = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if is_mech(ctx, minion) then candidates[#candidates + 1] = minion end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_mech") end
end

function card.buff_mech(ctx, self, target)
    ctx:buff(target, 2, 2)
end

return card
