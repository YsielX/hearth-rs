local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags) do
        if tag == wanted then return true end
    end
    return false
end

local function holding_dragon(ctx, self)
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if has_tag(ctx:card_definition(ctx:entity(entity).card_id), "dragon") then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "BRM_029",
    name = "Rend Blackhand",
    text = "<b>Battlecry:</b> If you're holding a Dragon, destroy a <b>Legendary</b> minion.",
    set = "BRM",
    type = "minion",
    rarity = "legendary",
    cost = 7,
    attack = 8,
    health = 4,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        if not holding_dragon(ctx, self) then return {} end
        local candidates = {}
        for _, minion in ipairs(ctx:minions()) do
            if ctx:card_definition(ctx:entity(minion).card_id).rarity == "legendary" then
                candidates[#candidates + 1] = minion
            end
        end
        return candidates
    end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:destroy(target) end
    end,
}
