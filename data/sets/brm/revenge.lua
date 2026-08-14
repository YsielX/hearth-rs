return {
    api_version = 1,
    id = "BRM_015",
    name = "Revenge",
    text = "Deal $1 damage to all minions. If you have 12 or less Health, deal $3 damage instead.",
    set = "BRM",
    type = "spell",
    class = "warrior",
    rarity = "rare",
    cost = 2,
    on_play = function(ctx, self)
        local hero = ctx:player(ctx:controller(self)).hero
        local amount = ctx:entity(hero).health <= 12 and 3 or 1
        ctx:damage_all(ctx:minions(), amount)
    end,
}
