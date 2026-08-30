local function is_murloc(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "murloc" or tag == "all" then return true end
    end
    return false
end
return {
    api_version = 1, id = "UNG_073", name = "Rockpool Hunter",
    text = "<b>Battlecry:</b> Give a friendly Murloc +1/+1.", set = "UNGORO",
    type = "minion", rarity = "common", cost = 2, attack = 2, health = 3,
    tags = { "murloc" }, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self and is_murloc(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target) if target then cardlib.effects.buff(ctx, target, 1, 1) end end,
}
