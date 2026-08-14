local card = {
    api_version = 1, id = "OG_031", name = "Hammer of Twilight",
    text = "<b>Deathrattle:</b> Summon a 4/2 Elemental.", set = "OG", type = "weapon",
    class = "shaman", rarity = "epic", cost = 5, attack = 4, health = 2,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self) ctx:summon(ctx:controller(self), "OG_031a") end,
}
card.tokens = {{ id = "OG_031a", name = "Twilight Elemental", text = "", set = "OG",
    type = "minion", class = "shaman", cost = 3, attack = 4, health = 2,
    tags = { "elemental" } }}
return card
