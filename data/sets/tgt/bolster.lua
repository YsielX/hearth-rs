return {
    api_version = 1, id = "AT_068", name = "Bolster",
    text = "Give your <b>Taunt</b> minions +2/+2.", set = "TGT", type = "spell",
    class = "warrior", rarity = "common", cost = 2,
    on_play = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "taunt" then cardlib.effects.buff(ctx, minion, 2, 2) break end
            end
        end
    end,
}
