return {
    api_version = 1,
    id = "GVG_073",
    name = "Cobra Shot",
    text = "Deal $3 damage to\na minion and the\nenemy hero.",
    set = "GVG",
    type = "spell",
    class = "hunter",
    spell_school = "nature",
    rarity = "common",
    cost = 4,
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target)
        local enemy = ctx:opponent(ctx:controller(self))
        cardlib.effects.damage(ctx, target, 3)
        cardlib.effects.damage(ctx, ctx:player(enemy).hero, 3)
    end,
}
