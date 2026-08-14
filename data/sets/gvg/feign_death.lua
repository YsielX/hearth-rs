local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == wanted then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_026",
    name = "Feign Death",
    text = "Trigger all <b>Deathrattles</b> on your minions.",
    set = "GVG",
    type = "spell",
    class = "hunter",
    rarity = "epic",
    cost = 2,

    on_play = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if has_keyword(ctx, minion, "deathrattle") then
                local repetitions = has_keyword(ctx, minion, "deathrattle_repeater") and 2 or 1
                for _ = 1, repetitions do
                    ctx:trigger_hook(minion, "on_deathrattle", ctx:board_position(minion))
                end
            end
        end
    end,
}
