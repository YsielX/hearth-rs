return {
    api_version = 1, id = "CFM_657", name = "Kabal Songstealer",
    text = "[x]<b>Battlecry:</b> <b>Silence</b> a minion.",
    set = "GANGS", type = "minion", class = "priest", rarity = "common",
    cost = 5, attack = 5, health = 5, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:characters()) do if ctx:entity(entity).type == "minion" and entity ~= self then result[#result + 1] = entity end end
        return result
    end,
    on_battlecry = function(ctx, self, target) if target then ctx:silence(target) end end,
}
