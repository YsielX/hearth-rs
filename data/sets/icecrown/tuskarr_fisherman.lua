return {
    api_version = 1, id = "ICC_093", name = "Tuskarr Fisherman",
    text = "<b>Battlecry:</b> Give a friendly minion <b>Spell Damage +1</b>.",
    set = "ICECROWN", type = "minion", rarity = "common",
    cost = 2, attack = 2, health = 3, tags = { "undead" }, keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self then result[#result + 1] = minion end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:modify(target, { stat = "spell_damage", operation = "add", value = 1 }) end
    end,
}
