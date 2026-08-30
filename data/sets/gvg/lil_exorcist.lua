return {
    api_version = 1, id = "GVG_097", name = "Lil' Exorcist",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> Gain +1/+1 for each enemy <b>Deathrattle</b> minion.",
    set = "GVG", type = "minion", rarity = "rare", cost = 3, attack = 2, health = 3,
    keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        local count = 0
        local enemy = ctx:opponent(ctx:controller(self))
        for _, minion in ipairs(ctx:board(enemy)) do
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "deathrattle" then count = count + 1 break end
            end
        end
        if count > 0 then cardlib.effects.buff(ctx, self, count, count) end
    end,
}
