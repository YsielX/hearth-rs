return {
    api_version = 1,
    id = "GVG_122",
    name = "Wee Spellstopper",
    text = "Adjacent minions\nhave <b>Elusive</b>.",
    set = "GVG",
    type = "minion",
    class = "mage",
    rarity = "epic",
    cost = 4,
    attack = 2,
    health = 5,
    auras = {
        {
            keywords = { "elusive" },
            targets = function(ctx, self) return ctx:adjacent_minions(self) end,
        },
    },
}
