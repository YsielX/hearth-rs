return {
    api_version = 1, id = "CFM_604", name = "Greater Healing Potion",
    text = "Restore #12 Health to a friendly character. Draw a card.",
    set = "GANGS", type = "spell", class = "priest", rarity = "rare", spell_school = "holy",
    cost = 4, target_mode = "required",
    targets = function(ctx, self)
        local result = { ctx:player(ctx:controller(self)).hero }
        for _, minion in ipairs(ctx:friendly_minions(self)) do result[#result + 1] = minion end
        return result
    end,
    on_play = function(ctx, self, target)
        ctx:heal(target, 12)
        ctx:draw(ctx:controller(self), 1)
    end,
}
