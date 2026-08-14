return {
    api_version = 1,
    id = "CFM_609",
    name = "Fel Orc Soulfiend",
    text = "At the start of your turn, deal 2 damage to this minion.",
    set = "GANGS",
    type = "minion",
    rarity = "epic",
    cost = 3,
    attack = 3,
    health = 7,
    triggers = {{
        event = "turn_started", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self) ctx:damage(self, 2) end,
    }},
}
