local card = {
    api_version = 1, id = "OG_102", name = "Darkspeaker",
    text = "<b>Battlecry:</b> Swap stats with a friendly minion.", set = "OG",
    type = "minion", rarity = "epic", cost = 5, attack = 3, health = 6,
    keywords = { "battlecry" }, target_mode = "required_if_available",
}
function card.targets(ctx, self)
    local result = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self then result[#result + 1] = minion end
    end
    return result
end
function card.on_battlecry(ctx, self, target)
    if not target then return end
    local own, other = ctx:entity(self), ctx:entity(target)
    cardlib.effects.modify(ctx, self, { stat = "attack", operation = "set", value = other.attack })
    ctx:set_health(self, other.health)
    cardlib.effects.modify(ctx, target, { stat = "attack", operation = "set", value = own.attack })
    ctx:set_health(target, own.health)
end
return card
