return {
    api_version = 1,
    id = "CS2_024",
    name = "Frostbolt",
    text = "Deal $3 damage to a character and <b>Freeze</b> it.",
    set = "LEGACY",
    type = "spell",
    class = "mage",
    cost = 2,
    keywords = { "freeze" },
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
    on_play = function(ctx, self, target)
        cardlib.effects.damage(ctx, target, 3)
        ctx:freeze(target)
    end,
}
