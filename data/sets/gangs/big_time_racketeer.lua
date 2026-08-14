return {
    api_version = 1, id = "CFM_648", name = "Big-Time Racketeer",
    text = "<b>Battlecry:</b> Summon a 6/6 Ogre.", set = "GANGS", type = "minion", rarity = "common",
    cost = 6, attack = 1, health = 1, keywords = { "battlecry" },
    on_battlecry = function(ctx, self) ctx:summon(ctx:controller(self), "CFM_648t") end,
    tokens = {{
        id = "CFM_648t", name = "\"Little Friend\"", text = "", set = "GANGS", type = "minion",
        cost = 6, attack = 6, health = 6,
    }},
}
