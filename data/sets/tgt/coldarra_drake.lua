return {
    api_version = 1, id = "AT_008", name = "Coldarra Drake",
    text = "You can use your Hero Power any number of times.",
    set = "TGT", type = "minion", class = "mage", rarity = "epic",
    cost = 6, attack = 6, health = 7, tags = { "dragon" },
    auras = {
        {
            keywords = { "hero_power_unlimited" },
            targets = function(ctx, self)
                return { ctx:player(ctx:controller(self)).hero_power }
            end,
        },
    },
}
