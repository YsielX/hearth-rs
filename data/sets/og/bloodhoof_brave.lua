return {
    api_version = 1, id = "OG_218", name = "Bloodhoof Brave",
    text = "<b>Taunt</b>\nHas +3 Attack while damaged.", set = "OG", type = "minion",
    class = "warrior", rarity = "common", cost = 4, attack = 2, health = 6,
    keywords = { "taunt" }, auras = {{
        attack = function(ctx, self) return ctx:entity(self).damage > 0 and 3 or 0 end,
        targets = function(ctx, self) return { self } end,
    }},
}
