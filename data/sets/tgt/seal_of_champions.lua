return {
    api_version = 1,
    id = "AT_074",
    name = "Seal of Champions",
    text = "Give a minion\n+3 Attack and <b>Divine Shield</b>.",
    set = "TGT",
    type = "spell",
    class = "paladin",
    rarity = "common",
    spell_school = "holy",
    cost = 3,
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target)
        ctx:buff(target, 3, 0)
        ctx:grant_keyword(target, "divine_shield")
    end,
}
