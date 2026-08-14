local card = {
    api_version = 1,
    id = "KAR_710",
    name = "Arcanosmith",
    text = "<b>Battlecry:</b> Summon a 0/5 minion with <b>Taunt</b>.",
    set = "KARA",
    type = "minion",
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:summon(ctx:controller(self), "KAR_710m")
end

card.tokens = {
    {
        id = "KAR_710m",
        name = "Animated Shield",
        text = "<b>Taunt</b>",
        set = "KARA",
        type = "minion",
        collectible = false,
        cost = 2,
        attack = 0,
        health = 5,
        keywords = { "taunt" },
    },
}

return card
