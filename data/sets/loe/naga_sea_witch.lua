return {
    api_version = 1, id = "LOE_038", name = "Naga Sea Witch",
    text = "Your cards cost (5).", set = "LOE", type = "minion", rarity = "epic",
    cost = 8, attack = 5, health = 5, tags = { "naga" },
    auras = {{
        cost_set = 5,
        targets = function(ctx, self)
            local player = ctx:controller(self)
            local result = {}
            for _, entity in ipairs(ctx:hand(player)) do result[#result + 1] = entity end
            for _, entity in ipairs(ctx:deck(player)) do result[#result + 1] = entity end
            return result
        end,
    }},
}
