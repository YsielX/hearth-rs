return {
    api_version = 1,
    id = "EX1_238",
    name = "Lightning Bolt",
    text = "Deal $3 damage. <b>Overload:</b> (1)",
    set = "EXPERT1",
    type = "spell",
    spell_school = "nature",
    class = "shaman",
    cost = 1,
    keywords = { "overload" },
    keyword_params = { overload = 1 },
    target_mode = "required",
    targets = function(ctx, self)
        return ctx:characters()
    end,
    on_play = function(ctx, self, target)
        cardlib.effects.damage(ctx, target, 3)
    end,
}
