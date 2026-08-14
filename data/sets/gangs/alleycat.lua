local card = {
    api_version = 1,
    id = "CFM_315",
    name = "Alleycat",
    text = "<b>Battlecry:</b> Summon a 1/1 Cat.",
    set = "GANGS",
    type = "minion",
    class = "hunter",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 1,
    tags = { "beast" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self) ctx:summon(ctx:controller(self), "CFM_315t") end,
}

card.tokens = {{
    id = "CFM_315t", name = "Tabbycat", text = "", set = "GANGS",
    type = "minion", class = "hunter", cost = 1, attack = 1, health = 1,
    tags = { "beast" },
}}

return card
