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
    id = "EX1_581",
    name = "Sap",
    text = "Return an enemy minion to your opponent's hand.",
    set = "LEGACY",
    type = "spell",
    class = "rogue",
    cost = 2,
    target_mode = "required",
    targets = enemy_minions,
    on_play = function(ctx, self, target)
        ctx:move(target, "hand")
    end,
}
