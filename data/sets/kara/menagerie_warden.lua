local function is_beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "KAR_065",
    name = "Menagerie Warden",
    text = "<b>Battlecry:</b> Choose a friendly Beast. Summon a copy of it.",
    set = "KARA",
    type = "minion",
    class = "druid",
    rarity = "common",
    cost = 5,
    attack = 5,
    health = 4,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self and is_beast(ctx, minion) then
                result[#result + 1] = minion
            end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target)
        if target then ctx:summon_copy(ctx:controller(self), target) end
    end,
}
