local card = {
    api_version = 1,
    id = "KAR_025",
    name = "Kara Kazham!",
    text = "Summon a 1/1 Candle, 2/2 Broom, and 3/3 Teapot.",
    set = "KARA",
    type = "spell",
    class = "warlock",
    rarity = "common",
    cost = 5,
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    ctx:summon(player, "KAR_025a")
    ctx:summon(player, "KAR_025b")
    ctx:summon(player, "KAR_025c")
end

card.tokens = {
    { id = "KAR_025a", name = "Candle", text = "", set = "KARA", type = "minion", class = "warlock", cost = 1, attack = 1, health = 1 },
    { id = "KAR_025b", name = "Broom", text = "", set = "KARA", type = "minion", class = "warlock", cost = 2, attack = 2, health = 2 },
    { id = "KAR_025c", name = "Teapot", text = "", set = "KARA", type = "minion", class = "warlock", cost = 3, attack = 3, health = 3 },
}

return card
