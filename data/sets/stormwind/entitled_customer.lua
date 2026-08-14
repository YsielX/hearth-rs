return {
    api_version = 1,
    id = "SW_089",
    name = "Entitled Customer",
    text = "<b>Battlecry:</b> Deal damage equal to your hand size to all other minions.",
    set = "STORMWIND",
    type = "minion",
    class = "warlock",
    cost = 6,
    attack = 3,
    health = 2,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:minions()) do
            if minion ~= self then targets[#targets + 1] = minion end
        end
        ctx:damage_all(targets, #ctx:hand(ctx:controller(self)))
    end,
}
