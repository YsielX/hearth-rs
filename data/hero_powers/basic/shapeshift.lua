return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_06bp",
    name = "Shapeshift",
    text = "<b>Hero Power</b>\n+$a1 Attack this turn.\n+$d1 Armor.",
    set = "LEGACY",
    class = "druid",
    cost = 2,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:buff_until_end_of_turn(ctx:player(player).hero, 1, 0)
        ctx:gain_armor(player, 1)
    end,
}
