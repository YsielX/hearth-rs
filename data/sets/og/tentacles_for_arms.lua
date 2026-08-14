local card = {
    api_version = 1, id = "OG_033", name = "Tentacles for Arms",
    text = "<b>Deathrattle:</b> Return this to your hand.", set = "OG", type = "weapon",
    class = "warrior", rarity = "epic", cost = 5, attack = 2, health = 2,
    keywords = { "deathrattle" },
}
function card.on_deathrattle(ctx, self) ctx:move(self, "hand") end
return card
