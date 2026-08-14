local card = {
    api_version = 1, id = "LOE_011", name = "Reno Jackson",
    text = "<b>Battlecry:</b> If your deck has no duplicates, fully heal your hero.",
    set = "LOE", type = "minion", rarity = "legendary", cost = 6, attack = 4, health = 6,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local seen = {}
    for _, entity in ipairs(ctx:deck(player)) do
        local card_id = ctx:entity(entity).card_id
        if seen[card_id] then return end
        seen[card_id] = true
    end
    local hero = ctx:player(player).hero
    ctx:heal(hero, ctx:entity(hero).damage)
end

return card
