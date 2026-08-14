return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_05bp",
    name = "Steady Shot",
    text = "<b>Hero Power</b>\nDeal $2 damage to the enemy hero.",
    set = "LEGACY",
    class = "hunter",
    cost = 2,
    on_play = function(ctx, self)
        local enemy = ctx:opponent(ctx:controller(self))
        ctx:damage(ctx:player(enemy).hero, 2)
    end,
}
