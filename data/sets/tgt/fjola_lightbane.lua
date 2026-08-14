return {
    api_version = 1, id = "AT_129", name = "Fjola Lightbane",
    text = "Whenever <b>you</b> target this minion with a spell, gain <b><b>Divine Shield</b>.</b>",
    set = "TGT", type = "minion", rarity = "legendary", cost = 3, attack = 3, health = 4,
    tags = { "undead" }, triggers = {{
        event = "spell_targeted", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.target == self
        end,
        effect = function(ctx, self) ctx:grant_keyword(self, "divine_shield") end,
    }},
}
