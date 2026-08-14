local card = {
    api_version = 1,
    id = "AT_036",
    name = "Anub'arak",
    text = "<b>Deathrattle:</b> Summon a 4/4 Nerubian with \"<b>Deathrattle:</b> \nSummon Anub'arak.\"",
    set = "TGT",
    type = "minion",
    class = "rogue",
    rarity = "legendary",
    cost = 8,
    attack = 8,
    health = 4,
    tags = { "undead" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self) ctx:summon(ctx:controller(self), "AT_036t") end,
}

card.tokens = {
    {
        id = "AT_036t",
        name = "Nerubian",
        text = "<b>Deathrattle:</b> Summon Anub'arak.",
        set = "TGT",
        type = "minion",
        class = "rogue",
        cost = 4,
        attack = 4,
        health = 4,
        tags = { "undead" },
        keywords = { "deathrattle" },
        on_deathrattle = function(ctx, self) ctx:summon(ctx:controller(self), "AT_036") end,
    },
}

return card
