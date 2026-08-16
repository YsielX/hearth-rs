local card = {
    api_version = 1, id = "AT_094", name = "Flame Juggler",
    text = "<b>Battlecry:</b> Deal 1 damage to a random enemy.",
    set = "TGT", type = "minion", rarity = "common", cost = 2, attack = 2, health = 3,
    tags = { "draenei" }, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:enemy_characters(self)) do
        local is_dormant = false
        for _, keyword in ipairs(ctx:entity(entity).keywords) do
            if keyword == "dormant" then is_dormant = true break end
        end
        if not is_dormant then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "deal_random_damage") end
end

function card.deal_random_damage(ctx, self, target) cardlib.effects.damage(ctx, target, 1) end

return card
