local card = {
    api_version = 1, id = "AT_023", name = "Void Crusher",
    text = "<b>Inspire:</b> Destroy a random minion for each player.", set = "TGT", type = "minion",
    class = "warlock", rarity = "rare", cost = 6, attack = 5, health = 4,
    tags = { "demon" }, keywords = { "inspire" },
}

local function enemy_minions(ctx, self)
    local result = {}
    for _, entity in ipairs(ctx:board(ctx:opponent(ctx:controller(self)))) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

function card.on_inspire(ctx, self)
    local friendly = ctx:friendly_minions(self)
    if #friendly > 0 then ctx:random_entity(friendly, "choose_friendly_victim") end
end

function card.choose_friendly_victim(ctx, self, target)
    ctx:set_data(self, "friendly_victim", target)
    local enemy = enemy_minions(ctx, self)
    if #enemy == 0 then cardlib.effects.destroy(ctx, target)
    else ctx:random_entity(enemy, "choose_enemy_victim") end
end

function card.choose_enemy_victim(ctx, self, target)
    cardlib.effects.destroy_all(ctx, { ctx:get_data(self, "friendly_victim"), target })
end

return card
