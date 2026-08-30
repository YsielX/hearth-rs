return {
    api_version = 1, id = "GVG_103", name = "Micro Machine",
    text = "At the start of each turn, gain +1 Attack.", set = "GVG", type = "minion",
    rarity = "common", cost = 2, attack = 1, health = 2, tags = { "mech" },
    triggers = {{ event = "turn_started", active_zones = { "board" }, effect = function(ctx, self) cardlib.effects.buff(ctx, self, 1, 0) end }},
}
