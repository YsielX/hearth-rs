local card = {
    api_version = 1,
    id = "KAR_114",
    name = "Barnes",
    text = "<b>Battlecry:</b> Summon a 1/1 copy of a random minion in your deck.",
    set = "KARA",
    type = "minion",
    rarity = "legendary",
    cost = 5,
    attack = 3,
    health = 4,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    if #ctx:board(player) >= 7 then return end
    local candidates = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "minion" then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "summon_one_one_copy") end
end

function card.summon_one_one_copy(ctx, self, entity)
    ctx:summon_fresh_copy_with_stats(entity, nil, 1, 1, {})
end

return card
