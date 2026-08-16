return {
    api_version = 1,
    id = "GVG_057",
    name = "Seal of Light",
    text = "Restore #4 Health to your hero and gain +2 Attack this turn.",
    set = "GVG",
    type = "spell",
    class = "paladin",
    rarity = "common",
    cost = 2,
    spell_school = "holy",
    on_play = function(ctx, self)
        local hero = ctx:player(ctx:controller(self)).hero
        cardlib.effects.heal(ctx, hero, 4)
        ctx:buff_until_end_of_turn(hero, 2, 0)
    end,
}
