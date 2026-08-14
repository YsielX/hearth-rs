return {
    api_version = 1, id = "AT_027", name = "Wilfred Fizzlebang",
    text = "Cards you draw from your Hero Power cost (0).", set = "TGT", type = "minion",
    class = "warlock", rarity = "legendary", cost = 6, attack = 4, health = 4,
    triggers = {{
        event = "card_drawn", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            local player = ctx:controller(self)
            return event.player == player and event.source == ctx:player(player).hero_power
        end,
        effect = function(ctx, self, event)
            ctx:modify(event.entity, { stat = "cost", operation = "set", value = 0 })
        end,
    }},
}
