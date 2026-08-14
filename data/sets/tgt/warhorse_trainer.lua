return {
    api_version = 1,
    id = "AT_075",
    name = "Warhorse Trainer",
    text = "Your Silver Hand Recruits have +2 Attack and <b>Taunt</b>.",
    set = "TGT",
    type = "minion",
    class = "paladin",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 4,
    auras = {
        {
            active_zones = { "board" },
            attack = 2,
            keywords = { "taunt" },
            targets = function(ctx, self)
                local result = {}
                for _, minion in ipairs(ctx:friendly_minions(self)) do
                    if ctx:entity(minion).card_id == "CS2_101t" then result[#result + 1] = minion end
                end
                return result
            end,
        },
    },
}
