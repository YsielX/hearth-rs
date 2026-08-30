local card = {
    api_version = 1,
    id = "ICC_705",
    name = "Bonemare",
    text = "<b>Battlecry:</b> Give a friendly minion +4/+4 and <b>Taunt</b>.",
    set = "ICECROWN",
    type = "minion",
    rarity = "common",
    cost = 7,
    attack = 5,
    health = 5,
    tags = { "undead", "beast" },
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
    if target then
        cardlib.effects.buff(ctx, target, 4, 4)
        cardlib.effects.grant_keyword(ctx, target, "taunt")
    end
end

return card
