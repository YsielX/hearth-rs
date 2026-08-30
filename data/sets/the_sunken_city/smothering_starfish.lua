return {
    api_version = 1,
    id = "TSC_926", rarity = "epic",
    name = "Smothering Starfish",
    text = "<b>Battlecry:</b> <b>Silence</b> ALL other minions.",
    set = "THE_SUNKEN_CITY",
    type = "minion",
    cost = 3,
    attack = 2,
    health = 4,
    tags = { "beast" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:minions()) do
            if minion ~= self then ctx:silence(minion) end
        end
    end,
}
