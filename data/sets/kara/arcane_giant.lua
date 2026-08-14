return {
    api_version = 1,
    id = "KAR_711",
    name = "Arcane Giant",
    text = "[x]Costs (1) less for each spell\nyou've cast this game.",
    set = "KARA",
    type = "minion",
    rarity = "epic",
    cost = 12,
    attack = 8,
    health = 8,
    auras = {{
        active_zones = { "hand" },
        targets = function(ctx, self) return { self } end,
        cost = function(ctx, self)
            return -#ctx:spells_cast(ctx:controller(self))
        end,
    }},
}
