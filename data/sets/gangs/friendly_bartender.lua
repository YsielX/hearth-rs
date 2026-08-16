return {
    api_version = 1, id = "CFM_654", name = "Friendly Bartender",
    text = "At the end of your turn, restore #1 Health to your hero.",
    set = "GANGS", type = "minion", rarity = "common", cost = 2, attack = 2, health = 3,
    triggers = {{
        event = "turn_ended", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self) cardlib.effects.heal(ctx, ctx:player(ctx:controller(self)).hero, 1) end,
    }},
}
