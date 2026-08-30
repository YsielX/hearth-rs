return {
    api_version = 1,
    id = "SCH_231", rarity = "common",
    name = "Intrepid Initiate",
    text = "<b>Spellburst:</b> Gain +2 Attack.",
    set = "SCHOLOMANCE",
    type = "minion",
    cost = 1,
    attack = 1,
    health = 2,
    keywords = { "spellburst" },
    on_spellburst = function(ctx, self)
        cardlib.effects.buff(ctx, self, 2, 0)
    end,
}
