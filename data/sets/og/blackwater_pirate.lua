return {
    api_version = 1, id = "OG_322", name = "Blackwater Pirate",
    text = "Your weapons cost (2) less.", set = "OG", type = "minion", rarity = "rare",
    cost = 4, attack = 2, health = 5, tags = { "pirate" }, auras = {{
        cost = -2,
        targets = function(ctx, self)
            local result = {}
            local player = ctx:controller(self)
            for _, zone in ipairs({ ctx:hand(player), ctx:deck(player) }) do
                for _, entity in ipairs(zone) do
                    if ctx:entity(entity).type == "weapon" then result[#result + 1] = entity end
                end
            end
            return result
        end,
    }},
}
