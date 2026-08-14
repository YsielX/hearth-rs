local card = {
    api_version = 1,
    id = "FP1_014",
    name = "Stalagg",
    text = "<b>Deathrattle:</b> If Feugen also died this game, summon Thaddius.",
    set = "NAXX",
    type = "minion",
    rarity = "legendary",
    cost = 5,
    attack = 7,
    health = 4,
    tags = { "undead" },
    keywords = { "deathrattle" },
    tokens = {
        {
            id = "FP1_014t",
            name = "Thaddius",
            text = "",
            set = "NAXX",
            type = "minion",
            cost = 10,
            attack = 11,
            health = 11,
            tags = { "undead" },
        },
    },
}

function card.on_deathrattle(ctx, self)
    for player = 0, 1 do
        for _, card_id in ipairs(ctx:minions_died(player)) do
            if card_id == "FP1_015" then
                ctx:summon(ctx:controller(self), "FP1_014t")
                return
            end
        end
    end
end

return card
