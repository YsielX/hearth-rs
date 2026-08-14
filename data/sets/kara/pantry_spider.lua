return {
    api_version = 1,
    id = "KAR_030a",
    name = "Pantry Spider",
    text = "<b>Battlecry:</b> Summon a\n1/3 Spider.",
    set = "KARA",
    type = "minion",
    rarity = "common",
    cost = 3,
    attack = 1,
    health = 3,
    tags = { "beast" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:summon(ctx:controller(self), "KAR_030a")
    end,
}
