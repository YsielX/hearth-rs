local card = {
    api_version = 1,
    id = "FP1_009",
    name = "Deathlord",
    text = "<b>Taunt. Deathrattle:</b> Your opponent puts a minion from their deck into the battlefield.",
    set = "NAXX",
    type = "minion",
    rarity = "rare",
    cost = 3,
    attack = 2,
    health = 8,
    tags = { "undead" },
    keywords = { "taunt", "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local opponent = ctx:opponent(ctx:controller(self))
    local candidates = {}
    for _, entity in ipairs(ctx:deck(opponent)) do
        if ctx:entity(entity).type == "minion" then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "recruit_enemy_minion") end
end

function card.recruit_enemy_minion(ctx, self, entity)
    ctx:recruit(ctx:opponent(ctx:controller(self)), entity)
end

return card
