local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "AT_016",
    name = "Confuse",
    text = "Swap the Attack and Health of all minions.",
    set = "TGT",
    type = "spell",
    class = "priest",
    rarity = "epic",
    spell_school = "shadow",
    cost = 2,
    on_play = function(ctx)
        local swaps = {}
        for _, minion in ipairs(ctx:minions()) do
            local entity = ctx:entity(minion)
            if not is_dormant(ctx, minion) then
                swaps[#swaps + 1] = { id = minion, attack = entity.attack, health = entity.health }
            end
        end
        for _, swap in ipairs(swaps) do
            ctx:modify(swap.id, { stat = "attack", operation = "set", value = swap.health })
            ctx:set_health(swap.id, swap.attack)
        end
    end,
}
