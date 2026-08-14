local function combo_targets(ctx, self)
    local player = ctx:controller(self)
    if ctx:cards_played_this_turn(player) == 0 then
        return {}
    end
    return ctx:characters()
end

return {
    api_version = 1,
    id = "EX1_134",
    name = "SI:7 Agent",
    text = "<b>Combo:</b> Deal 3 damage.",
    set = "EXPERT1",
    type = "minion",
    class = "rogue",
    cost = 3,
    attack = 3,
    health = 3,
    keywords = { "combo" },
    target_mode = "required_if_available",
    targets = combo_targets,

    on_combo = function(ctx, self, target)
        if target ~= nil then
            ctx:damage(target, 3)
        end
    end,
}
