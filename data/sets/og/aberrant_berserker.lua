return {
    api_version = 1, id = "OG_150", name = "Aberrant Berserker",
    text = "Has +2 Attack while damaged.", set = "OG", type = "minion",
    rarity = "common", cost = 4, attack = 3, health = 5,
    auras = {{
        attack = function(ctx, self) return ctx:entity(self).damage > 0 and 2 or 0 end,
        targets = function(ctx, self) return { self } end,
    }},
}
