local card = {
    api_version = 1, id = "LOOT_398", name = "Benevolent Djinn",
    text = "At the end of your turn, restore #3 Health to your hero.", set = "LOOTAPALOOZA",
    type = "minion", class = "paladin", rarity = "common", cost = 3, attack = 2, health = 4,
    tags = { "elemental" },
    triggers = {{ event = "turn_ended", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self) ctx:heal(ctx:player(ctx:controller(self)).hero, 3) end }},
}
return card
