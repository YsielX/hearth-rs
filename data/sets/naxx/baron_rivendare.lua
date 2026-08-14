return {
    api_version = 1,
    id = "FP1_031",
    name = "Baron Rivendare",
    text = "Your minions trigger their <b>Deathrattles</b> twice.",
    set = "NAXX",
    type = "minion",
    rarity = "legendary",
    cost = 4,
    attack = 1,
    health = 7,
    tags = { "undead" },
    auras = {
        {
            keywords = { "deathrattle_repeater" },
            targets = function(ctx, self)
                local targets = {}
                for _, minion in ipairs(ctx:friendly_minions(self)) do
                    for _, keyword in ipairs(ctx:entity(minion).keywords) do
                        if keyword == "deathrattle" then
                            targets[#targets + 1] = minion
                            break
                        end
                    end
                end
                return targets
            end,
        },
    },
}
