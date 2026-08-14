local card = {
    api_version = 1, id = "AT_044", name = "Mulch",
    text = "Destroy a minion.\nAdd a random minion to your opponent's hand.",
    set = "TGT", type = "spell", class = "druid", rarity = "epic",
    spell_school = "nature", cost = 3, target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
}

function card.on_play(ctx, self, target)
    ctx:destroy(target)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(id).type == "minion" then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:random_value(pool, "give_random_minion") end
end

function card.give_random_minion(ctx, self, card_id)
    ctx:give_card(ctx:opponent(ctx:controller(self)), card_id)
end

return card
