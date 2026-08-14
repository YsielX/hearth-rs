return {
    api_version = 1,
    id = "GVG_014",
    name = "Vol'jin",
    text = "<b>Battlecry:</b> Swap Health with another minion.",
    set = "GVG",
    type = "minion",
    class = "priest",
    rarity = "legendary",
    cost = 5,
    attack = 6,
    health = 2,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:minions()) do
            if minion ~= self then targets[#targets + 1] = minion end
        end
        return targets
    end,
    on_battlecry = function(ctx, self, target)
        if target == nil then return end
        local own_health = ctx:entity(self).health
        local target_health = ctx:entity(target).health
        ctx:set_health(self, target_health)
        ctx:set_health(target, own_health)
    end,
}
