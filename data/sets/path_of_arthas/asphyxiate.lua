local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "RLK_087",
    name = "Asphyxiate",
    text = "Destroy the highest Attack enemy minion.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    spell_school = "shadow",
    cost = 3,
}

function card.on_play(ctx, self)
    local highest = nil
    local candidates = {}
    for _, minion in ipairs(ctx:enemy_minions(self)) do
        if not is_dormant(ctx, minion) then
            local attack = ctx:entity(minion).attack
            if highest == nil or attack > highest then
                highest = attack
                candidates = { minion }
            elseif attack == highest then
                candidates[#candidates + 1] = minion
            end
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "destroy_highest") end
end

function card.destroy_highest(ctx, self, target)
    cardlib.effects.destroy(ctx, target)
end

return card
