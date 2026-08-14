local function is_mech(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "mech" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_055",
    name = "Screwjank Clunker",
    text = "<b>Battlecry:</b> Give a friendly Mech +2/+2.",
    set = "GVG",
    type = "minion",
    class = "warrior",
    rarity = "rare",
    cost = 4,
    attack = 2,
    health = 5,
    tags = { "mech" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local candidates = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if is_mech(ctx, minion) then candidates[#candidates + 1] = minion end
        end
        return candidates
    end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:buff(target, 2, 2) end
    end,
}
