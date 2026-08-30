return {
    api_version = 1, id = "CFM_626", name = "Kabal Talonpriest",
    text = "<b>Battlecry:</b> Give a friendly minion +3 Health.",
    set = "GANGS", type = "minion", class = "priest", rarity = "common",
    cost = 3, attack = 3, health = 4, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:friendly_minions(self)) do if entity ~= self then result[#result + 1] = entity end end
        return result
    end,
    on_battlecry = function(ctx, self, target) if target then cardlib.effects.buff(ctx, target, 0, 3) end end,
}
