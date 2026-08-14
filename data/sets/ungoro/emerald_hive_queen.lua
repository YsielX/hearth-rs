return {
    api_version = 1, id = "UNG_085", name = "Emerald Hive Queen",
    text = "Your minions cost (2) more.", set = "UNGORO", type = "minion",
    rarity = "epic", cost = 1, attack = 3, health = 3, tags = { "beast" },
    auras = {{ active_zones = { "board" }, cost = 2,
        targets = function(ctx, self)
            local result = {}
            for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
            end
            return result
        end }},
}
