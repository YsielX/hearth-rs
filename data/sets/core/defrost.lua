return {
    api_version = 1,
    id = "RLK_101",
    name = "Defrost",
    text = "Draw a card.\nSpend 2 <b>Corpses</b> to draw another.",
    set = "CORE_HIDDEN",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    spell_school = "frost",
    cost = 2,
    rune_cost = { frost = 1 },
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:draw(player, 1)
        ctx:spend_corpses_and_continue(player, 2, "draw_again")
    end,
    draw_again = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
}
