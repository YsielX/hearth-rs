local horsemen = { "ICC_829t2", "ICC_829t3", "ICC_829t4", "ICC_829t5" }

local function present_horsemen(ctx, player)
    local present = {}
    for _, entity in ipairs(ctx:board(player)) do
        present[ctx:entity(entity).card_id] = true
    end
    return present
end

local power = {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_829p",
    name = "The Four Horsemen",
    text = "Summon a 2/2 Horseman.\nIf you have all 4, destroy\nthe enemy hero.",
    set = "ICECROWN",
    class = "neutral",
    cost = 2,
}

function power.on_play(ctx, self)
    local player = ctx:controller(self)
    local present = present_horsemen(ctx, player)
    local candidates = {}
    for _, card_id in ipairs(horsemen) do
        if not present[card_id] then candidates[#candidates + 1] = card_id end
    end
    if #candidates == 0 then
        power.check_victory(ctx, self)
    else
        ctx:random_value(candidates, "summon_horseman")
    end
end

function power.summon_horseman(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
    ctx:continue_with("check_victory")
end

function power.check_victory(ctx, self)
    local player = ctx:controller(self)
    local present = present_horsemen(ctx, player)
    for _, card_id in ipairs(horsemen) do
        if not present[card_id] then return end
    end
    local enemy = ctx:opponent(player)
    ctx:damage(ctx:player(enemy).hero, 9999)
end

power.tokens = {
    { id = "ICC_829t2", name = "Deathlord Nazgrim", text = "", set = "ICECROWN", type = "minion", class = "paladin", cost = 2, attack = 2, health = 2, tags = { "undead" } },
    { id = "ICC_829t3", name = "Thoras Trollbane", text = "", set = "ICECROWN", type = "minion", class = "paladin", cost = 2, attack = 2, health = 2, tags = { "undead" } },
    { id = "ICC_829t4", name = "Inquisitor Whitemane", text = "", set = "ICECROWN", type = "minion", class = "paladin", cost = 2, attack = 2, health = 2, tags = { "undead" } },
    { id = "ICC_829t5", name = "Darion Mograine", text = "", set = "ICECROWN", type = "minion", class = "paladin", cost = 2, attack = 2, health = 2, tags = { "undead" } },
}

return power
