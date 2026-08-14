return {
    api_version = 1, id = "AT_080", name = "Garrison Commander",
    text = "You can use your Hero Power twice a turn.",
    set = "TGT", type = "minion", rarity = "epic", cost = 2, attack = 2, health = 3,
    auras = {{
        keywords = { "hero_power_twice_per_turn" },
        targets = function(ctx, self)
            return { ctx:player(ctx:controller(self)).hero_power }
        end,
    }},
}
