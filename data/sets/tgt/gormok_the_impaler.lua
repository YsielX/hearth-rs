local function enough_others(ctx, self)
    local count = 0
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self then count = count + 1 end
    end
    return count >= 4
end

return {
    api_version = 1, id = "AT_122", name = "Gormok the Impaler",
    text = "<b>Battlecry:</b> If you have at least 4 other minions, deal 4 damage.", set = "TGT",
    type = "minion", rarity = "legendary", cost = 4, attack = 4, health = 4,
    keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self) if enough_others(ctx, self) then return ctx:characters() end return {} end,
    on_battlecry = function(ctx, self, target) if target ~= nil then ctx:damage(target, 4) end end,
}
