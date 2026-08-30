local card = {
    api_version = 1,
    id = "TTN_736",
    name = "Staff of the Primus",
    text = "After your hero attacks, shuffle a random\nPlague into your opponent's deck.",
    set = "TITANS",
    type = "weapon",
    class = "death_knight",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 3,
    rune_cost = { unholy = 2 },
}

card.triggers = {{
    event = "attack",
    timing = "after",
    active_zones = { "weapon", "graveyard" },
    condition = function(ctx, self, event)
        return event.attacker == ctx:player(ctx:controller(self)).hero
    end,
    effect = function(ctx)
        ctx:random_value(cardlib.plagues.ids, "shuffle_random_plague")
    end,
}}

function card.shuffle_random_plague(ctx, self, card_id)
    local player = ctx:controller(self)
    cardlib.plagues.shuffle(ctx, player, ctx:opponent(player), card_id)
end

return card
