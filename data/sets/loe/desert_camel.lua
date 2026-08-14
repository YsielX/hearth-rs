local card = {
    api_version = 1, id = "LOE_020", name = "Desert Camel",
    text = "<b>Battlecry:</b> Put a 1-Cost minion from each deck into the battlefield.",
    set = "LOE", type = "minion", class = "hunter", rarity = "common",
    cost = 3, attack = 2, health = 4, tags = { "beast" }, keywords = { "battlecry" },
}

local function one_cost_minions(ctx, player)
    local result = {}
    for _, entity in ipairs(ctx:deck(player)) do
        local snapshot = ctx:entity(entity)
        if snapshot.type == "minion" and snapshot.cost == 1 then result[#result + 1] = entity end
    end
    return result
end

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    if #ctx:board(player) < 7 then
        local own = one_cost_minions(ctx, player)
        if #own > 0 then ctx:random_value(own, "recruit_friendly_minion") end
    end
    ctx:continue_with("begin_enemy_recruit")
end

function card.recruit_friendly_minion(ctx, self, entity)
    ctx:recruit(ctx:controller(self), entity)
end

function card.begin_enemy_recruit(ctx, self)
    local enemy = ctx:opponent(ctx:controller(self))
    if #ctx:board(enemy) >= 7 then return end
    local candidates = one_cost_minions(ctx, enemy)
    if #candidates > 0 then ctx:random_value(candidates, "recruit_enemy_minion") end
end

function card.recruit_enemy_minion(ctx, self, entity)
    ctx:recruit(ctx:opponent(ctx:controller(self)), entity)
end

return card
