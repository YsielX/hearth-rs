local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do if keyword == wanted then return true end end
    return false
end

return {
    api_version = 1, id = "CFM_694", name = "Shadow Sensei",
    text = "<b>Battlecry:</b> Give a <b>Stealthed</b> minion +2/+2.",
    set = "GANGS", type = "minion", class = "rogue", rarity = "rare",
    cost = 4, attack = 4, health = 4, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:friendly_minions(self)) do
            if entity ~= self and has_keyword(ctx, entity, "stealth") then result[#result + 1] = entity end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target) if target then cardlib.effects.buff(ctx, target, 2, 2) end end,
}
