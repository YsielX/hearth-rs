local function is_mech(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "mech" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_083",
    name = "Upgraded Repair Bot",
    text = "<b>Battlecry:</b> Give a friendly Mech +4 Health.",
    set = "GVG",
    type = "minion",
    class = "priest",
    rarity = "rare",
    cost = 5,
    attack = 5,
    health = 5,
    tags = { "mech" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if is_mech(ctx, minion) then targets[#targets + 1] = minion end
        end
        return targets
    end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then cardlib.effects.buff(ctx, target, 0, 4) end
    end,
}
