local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == wanted then return true end
    end
    return false
end

return {
    api_version = 1, id = "ICC_807", name = "Strongshell Scavenger",
    text = "<b>Battlecry:</b> Give your <b>Taunt</b> minions +2/+2.",
    set = "ICECROWN", type = "minion", class = "druid", rarity = "rare",
    cost = 4, attack = 2, health = 3, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if has_keyword(ctx, minion, "taunt") then targets[#targets + 1] = minion end
        end
        cardlib.effects.modify_all(ctx, targets, { attack = 2, health = 2, operation = "add" })
    end,
}
