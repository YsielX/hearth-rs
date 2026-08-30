local card = {
    api_version = 1,
    id = "TTN_454",
    name = "Down with the Ship",
    text = "Deal $3 damage. Shuffle two random Plagues into your opponent's deck.",
    set = "TITANS",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    spell_school = "shadow",
    cost = 2,
    rune_cost = { unholy = 1 },
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.damage(ctx, target, 3)
    ctx:set_data(self, "plagues_left", 2)
    ctx:continue_with("choose_random_plague")
end

function card.choose_random_plague(ctx, self)
    if ctx:get_data(self, "plagues_left") > 0 then
        ctx:random_value(cardlib.plagues.ids, "shuffle_random_plague")
    end
end

function card.shuffle_random_plague(ctx, self, card_id)
    local player = ctx:controller(self)
    cardlib.plagues.shuffle(ctx, player, ctx:opponent(player), card_id)
    ctx:set_data(self, "plagues_left", ctx:get_data(self, "plagues_left") - 1)
    ctx:continue_with("choose_random_plague")
end

return card
