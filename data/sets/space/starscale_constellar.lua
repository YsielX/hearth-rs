return {
    api_version = 1,
    id = "GDB_860", rarity = "common",
    name = "Starscale Constellar",
    text = "<b><b>Spellburst</b>:</b> Double this minion's Attack.",
    set = "SPACE",
    type = "minion",
    cost = 5,
    attack = 4,
    health = 7,
    tags = { "dragon" },
    keywords = { "spellburst" },
    on_spellburst = function(ctx, self)
        cardlib.effects.modify(ctx, self, {
            stat = "attack",
            operation = "multiply",
            value = 2,
        })
    end,
}
