return {
    api_version = 1, id = "CFM_651", name = "Naga Corsair",
    text = "<b>Battlecry:</b> Give your weapon +1 Attack.", set = "GANGS", type = "minion", rarity = "common",
    cost = 4, attack = 5, health = 4, tags = { "naga", "pirate" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local weapon = ctx:player(ctx:controller(self)).weapon
        if weapon then ctx:buff(weapon, 1, 0) end
    end,
}
