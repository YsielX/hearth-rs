return {
    api_version = 1,
    id = "RLK_042",
    name = "Horn of Winter",
    text = "Refresh 2 Mana Crystals.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    spell_school = "frost",
    cost = 0,
    rune_cost = { frost = 2 },
    on_play = function(ctx, self)
        ctx:refresh_mana_crystals(ctx:controller(self), 2)
    end,
}
