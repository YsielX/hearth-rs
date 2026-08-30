local function is_beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "CFM_334",
    name = "Smuggler's Crate",
    text = "Give a random Beast in your hand +2/+2.",
    set = "GANGS",
    type = "spell",
    class = "hunter",
    rarity = "common",
    cost = 0,
}

function card.on_play(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" and is_beast(ctx, entity) then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_beast") end
end

function card.buff_beast(ctx, self, target) cardlib.effects.buff(ctx, target, 2, 2) end

return card
