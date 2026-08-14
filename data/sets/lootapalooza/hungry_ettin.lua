local card = {
    api_version = 1,
    id = "LOOT_383",
    name = "Hungry Ettin",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> Summon a random 2-Cost minion for your opponent.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "rare",
    cost = 6,
    attack = 4,
    health = 10,
    keywords = { "taunt", "battlecry" },
}

function card.on_battlecry(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 2 then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then ctx:random_value(pool, "hungry_ettin_minion_chosen") end
end

function card.hungry_ettin_minion_chosen(ctx, self, card_id)
    ctx:summon(ctx:opponent(ctx:controller(self)), card_id)
end

return card
