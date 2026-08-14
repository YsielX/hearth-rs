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
    id = "BRM_034",
    name = "Blackwing Corruptor",
    text = "<b>Battlecry:</b> If you're holding a Dragon, deal 5 damage.",
    set = "BRM",
    type = "minion",
    rarity = "common",
    cost = 5,
    attack = 5,
    health = 4,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        if holding_dragon(ctx, self) then return ctx:characters() end
        return {}
    end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:damage(target, 5) end
    end,
}
