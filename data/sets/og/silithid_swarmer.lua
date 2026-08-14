return {
    api_version = 1,
    id = "OG_034",
    name = "Silithid Swarmer",
    text = "Can only attack if your hero attacked this turn.",
    set = "OG",
    type = "minion",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 5,
    tags = { "beast" },
    rules = {
        can_attack = function(ctx, self, current)
            local hero = ctx:player(ctx:controller(self)).hero
            return current and ctx:entity(hero).attacks_this_turn > 0
        end,
    },
}
