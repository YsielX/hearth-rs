return {
    api_version = 1, id = "GVG_089", name = "Illuminator",
    text = "If you control a <b>Secret</b> at the end of your turn, restore #4 Health to your hero.",
    set = "GVG", type = "minion", rarity = "rare", cost = 3, attack = 2, health = 4,
    triggers = {{
        event = "turn_ended", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and #ctx:secrets(event.player) > 0
        end,
        effect = function(ctx, self)
            local player = ctx:controller(self)
            ctx:heal(ctx:player(player).hero, 4)
        end,
    }},
}
