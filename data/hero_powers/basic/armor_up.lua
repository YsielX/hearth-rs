return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_01bp", rarity = "free",
    name = "Armor Up!",
    text = "<b>Hero Power</b>\nGain $d2 Armor.",
    set = "LEGACY",
    class = "warrior",
    cost = 2,
    on_play = function(ctx, self)
        ctx:gain_armor(ctx:controller(self), 2)
    end,
}
