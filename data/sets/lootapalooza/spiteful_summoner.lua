local card = {
    api_version = 1,
    id = "LOOT_539",
    name = "Spiteful Summoner",
    text = "[x]<b>Battlecry:</b> Reveal a spell\nfrom your deck. Summon\n a random minion with\nthe same Cost.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "epic",
    cost = 6,
    attack = 4,
    health = 4,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local spells = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if ctx:entity(entity).type == "spell" then spells[#spells + 1] = entity end
    end
    if #spells > 0 then ctx:random_value(spells, "spiteful_spell_revealed") end
end

function card.spiteful_spell_revealed(ctx, self, spell)
    local cost = ctx:entity(spell).cost
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == cost then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then ctx:random_value(pool, "spiteful_minion_chosen") end
end

function card.spiteful_minion_chosen(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
