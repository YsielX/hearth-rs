return {
    api_version = 1, id = "ICC_079", name = "Gnash",
    text = "Give your hero +3 Attack this turn. Gain 3 Armor.",
    set = "ICECROWN", type = "spell", class = "druid", rarity = "common", cost = 3,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:buff_until_end_of_turn(ctx:player(player).hero, 3, 0)
        ctx:gain_armor(player, 3)
    end,
}
