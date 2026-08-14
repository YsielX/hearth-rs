local function has_deathrattle(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do if keyword == "deathrattle" then return true end end
    return false
end
return { api_version = 1, id = "UNG_800", name = "Terrorscale Stalker",
    text = "<b>Battlecry:</b> Trigger a friendly minion's <b>Deathrattle</b>.", set = "UNGORO",
    type = "minion", class = "hunter", rarity = "rare", cost = 2, attack = 2, health = 3,
    keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self and has_deathrattle(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target)
        if target then ctx:trigger_hook(target, "on_deathrattle", ctx:board_position(target)) end
    end }
