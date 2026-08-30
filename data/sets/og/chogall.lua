local card = {
    api_version = 1,
    id = "OG_121",
    name = "Cho'gall",
    text = "[x]<b>Battlecry:</b> Return all cards\nyou discarded this game to\nyour hand. They cost Health\ninstead of Mana.",
    set = "OG",
    type = "minion",
    class = "warlock",
    rarity = "legendary",
    cost = 8,
    attack = 8,
    health = 8,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local seen = {}
    for _, entity in ipairs(ctx:discarded_cards(ctx:controller(self))) do
        if not seen[entity] and ctx:entity(entity).zone == "graveyard" then
            seen[entity] = true
            ctx:move(entity, "hand")
            cardlib.effects.grant_keyword(ctx, entity, "costs_health_instead_of_mana")
        end
    end
end

return card
