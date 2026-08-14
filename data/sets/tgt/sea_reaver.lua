return {
    api_version = 1, id = "AT_130", name = "Sea Reaver",
    text = "When you draw this, deal 1 damage to your minions.", set = "TGT", type = "minion",
    class = "warrior", rarity = "epic", cost = 6, attack = 6, health = 7, tags = { "undead" },
    triggers = {{
        event = "card_drawn", timing = "after", active_zones = { "hand" },
        condition = function(ctx, self, event) return event.entity == self end,
        effect = function(ctx, self) ctx:damage_all(ctx:friendly_minions(self), 1) end,
    }},
}
