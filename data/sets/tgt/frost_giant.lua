return {
    api_version = 1, id = "AT_120", name = "Frost Giant",
    text = "Costs (1) less for each time you used your Hero Power this game.", set = "TGT",
    type = "minion", rarity = "epic", cost = 10, attack = 8, health = 8,
    auras = {{
        active_zones = { "hand" }, targets = function(ctx, self) return { self } end,
        cost = function(ctx, self) return -ctx:hero_power_uses(ctx:controller(self)) end,
    }},
}
