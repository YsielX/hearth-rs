return {
    api_version = 1,
    id = "CS2_031",
    name = "Ice Lance",
    text = "<b>Freeze</b> a character. If it was already <b>Frozen</b>, deal $4 damage instead.",
    set = "EXPERT1",
    type = "spell",
    class = "mage",
    rarity = "common",
    spell_school = "frost",
    cost = 1,
    target_mode = "required",
    targets = function(ctx)
        return ctx:characters()
    end,
    on_play = function(ctx, self, target)
        if ctx:entity(target).frozen then
            cardlib.effects.damage(ctx, target, 4)
        else
            ctx:freeze(target)
        end
    end,
}
