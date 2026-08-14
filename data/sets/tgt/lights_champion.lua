local function is_demon(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "demon" or tag == "all" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "AT_106",
    name = "Light's Champion",
    text = "<b>Battlecry:</b> <b>Silence</b> a Demon.",
    set = "TGT",
    type = "minion",
    rarity = "rare",
    cost = 3,
    attack = 4,
    health = 3,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx)
        local result = {}
        for _, minion in ipairs(ctx:minions()) do
            if is_demon(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:silence(target) end
    end,
}
