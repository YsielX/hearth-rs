return {
    api_version = 1,
    id = "OG_316",
    name = "Herald Volazj",
    text = "<b>Battlecry:</b> Summon a 1/1 copy of each of your other minions.",
    set = "OG",
    type = "minion",
    class = "priest",
    rarity = "legendary",
    cost = 6,
    attack = 5,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        local originals = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            local dormant = false
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if minion ~= self and not dormant then originals[#originals + 1] = minion end
        end
        for _, minion in ipairs(originals) do
            ctx:summon_copy_with_stats(player, minion, 1, 1)
        end
    end,
}
