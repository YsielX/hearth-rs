local function has_battlecry(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "battlecry" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "LOE_077",
    name = "Brann Bronzebeard",
    text = "Your <b>Battlecries</b> trigger twice.",
    set = "LOE",
    type = "minion",
    rarity = "legendary",
    cost = 3,
    attack = 2,
    health = 4,
    auras = {{
        keywords = { "battlecry_repeater" },
        targets = function(ctx, self)
            local result = {}
            for _, minion in ipairs(ctx:friendly_minions(self)) do
                if has_battlecry(ctx, minion) then result[#result + 1] = minion end
            end
            return result
        end,
    }},
}
