local card = {
    api_version = 1,
    id = "ICC_467",
    name = "Deathspeaker",
    text = "<b>Battlecry:</b> Give a friendly minion <b>Immune</b> this turn.",
    set = "ICECROWN",
    type = "minion",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 4,
    tags = { "undead" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
}

function card.targets(ctx, self)
    local result = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self then result[#result + 1] = minion end
    end
    return result
end

function card.on_battlecry(ctx, self, target)
    if target then ctx:grant_keyword_until_end_of_turn(target, "immune") end
end

return card
