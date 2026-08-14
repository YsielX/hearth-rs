return {
    api_version = 1, id = "GVG_068", name = "Burly Rockjaw Trogg",
    text = "Whenever your opponent casts a spell, gain +2 Attack.", set = "GVG",
    type = "minion", rarity = "common", cost = 4, attack = 3, health = 5,
    triggers = {{
        event = "spell_cast", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player ~= ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self) ctx:buff(self, 2, 0) end,
    }},
}
