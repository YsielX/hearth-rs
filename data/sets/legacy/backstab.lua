local function all_minions(ctx)
    return ctx:minions()
end

return {
    api_version = 1,
    id = "CS2_072", rarity = "free",
    name = "Backstab",
    text = "Deal $2 damage to an undamaged minion.",
    set = "LEGACY",
    type = "spell",
    class = "rogue",
    cost = 0,
    target_mode = "required",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(all_minions(ctx)) do
            local minion = ctx:entity(entity)
            if minion.damage == 0 then
                result[#result + 1] = entity
            end
        end
        return result
    end,
    on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 2) end,
}
