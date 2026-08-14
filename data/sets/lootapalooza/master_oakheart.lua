local card = {
    api_version = 1,
    id = "LOOT_521",
    name = "Master Oakheart",
    text = "<b>Battlecry:</b> <b>Recruit</b> a 1, 2, and 3-Attack minion.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "legendary",
    cost = 9,
    attack = 5,
    health = 5,
    keywords = { "battlecry", "recruit" },
}

local function candidates_with_attack(ctx, self, attack)
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        local minion = ctx:entity(entity)
        if minion.type == "minion" and minion.attack == attack then
            candidates[#candidates + 1] = entity
        end
    end
    return candidates
end

function card.on_battlecry(ctx, self)
    local candidates = candidates_with_attack(ctx, self, 1)
    if #candidates > 0 then
        ctx:random_value(candidates, "oakheart_recruit_1")
    else
        ctx:continue_with("oakheart_recruit_1")
    end
end

function card.oakheart_recruit_1(ctx, self, entity)
    if entity ~= nil then ctx:recruit(ctx:controller(self), entity) end
    ctx:continue_with("oakheart_choose_2")
end

function card.oakheart_choose_2(ctx, self)
    local candidates = candidates_with_attack(ctx, self, 2)
    if #candidates > 0 then
        ctx:random_value(candidates, "oakheart_recruit_2")
    else
        ctx:continue_with("oakheart_recruit_2")
    end
end

function card.oakheart_recruit_2(ctx, self, entity)
    if entity ~= nil then ctx:recruit(ctx:controller(self), entity) end
    ctx:continue_with("oakheart_choose_3")
end

function card.oakheart_choose_3(ctx, self)
    local candidates = candidates_with_attack(ctx, self, 3)
    if #candidates > 0 then
        ctx:random_value(candidates, "oakheart_recruit_3")
    else
        ctx:continue_with("oakheart_recruit_3")
    end
end

function card.oakheart_recruit_3(ctx, self, entity)
    if entity ~= nil then ctx:recruit(ctx:controller(self), entity) end
end

return card
