local card = {
    api_version = 1, id = "AT_025", name = "Dark Bargain",
    text = "Destroy 2 random enemy minions. Discard 2 random cards.", set = "TGT", type = "spell",
    class = "warlock", rarity = "epic", cost = 4, spell_school = "shadow",
}

local function enemy_minions(ctx, self)
    local result = {}
    for _, entity in ipairs(ctx:board(ctx:opponent(ctx:controller(self)))) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

local function start_discard(ctx, self)
    local hand = ctx:hand(ctx:controller(self))
    if #hand > 0 then ctx:random_entity(hand, "choose_first_discard") end
end

function card.on_play(ctx, self)
    local enemies = enemy_minions(ctx, self)
    if #enemies == 0 then start_discard(ctx, self)
    else ctx:random_entity(enemies, "choose_first_destroy") end
end

function card.choose_first_destroy(ctx, self, target)
    ctx:set_data(self, "first_destroy", target)
    local candidates = {}
    for _, minion in ipairs(enemy_minions(ctx, self)) do
        if minion ~= target then candidates[#candidates + 1] = minion end
    end
    if #candidates == 0 then cardlib.effects.destroy(ctx, target); start_discard(ctx, self)
    else ctx:random_entity(candidates, "choose_second_destroy") end
end

function card.choose_second_destroy(ctx, self, target)
    cardlib.effects.destroy_all(ctx, { ctx:get_data(self, "first_destroy"), target })
    start_discard(ctx, self)
end

function card.choose_first_discard(ctx, self, target)
    ctx:set_data(self, "first_discard", target)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if entity ~= target then candidates[#candidates + 1] = entity end
    end
    if #candidates == 0 then ctx:discard(ctx:controller(self), target)
    else ctx:random_entity(candidates, "choose_second_discard") end
end

function card.choose_second_discard(ctx, self, target)
    local player = ctx:controller(self)
    ctx:discard(player, ctx:get_data(self, "first_discard"))
    ctx:discard(player, target)
end

return card
