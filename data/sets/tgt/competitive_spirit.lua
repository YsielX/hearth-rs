return {
    api_version = 1,
    id = "AT_073",
    name = "Competitive Spirit",
    text = "<b>Secret:</b> When your turn starts, give your minions +1/+1.",
    set = "TGT",
    type = "spell",
    class = "paladin",
    rarity = "rare",
    cost = 1,
    keywords = { "secret" },
    triggers = {
        {
            event = "turn_started",
            timing = "after",
            active_zones = { "secret" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self) and #ctx:friendly_minions(self) > 0
            end,
            effect = function(ctx, self)
                ctx:reveal_secret(self)
                for _, minion in ipairs(ctx:friendly_minions(self)) do ctx:buff(minion, 1, 1) end
            end,
        },
    },
}
