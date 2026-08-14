local card = {
    api_version = 1,
    id = "AT_099",
    name = "Kodorider",
    text = "<b>Inspire:</b> Summon a 3/5 War Kodo.",
    set = "TGT",
    type = "minion",
    rarity = "epic",
    cost = 6,
    attack = 3,
    health = 5,
    keywords = { "inspire" },
    on_inspire = function(ctx, self) ctx:summon(ctx:controller(self), "AT_099t") end,
}

card.tokens = {
    {
        id = "AT_099t",
        name = "War Kodo",
        text = "",
        set = "TGT",
        type = "minion",
        cost = 5,
        attack = 3,
        health = 5,
        tags = { "beast" },
    },
}

return card
