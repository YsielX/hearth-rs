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
    id = "BRM_033",
    name = "Blackwing Technician",
    text = "<b>Battlecry:</b> If you're holding a Dragon, gain +1/+1.",
    set = "BRM",
    type = "minion",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 4,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        if holding_dragon(ctx, self) then ctx:buff(self, 1, 1) end
    end,
}
