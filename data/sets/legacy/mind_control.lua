local function enemy_minions(ctx, self)
    local result = {}
    local controller = ctx:controller(self)
    for _, minion in ipairs(ctx:minions()) do
        if ctx:controller(minion) ~= controller then
            result[#result + 1] = minion
        end
    end
    return result
end

return {
    api_version = 1,
    id = "CS1_113", rarity = "free",
    name = "Mind Control",
    text = "Take control of an enemy minion.",
    set = "LEGACY",
    type = "spell",
    spell_school = "shadow",
    class = "priest",
    cost = 9,
    target_mode = "required",
    targets = enemy_minions,
    on_play = function(ctx, self, target)
        ctx:change_controller(target, ctx:controller(self))
    end,
}
