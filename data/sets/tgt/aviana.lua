return {
    api_version = 1, id = "AT_045", name = "Aviana",
    text = "Your minions cost (1).", set = "TGT", type = "minion",
    class = "druid", rarity = "legendary", cost = 9, attack = 5, health = 5,
    auras = {
        {
            cost_set = 1,
            targets = function(ctx, self)
                local result = {}
                local player = ctx:controller(self)
                for _, entity in ipairs(ctx:hand(player)) do
                    if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
                end
                for _, entity in ipairs(ctx:deck(player)) do
                    if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
                end
                return result
            end,
        },
    },
}
