return {
    api_version = 1, id = "OG_138", name = "Nerubian Prophet",
    text = "At the start of your turn, reduce this card's\nCost by (1).",
    set = "OG", type = "minion", rarity = "common", cost = 6, attack = 4, health = 4,
    tags = { "undead" },
    triggers = {{
        event = "turn_started", timing = "after", active_zones = { "hand" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self)
            ctx:modify(self, { stat = "cost", operation = "add", value = -1 })
        end,
    }},
}
