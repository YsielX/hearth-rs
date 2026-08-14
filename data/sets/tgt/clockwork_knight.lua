local function is_mech(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "mech" or tag == "all" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "AT_096",
    name = "Clockwork Knight",
    text = "<b>Battlecry:</b> Give a friendly Mech +1/+1.",
    set = "TGT",
    type = "minion",
    rarity = "common",
    cost = 5,
    attack = 5,
    health = 5,
    tags = { "mech" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self and is_mech(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:buff(target, 1, 1) end
    end,
}
