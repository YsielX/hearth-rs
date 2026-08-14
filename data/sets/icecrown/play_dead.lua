local function has_deathrattle(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "deathrattle" then return true end
    end
    return false
end

return {
    api_version = 1, id = "ICC_052", name = "Play Dead",
    text = "Trigger a friendly minion's <b>Deathrattle</b>.",
    set = "ICECROWN", type = "spell", class = "hunter", rarity = "common", cost = 1,
    target_mode = "required", targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if has_deathrattle(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
    on_play = function(ctx, self, target)
        local repetitions = has_deathrattle(ctx, target) and 1 or 0
        for _, keyword in ipairs(ctx:entity(target).keywords) do
            if keyword == "deathrattle_repeater" then repetitions = 2 break end
        end
        for _ = 1, repetitions do
            ctx:trigger_hook(target, "on_deathrattle", ctx:board_position(target))
        end
    end,
}
