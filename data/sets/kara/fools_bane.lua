return {
    api_version = 1,
    id = "KAR_028",
    name = "Fool's Bane",
    text = "Unlimited attacks each turn. Can't attack heroes.",
    set = "KARA",
    type = "weapon",
    class = "warrior",
    rarity = "common",
    cost = 5,
    attack = 3,
    health = 4,
    auras = {
        {
            active_zones = { "weapon" },
            keywords = { "fools_bane_unlimited_attacks" },
            targets = function(ctx, self)
                return { ctx:player(ctx:controller(self)).hero }
            end,
        },
        {
            active_zones = { "weapon" },
            keywords = { "cannot_be_attacked_by_fools_bane" },
            targets = function(ctx, self)
                local enemy = ctx:opponent(ctx:controller(self))
                return { ctx:player(enemy).hero }
            end,
        },
    },
}
