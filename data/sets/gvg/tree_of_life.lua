return {
    api_version = 1,
    id = "GVG_033",
    name = "Tree of Life",
    text = "Restore all characters to full Health.",
    set = "GVG",
    type = "spell",
    class = "druid",
    spell_school = "nature",
    rarity = "epic",
    cost = 9,
    on_play = function(ctx)
        cardlib.effects.heal_all(ctx, ctx:characters(), 1000000)
    end,
}
