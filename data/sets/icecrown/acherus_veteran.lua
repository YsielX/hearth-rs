return {
    api_version = 1, id = "ICC_092", name = "Acherus Veteran",
    text = "<b>Battlecry:</b> Give a friendly minion +1 Attack.",
    set = "ICECROWN", type = "minion", rarity = "common",
    cost = 1, attack = 2, health = 1, tags = { "undead" }, keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self then result[#result + 1] = minion end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target) if target ~= nil then cardlib.effects.buff(ctx, target, 1, 0) end end,
}
