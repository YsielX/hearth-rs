local card = {
    api_version = 1,
    id = "LOOT_516",
    name = "Zola the Gorgon",
    text = "<b>Battlecry:</b> Choose a friendly minion. Add a Golden copy of it to your hand.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "legendary",
    cost = 3,
    attack = 2,
    health = 2,
    tags = { "naga" },
    keywords = { "battlecry" },
    target_mode = "required",
}

function card.targets(ctx, self)
    local result = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(minion).keywords or {}) do
            if keyword == "dormant" then dormant = true; break end
        end
        if not dormant then result[#result + 1] = minion end
    end
    return result
end

function card.on_battlecry(ctx, self, target)
    -- Golden is cosmetic; Battlefield-to-Hand copy semantics use the base card.
    cardlib.effects.give_base_copy(ctx, ctx:controller(self), target)
end

return card
