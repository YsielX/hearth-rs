return {
    api_version = 1, id = "UNG_034", name = "Radiant Elemental",
    text = "Your spells cost (1) less\n<i>(but not less than 1)</i>.",
    set = "UNGORO", type = "minion", class = "priest", rarity = "common",
    cost = 2, attack = 2, health = 3, tags = { "elemental" },
    auras = {{
        active_zones = { "board" }, cost = -1, keywords = { "radiant_elemental_minimum_cost" },
        targets = function(ctx, self)
            local result = {}
            for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                if ctx:entity(entity).type == "spell" then result[#result + 1] = entity end
            end
            return result
        end,
    }},
}
