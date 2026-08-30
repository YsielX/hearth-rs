return {
    api_version = 1, id = "OG_320", name = "Midnight Drake",
    text = "<b>Battlecry:</b> Gain +1 Attack for each other card\nin your hand.", set = "OG",
    type = "minion", rarity = "rare", cost = 4, attack = 1, health = 4, tags = { "dragon" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local count = #ctx:hand(ctx:controller(self))
        if count > 0 then cardlib.effects.buff(ctx, self, count, 0) end
    end,
}
