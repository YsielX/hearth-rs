local card = {
    api_version = 1, id = "AT_046", name = "Tuskarr Totemic",
    text = "<b>Battlecry:</b> Summon a random basic Totem.", set = "TGT", type = "minion",
    class = "shaman", rarity = "common", cost = 2, attack = 3, health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:random_value({ "NEW1_009", "CS2_050", "CS2_051", "CS2_052" }, "summon_basic_totem")
end

function card.summon_basic_totem(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
