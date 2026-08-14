return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_05bp",
    name = "Steady Shot",
    text = "<b>Hero Power</b>\nDeal $2 damage to the enemy hero.",
    set = "LEGACY",
    class = "hunter",
    cost = 2,
    targets = function(ctx, self)
        local enemy = ctx:opponent(ctx:controller(self))
        local targets = { ctx:player(enemy).hero }
        for _, keyword in ipairs(ctx:entity(self).keywords) do
            if keyword == "hero_power_can_target_minions" then
                for _, minion in ipairs(ctx:board(enemy)) do
                    if ctx:entity(minion).type == "minion" then
                        targets[#targets + 1] = minion
                    end
                end
                break
            end
        end
        return targets
    end,
    on_play = function(ctx, self, target)
        if target == nil then
            local enemy = ctx:opponent(ctx:controller(self))
            target = ctx:player(enemy).hero
        end
        ctx:damage(target, 2)
    end,
}
