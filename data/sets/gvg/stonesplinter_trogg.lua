return {
    api_version = 1, id = "GVG_067", name = "Stonesplinter Trogg",
    text = "Whenever your opponent casts a spell, gain +1 Attack.", set = "GVG",
    type = "minion", rarity = "common", cost = 2, attack = 2, health = 3,
    triggers = {{
        event = "spell_cast", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player ~= ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 1, 0) end,
    }},
}
