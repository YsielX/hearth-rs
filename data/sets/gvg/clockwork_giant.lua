return {
    api_version = 1, id = "GVG_121", name = "Clockwork Giant",
    text = "Costs (1) less for each card in your opponent's hand.", set = "GVG",
    type = "minion", rarity = "epic", cost = 12, attack = 8, health = 8, tags = { "mech" },
    auras = {{
        active_zones = { "hand", "deck" },
        cost = function(ctx, self)
            return -#ctx:hand(ctx:opponent(ctx:controller(self)))
        end,
        targets = function(ctx, self) return { self } end,
    }},
}
