local function beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "beast" then return true end
    end
    return false
end
return {
    api_version = 1, id = "GVG_120", name = "Hemet Nesingwary",
    text = "<b>Battlecry:</b> Destroy a Beast.", set = "GVG", type = "minion",
    rarity = "legendary", cost = 5, attack = 6, health = 3,
    keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:minions()) do if beast(ctx, minion) then result[#result + 1] = minion end end
        return result
    end,
    on_battlecry = function(ctx, self, target) if target ~= nil then cardlib.effects.destroy(ctx, target) end end,
}
