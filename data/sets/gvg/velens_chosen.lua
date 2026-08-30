return {
    api_version = 1,
    id = "GVG_010",
    name = "Velen's Chosen",
    text = "Give a minion +2/+4 and <b>Spell Damage +1</b>.",
    set = "GVG",
    type = "spell",
    class = "priest",
    rarity = "common",
    cost = 3,
    spell_school = "holy",
    target_mode = "required",
    targets = function(ctx, self)
        return ctx:minions()
    end,
    on_play = function(ctx, self, target)
        cardlib.effects.buff(ctx, target, 2, 4)
        cardlib.effects.modify(ctx, target, {
            stat = "spell_damage",
            operation = "add",
            value = 1,
        })
    end,
}
