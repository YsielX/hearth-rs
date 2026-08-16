local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "AT_081",
    name = "Eadric the Pure",
    text = "<b>Battlecry:</b> Change all enemy minions'\nAttack to 1.",
    set = "TGT",
    type = "minion",
    class = "paladin",
    rarity = "legendary",
    cost = 7,
    attack = 3,
    health = 7,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(entity).type == "minion" and not is_dormant(ctx, entity) then
                cardlib.effects.modify(ctx, entity, { stat = "attack", operation = "set", value = 1 })
            end
        end
    end,
}
