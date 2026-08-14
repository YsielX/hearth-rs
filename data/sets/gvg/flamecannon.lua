local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_001",
    name = "Flamecannon",
    text = "Deal $4 damage to a random enemy minion.",
    set = "GVG",
    type = "spell",
    class = "mage",
    spell_school = "fire",
    rarity = "common",
    cost = 2,
    rules = {
        can_play = function(ctx, self, current)
            if not current then return false end
            for _, entity in ipairs(ctx:enemy_characters(self)) do
                if ctx:entity(entity).type == "minion" and not is_dormant(ctx, entity) then return true end
            end
            return false
        end,
    },
}

function card.on_play(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:enemy_characters(self)) do
        if ctx:entity(entity).type == "minion" and not is_dormant(ctx, entity) then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "hit_minion") end
end

function card.hit_minion(ctx, self, target)
    ctx:damage(target, 4)
end

return card
