local card = {
    api_version = 1,
    id = "TTN_450",
    name = "Distressed Kvaldir",
    text = "<b>Deathrattle:</b> Shuffle two random Plagues into your opponent's deck.",
    set = "TITANS",
    type = "minion",
    class = "death_knight",
    rarity = "epic",
    cost = 2,
    attack = 3,
    health = 2,
    rune_cost = { unholy = 2 },
    tags = { "undead" },
    keywords = { "deathrattle" },
}

function card.choose_random_plague(ctx, self)
    if ctx:get_data(self, "plagues_left") > 0 then
        ctx:random_value(cardlib.plagues.ids, "shuffle_random_plague")
    end
end

function card.on_deathrattle(ctx, self)
    ctx:set_data(self, "plagues_left", 2)
    ctx:continue_with("choose_random_plague")
end

function card.shuffle_random_plague(ctx, self, card_id)
    local player = ctx:controller(self)
    cardlib.plagues.shuffle(ctx, player, ctx:opponent(player), card_id)
    ctx:set_data(self, "plagues_left", ctx:get_data(self, "plagues_left") - 1)
    ctx:continue_with("choose_random_plague")
end

return card
