return {
    api_version = 1, id = "AT_125", name = "Icehowl",
    text = "<b>Charge</b>\nCan't attack heroes.", set = "TGT", type = "minion",
    rarity = "legendary", cost = 9, attack = 10, health = 10, keywords = { "charge" },
    auras = {{
        active_zones = { "board" }, keywords = { "cannot_be_attacked_by_icehowl" },
        targets = function(ctx, self)
            return { ctx:player(ctx:opponent(ctx:controller(self))).hero }
        end,
    }},
}
