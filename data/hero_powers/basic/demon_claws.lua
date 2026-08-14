return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_10bp",
    name = "Demon Claws",
    text = "[x]<b>Hero Power</b>\n+$a1 Attack this turn.",
    set = "LEGACY",
    class = "demon_hunter",
    cost = 1,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:buff_until_end_of_turn(ctx:player(player).hero, 1, 0)
    end,
}
