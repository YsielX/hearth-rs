return {
    api_version = 1,
    id = "CATA_612", rarity = "common",
    name = "Frostbitten Imp",
    text = "<b>Battlecry:</b> <b>Freeze</b> this.",
    set = "CATACLYSM",
    type = "minion",
    cost = 2,
    attack = 5,
    health = 3,
    tags = { "demon" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:freeze(self)
    end,
}
