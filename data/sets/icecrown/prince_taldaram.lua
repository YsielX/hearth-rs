local function condition_met(ctx, self)
    local player = ctx:controller(self)
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).cost == 3 then
            return false
        end
    end
    return true
end

local card = {
    api_version = 1,
    id = "ICC_852",
    name = "Prince Taldaram",
    text = "<b>Battlecry:</b> If your deck has no 3-Cost cards, transform into a 3/3 copy of a minion.",
    set = "ICECROWN",
    type = "minion",
    rarity = "legendary",
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "undead" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
}

function card.targets(ctx, self)
    if not condition_met(ctx, self) then return {} end
    local result = {}
    for _, entity in ipairs(ctx:minions()) do
        local snapshot = ctx:entity(entity)
        local dormant = false
        for _, keyword in ipairs(snapshot.keywords or {}) do
            if keyword == "dormant" then dormant = true; break end
        end
        if not dormant then result[#result + 1] = entity end
    end
    return result
end

function card.on_battlecry(ctx, self, target)
    if target and condition_met(ctx, self) then
        ctx:transform_into_copy(self, target, 3, 3)
    end
end

return card
