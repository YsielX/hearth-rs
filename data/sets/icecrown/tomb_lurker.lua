local card = {
    api_version = 1,
    id = "ICC_098",
    name = "Tomb Lurker",
    text = "[x]<b>Battlecry:</b> Add a random\n<b>Deathrattle</b> minion that died\nthis game to your hand.",
    set = "ICECROWN",
    type = "minion",
    rarity = "epic",
    cost = 5,
    attack = 5,
    health = 3,
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, record in ipairs(ctx:minion_death_records(ctx:controller(self))) do
        local had_deathrattle = false
        for _, keyword in ipairs(record.keywords or {}) do
            if keyword == "deathrattle" then had_deathrattle = true; break end
        end
        if had_deathrattle then candidates[#candidates + 1] = record.card_id end
    end
    if #candidates > 0 then ctx:random_value(candidates, "receive_dead_deathrattle_minion") end
end

function card.receive_dead_deathrattle_minion(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
