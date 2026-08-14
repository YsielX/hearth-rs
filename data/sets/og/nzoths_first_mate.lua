local card = {
    api_version = 1, id = "OG_312", name = "N'Zoth's First Mate",
    text = "<b>Battlecry:</b> Equip a 1/3 Rusty Hook.", set = "OG", type = "minion",
    class = "warrior", rarity = "common", cost = 1, attack = 1, health = 1,
    tags = { "pirate" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self) ctx:equip_weapon(ctx:controller(self), "OG_058") end,
}
card.tokens = {{ id = "OG_058", name = "Rusty Hook", text = "", set = "OG", type = "weapon",
    class = "warrior", cost = 1, attack = 1, health = 3 }}
return card
