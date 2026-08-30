return {
    api_version = 1,
    id = "EX1_169", spell_school = "nature", rarity = "free",
    name = "Innervate",
    text = "Gain 1 Mana Crystal this turn only.",
    set = "LEGACY",
    type = "spell",
    class = "druid",
    cost = 0,
    on_play = function(ctx, self)
        ctx:gain_temporary_mana(ctx:controller(self), 1)
    end,
}
