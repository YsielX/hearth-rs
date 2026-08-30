return {
    api_version = 1, id = "CFM_851", name = "Daring Reporter",
    text = "Whenever your opponent draws a card, gain +1/+1.", set = "GANGS",
    type = "minion", rarity = "common", cost = 4, attack = 3, health = 3,
    triggers = {{
        event = "card_drawn", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:opponent(ctx:controller(self))
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 1, 1) end,
    }},
}
