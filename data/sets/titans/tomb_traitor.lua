local card = {
    api_version = 1,
    id = "TTN_455",
    name = "Tomb Traitor",
    text = "<b>Battlecry:</b> Destroy a Plague in your opponent's deck to deal 3 damage to all enemy minions.",
    set = "TITANS",
    type = "minion",
    class = "death_knight",
    rarity = "rare",
    cost = 4,
    attack = 4,
    health = 3,
    rune_cost = { unholy = 1 },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:opponent(ctx:controller(self)))) do
        if cardlib.plagues.is_plague(ctx:entity(entity).card_id) then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "destroy_selected_plague") end
end

function card.destroy_selected_plague(ctx, self, plague)
    ctx:move(plague, "removed")
    local hits = {}
    for _, enemy in ipairs(ctx:enemy_minions(self)) do
        hits[#hits + 1] = { enemy, 3 }
    end
    cardlib.effects.damage_batch_ignoring_spell_damage(ctx, hits)
end

return card
