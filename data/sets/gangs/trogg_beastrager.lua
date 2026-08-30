local function is_beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "CFM_338",
    name = "Trogg Beastrager",
    text = "<b>Battlecry:</b> Give a random Beast in your hand +1/+1.",
    set = "GANGS",
    type = "minion",
    class = "hunter",
    rarity = "rare",
    cost = 2,
    attack = 3,
    health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" and is_beast(ctx, entity) then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_beast") end
end

function card.buff_beast(ctx, self, target) cardlib.effects.buff(ctx, target, 1, 1) end

return card
