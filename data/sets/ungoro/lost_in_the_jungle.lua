return {
    api_version = 1, id = "UNG_960", name = "Lost in the Jungle",
    text = "Summon two {0} Silver Hand Recruits.", set = "UNGORO", type = "spell", class = "paladin",
    rarity = "common", cost = 1,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:summon(player, "CS2_101t")
        ctx:summon(player, "CS2_101t")
    end,
}
