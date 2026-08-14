return {
    api_version = 1, id = "GVG_118", name = "Troggzor the Earthinator",
    text = "Whenever your opponent casts a spell, summon a Burly Rockjaw Trogg.", set = "GVG",
    type = "minion", rarity = "legendary", cost = 7, attack = 6, health = 6,
    triggers = {{
        event = "spell_cast", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player ~= ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self) ctx:summon(ctx:controller(self), "GVG_068") end,
    }},
}
