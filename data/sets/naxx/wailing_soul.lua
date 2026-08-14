return {
    api_version = 1,
    id = "FP1_016",
    name = "Wailing Soul",
    text = "<b>Battlecry: Silence</b> your other minions.",
    set = "NAXX",
    type = "minion",
    rarity = "rare",
    cost = 4,
    attack = 3,
    health = 5,
    tags = { "undead" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self then ctx:silence(minion) end
        end
    end,
}
