local card = {
    api_version = 1,
    id = "FP1_015",
    name = "Feugen",
    text = "<b>Deathrattle:</b> If Stalagg also died this game, summon Thaddius.",
    set = "NAXX",
    type = "minion",
    rarity = "legendary",
    cost = 5,
    attack = 4,
    health = 7,
    tags = { "undead" },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    for player = 0, 1 do
        for _, card_id in ipairs(ctx:minions_died(player)) do
            if card_id == "FP1_014" then
                ctx:summon(ctx:controller(self), "FP1_014t")
                return
            end
        end
    end
end

return card
