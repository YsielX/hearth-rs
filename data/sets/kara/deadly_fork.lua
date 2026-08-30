local card = {
    api_version = 1,
    id = "KAR_094",
    name = "Deadly Fork",
    text = "<b>Deathrattle:</b> Add a 3/2 weapon to your hand.",
    set = "KARA",
    type = "minion",
    class = "rogue",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 2,
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    cardlib.effects.give_card(ctx, ctx:controller(self), "KAR_094a")
end

card.tokens = {
    {
        id = "KAR_094a",
        name = "Sharp Fork",
        text = "",
        set = "KARA",
        type = "weapon",
        class = "rogue",
        collectible = false,
        cost = 3,
        attack = 3,
        health = 2,
    },
}

return card
