local card = {
    api_version = 1,
    id = "AT_035",
    name = "Beneath the Grounds",
    text = "Shuffle 3 Ambushes into your opponent's deck. When drawn, you summon a 4/4 Nerubian.",
    set = "TGT",
    type = "spell",
    class = "rogue",
    rarity = "epic",
    cost = 3,
    on_play = function(ctx, self)
        local opponent = ctx:opponent(ctx:controller(self))
        for _ = 1, 3 do ctx:shuffle_card_into_deck(opponent, "AT_035t") end
    end,
}

card.tokens = {
    {
        id = "AT_035t",
        name = "Nerubian Ambush!",
        text = "<b>Casts When Drawn</b>\nSummon a 4/4 Nerubian for your opponent.",
        set = "TGT",
        type = "spell",
        class = "rogue",
        cost = 3,
        keywords = { "casts_when_drawn" },
        on_play = function(ctx, self)
            ctx:summon(ctx:opponent(ctx:controller(self)), "FP1_007t")
        end,
    },
}

return card
