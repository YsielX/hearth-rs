return {
    api_version = 1,
    id = "TTN_459",
    name = "Chained Guardian",
    text = "[x]<b><b>Rush</b>, Reborn</b>\nCosts (1) less for each\nPlague shuffled into the\n enemy deck this game.",
    set = "TITANS",
    type = "minion",
    class = "death_knight",
    rarity = "epic",
    cost = 11,
    attack = 8,
    health = 5,
    rune_cost = { unholy = 1 },
    keywords = { "rush", "reborn" },
    auras = {{
        active_zones = { "hand" },
        targets = function(ctx, self) return { self } end,
        cost = function(ctx, self)
            return -ctx:get_player_data(ctx:controller(self), "plagues_shuffled_into_enemy")
        end,
    }},
}
