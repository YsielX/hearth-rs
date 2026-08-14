local function characters(ctx)
    local result = {}
    for _, entity in ipairs(ctx:characters()) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(entity).keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant then result[#result + 1] = entity end
    end
    return result
end

local card = {
    api_version = 1, id = "OG_116", name = "Spreading Madness",
    text = "Deal $13 damage randomly split among ALL characters.", set = "OG", type = "spell",
    class = "warlock", rarity = "rare", cost = 3, spell_school = "shadow",
}
function card.on_play(ctx, self)
    ctx:set_data(self, "madness_left", 13)
    ctx:continue_with("choose_madness_target")
end
function card.choose_madness_target(ctx, self)
    if (ctx:get_data(self, "madness_left") or 0) > 0 then
        local candidates = characters(ctx)
        if #candidates > 0 then ctx:random_entity(candidates, "deal_madness_damage") end
    end
end
function card.deal_madness_damage(ctx, self, target)
    local left = ctx:get_data(self, "madness_left") or 0
    if left <= 0 then return end
    ctx:set_data(self, "madness_left", left - 1)
    ctx:damage_ignoring_spell_damage(target, 1)
    if left > 1 then ctx:continue_with("choose_madness_target") end
end
return card
