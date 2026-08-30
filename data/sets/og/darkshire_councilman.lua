return {
    api_version = 1, id = "OG_113", name = "Darkshire Councilman",
    text = "[x]After you summon a minion,\n gain +1 Attack.", set = "OG", type = "minion",
    class = "warlock", rarity = "common", cost = 3, attack = 1, health = 5,
    triggers = {{
        event = "minion_summoned", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.entity ~= self
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 1, 0) end,
    }},
}
