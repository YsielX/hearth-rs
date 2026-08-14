return {
    api_version = 1, id = "CFM_655", name = "Toxic Sewer Ooze",
    text = "<b>Battlecry:</b> Remove 1 Durability from your opponent's weapon.",
    set = "GANGS", type = "minion", rarity = "common", cost = 3, attack = 4, health = 3,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local weapon = ctx:player(ctx:opponent(ctx:controller(self))).weapon
        if weapon then ctx:lose_weapon_durability(weapon, 1) end
    end,
}
