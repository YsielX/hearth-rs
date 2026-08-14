local card = {
    api_version = 1, id = "CFM_672", name = "Madam Goya",
    text = "<b>Battlecry:</b> Choose a friendly\nminion. Summon any copies\nof it in your deck.",
    set = "GANGS", type = "minion", rarity = "legendary", cost = 6, attack = 4,
    health = 3, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:friendly_minions(self)) do
            if entity ~= self then result[#result + 1] = entity end
        end
        return result
    end,
}
function card.on_battlecry(ctx, self, target)
    if not target then return end
    local player = ctx:controller(self)
    local card_id = ctx:entity(target).card_id
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).card_id == card_id then ctx:recruit(player, entity) end
    end
end
return card
