return {
    api_version = 1, id = "UNG_079", name = "Frozen Crusher",
    text = "After this minion attacks, <b>Freeze</b> it.", set = "UNGORO",
    type = "minion", rarity = "rare", cost = 5, attack = 8, health = 8,
    tags = { "elemental" }, triggers = {{ event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.attacker == self end,
        effect = function(ctx, self) ctx:freeze(self) end }},
}
