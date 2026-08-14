local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local function highest_attack(ctx, player)
    local result, highest = {}, nil
    for _, entity in ipairs(ctx:board(player)) do
        local minion = ctx:entity(entity)
        if minion.type == "minion" and not is_dormant(ctx, entity) then
            if highest == nil or minion.attack > highest then
                highest, result = minion.attack, { entity }
            elseif minion.attack == highest then
                result[#result + 1] = entity
            end
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "AT_078",
    name = "Enter the Coliseum",
    text = "Destroy all minions except each player's highest Attack minion.",
    set = "TGT",
    type = "spell",
    class = "paladin",
    rarity = "epic",
    cost = 3,
}

local function select_for(ctx, self, player, key, continuation)
    local candidates = highest_attack(ctx, player)
    if #candidates == 0 then
        ctx:set_data(self, key, 0)
        ctx:continue_with(continuation)
    elseif #candidates == 1 then
        ctx:set_data(self, key, candidates[1])
        ctx:continue_with(continuation)
    else
        ctx:random_value(candidates, continuation == "select_enemy_survivor" and "save_friendly_survivor" or "save_enemy_survivor")
    end
end

function card.on_play(ctx, self)
    select_for(ctx, self, ctx:controller(self), "friendly_survivor", "select_enemy_survivor")
end

function card.save_friendly_survivor(ctx, self, entity)
    ctx:set_data(self, "friendly_survivor", entity)
    ctx:continue_with("select_enemy_survivor")
end

function card.select_enemy_survivor(ctx, self)
    select_for(ctx, self, ctx:opponent(ctx:controller(self)), "enemy_survivor", "destroy_losers")
end

function card.save_enemy_survivor(ctx, self, entity)
    ctx:set_data(self, "enemy_survivor", entity)
    ctx:continue_with("destroy_losers")
end

function card.destroy_losers(ctx, self)
    local friendly = ctx:get_data(self, "friendly_survivor")
    local enemy = ctx:get_data(self, "enemy_survivor")
    local losers = {}
    for _, minion in ipairs(ctx:minions()) do
        if minion ~= friendly and minion ~= enemy and not is_dormant(ctx, minion) then
            losers[#losers + 1] = minion
        end
    end
    ctx:destroy_all(losers)
end

return card
