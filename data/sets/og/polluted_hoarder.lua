return {
    api_version = 1, id = "OG_323", name = "Polluted Hoarder",
    text = "<b>Deathrattle:</b> Draw a card.", set = "OG", type = "minion", rarity = "common",
    cost = 4, attack = 4, health = 2, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self) ctx:draw(ctx:controller(self), 1) end,
}
