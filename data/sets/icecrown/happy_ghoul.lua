return {
    api_version = 1,
    id = "ICC_700",
    name = "Happy Ghoul",
    text = "Costs (0) if your hero was healed this turn.",
    set = "ICECROWN",
    type = "minion",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "undead" },
    auras = {
        {
            active_zones = { "hand" },
            cost_set = 0,
            targets = function(ctx, self)
                if ctx:hero_was_healed_this_turn(ctx:controller(self)) then return { self } end
                return {}
            end,
        },
    },
}
