return {
    api_version = 1,
    id = "CFM_614",
    name = "Mark of the Lotus",
    text = "Give your minions +1/+1.",
    set = "GANGS",
    type = "spell",
    class = "druid",
    rarity = "common",
    cost = 1,
    on_play = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            local dormant = false
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if not dormant then ctx:buff(minion, 1, 1) end
        end
    end,
}
