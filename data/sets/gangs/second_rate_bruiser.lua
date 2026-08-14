return {
    api_version = 1, id = "CFM_652", name = "Second-Rate Bruiser",
    text = "[x]<b>Taunt</b>\nCosts (2) less if your\nopponent has at least\nthree minions.",
    set = "GANGS", type = "minion", rarity = "rare", cost = 5, attack = 4, health = 5,
    keywords = { "taunt" },
    auras = {{
        active_zones = { "hand" },
        cost = function(ctx, self)
            local player = ctx:controller(self)
            return #ctx:board(ctx:opponent(player)) >= 3 and -2 or 0
        end,
        targets = function(ctx, self) return { self } end,
    }},
}
