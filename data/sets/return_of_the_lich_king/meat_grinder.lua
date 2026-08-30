local card = {
    api_version = 1,
    id = "RLK_120",
    name = "Meat Grinder",
    text = "<b>Battlecry:</b> Shred a random minion in your deck to gain 4 <b>Corpses.</b>",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "epic",
    cost = 3,
    attack = 3,
    health = 4,
    rune_cost = { unholy = 1 },
    tags = { "mech" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local minions = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" then minions[#minions + 1] = entity end
    end
    if #minions > 0 then ctx:random_entity(minions, "shred_minion") end
end

function card.shred_minion(ctx, self, minion)
    ctx:move(minion, "removed")
    ctx:gain_resource(ctx:controller(self), "corpses", 4)
end

return card
