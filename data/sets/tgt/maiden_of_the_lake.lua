return {
    api_version = 1, id = "AT_085", name = "Maiden of the Lake",
    text = "Your Hero Power costs (1).",
    set = "TGT", type = "minion", rarity = "common", cost = 4, attack = 2, health = 6,
    auras = {{
        cost_set = 1,
        targets = function(ctx, self)
            return { ctx:player(ctx:controller(self)).hero_power }
        end,
    }},
}
