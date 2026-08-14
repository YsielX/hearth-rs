local card = {
    api_version = 1, id = "OG_216", name = "Infested Wolf",
    text = "<b>Deathrattle:</b> Summon two 1/1 Spiders.", set = "OG", type = "minion",
    class = "hunter", rarity = "rare", cost = 4, attack = 3, health = 3,
    tags = { "beast" }, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local player = ctx:controller(self)
        ctx:summon(player, "OG_216a"); ctx:summon(player, "OG_216a")
    end,
}
card.tokens = {{ id = "OG_216a", name = "Spider", text = "", set = "OG",
    type = "minion", class = "hunter", cost = 1, attack = 1, health = 1,
    tags = { "beast" } }}
return card
