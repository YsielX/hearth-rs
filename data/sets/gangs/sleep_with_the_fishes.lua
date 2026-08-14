local card = {
    api_version = 1, id = "CFM_716", name = "Sleep with the Fishes",
    text = "Deal $3 damage to all damaged minions.", set = "GANGS", type = "spell",
    class = "warrior", rarity = "epic", cost = 2,
}
function card.on_play(ctx, self)
    local targets = {}
    for _, entity in ipairs(ctx:minions()) do
        local minion = ctx:entity(entity)
        if minion.health < minion.max_health then targets[#targets + 1] = entity end
    end
    if #targets > 0 then ctx:damage_all(targets, 3) end
end
return card
