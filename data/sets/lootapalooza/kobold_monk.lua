return {
    api_version = 1,
    id = "LOOT_382",
    name = "Kobold Monk",
    text = "Your hero is <b>Elusive</b>.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "rare",
    cost = 4,
    attack = 3,
    health = 6,
    auras = {
        {
            keywords = { "elusive" },
            targets = function(ctx, self)
                return { ctx:player(ctx:controller(self)).hero }
            end,
        },
    },
}
