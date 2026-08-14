local card = {
    api_version = 1, id = "OG_241", name = "Possessed Villager",
    text = "<b>Deathrattle:</b> Summon a 1/1 Shadowbeast.", set = "OG", type = "minion",
    class = "warlock", rarity = "common", cost = 1, attack = 1, health = 1,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self) ctx:summon(ctx:controller(self), "OG_241a") end,
}
card.tokens = {{ id = "OG_241a", name = "Shadowbeast", text = "", set = "OG",
    type = "minion", class = "warlock", cost = 1, attack = 1, health = 1 }}
return card
