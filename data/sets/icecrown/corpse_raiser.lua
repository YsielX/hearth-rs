local card = {
    api_version = 1,
    id = "ICC_257",
    name = "Corpse Raiser",
    text = "[x]<b>Battlecry:</b> Give a friendly\nminion \"<b>Deathrattle:</b>\n  Resummon this minion.\"",
    set = "ICECROWN",
    type = "minion",
    rarity = "rare",
    cost = 5,
    attack = 3,
    health = 3,
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
        ctx:attach_deathrattle(target, "ICC_257")
        ctx:grant_keyword(target, "deathrattle")
    end
end

function card.on_deathrattle(ctx, self, position)
    ctx:summon_at(ctx:controller(self), ctx:entity(self).card_id, position)
end

return card
