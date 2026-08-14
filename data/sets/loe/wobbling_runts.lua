local card = {
    api_version = 1,
    id = "LOE_089",
    name = "Wobbling Runts",
    text = "<b>Deathrattle:</b> Summon three 2/2 Runts.",
    set = "LOE",
    type = "minion",
    rarity = "rare",
    cost = 6,
    attack = 2,
    health = 6,
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    ctx:summon(player, "LOE_089t")
    ctx:summon(player, "LOE_089t2")
    ctx:summon(player, "LOE_089t3")
end

card.tokens = {
    { id = "LOE_089t", name = "Rascally Runt", text = "", set = "LOE", type = "minion", cost = 2, attack = 2, health = 2 },
    { id = "LOE_089t2", name = "Wily Runt", text = "", set = "LOE", type = "minion", cost = 2, attack = 2, health = 2 },
    { id = "LOE_089t3", name = "Grumbly Runt", text = "", set = "LOE", type = "minion", cost = 2, attack = 2, health = 2 },
}

return card
