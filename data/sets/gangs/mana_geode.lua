return {
    api_version = 1, id = "CFM_606", name = "Mana Geode",
    text = "<b>Overheal:</b> Summon a\n2/2 Crystal.",
    set = "GANGS", type = "minion", class = "priest", rarity = "epic",
    cost = 2, attack = 2, health = 3, tags = { "elemental" }, keywords = { "overheal" },
    on_overheal = function(ctx, self) ctx:summon(ctx:controller(self), "CFM_606t") end,
    tokens = {{
        id = "CFM_606t", name = "Crystal", text = "", set = "GANGS", type = "minion", class = "priest",
        cost = 2, attack = 2, health = 2, tags = { "elemental" },
    }},
}
