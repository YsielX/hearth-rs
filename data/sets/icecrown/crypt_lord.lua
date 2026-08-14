return {
    api_version = 1, id = "ICC_808", name = "Crypt Lord",
    text = "[x]<b>Taunt</b>\nAfter you summon a minion,\n gain +1 Health.",
    set = "ICECROWN", type = "minion", class = "druid", rarity = "common",
    cost = 3, attack = 1, health = 6, tags = { "undead" }, keywords = { "taunt" },
    triggers = {{
        event = "minion_summoned", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.entity ~= self
        end,
        effect = function(ctx, self) ctx:buff(self, 0, 1) end,
    }},
}
