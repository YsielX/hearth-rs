return {
    api_version = 1, id = "ICC_083", name = "Doomed Apprentice",
    text = "Your opponent's spells cost (1) more.",
    set = "ICECROWN", type = "minion", class = "mage", rarity = "rare",
    cost = 3, attack = 3, health = 2, tags = { "undead" },
    auras = {{
        active_zones = { "board" }, cost = 1,
        targets = function(ctx, self)
            local result = {}
            local opponent = ctx:opponent(ctx:controller(self))
            for _, zone in ipairs({ ctx:hand(opponent), ctx:deck(opponent) }) do
                for _, entity in ipairs(zone) do
                    if ctx:entity(entity).type == "spell" then result[#result + 1] = entity end
                end
            end
            return result
        end,
    }},
}
