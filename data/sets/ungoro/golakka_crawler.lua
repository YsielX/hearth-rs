local function pirate(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "pirate" or tag == "all" then return true end
    end
    return false
end
return {
    api_version = 1, id = "UNG_807", name = "Golakka Crawler",
    text = "<b>Battlecry:</b> Destroy a Pirate and gain +1/+1.",
    set = "UNGORO", type = "minion", rarity = "rare", cost = 2, attack = 2, health = 3,
    tags = { "beast" }, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:all_characters()) do
            if entity ~= self and ctx:entity(entity).type == "minion" and pirate(ctx, entity) then result[#result + 1] = entity end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target)
        if target then ctx:destroy(target) ctx:buff(self, 1, 1) end
    end,
}
