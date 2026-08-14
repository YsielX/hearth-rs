return {
    api_version = 1,
    id = "GVG_023",
    name = "Goblin Auto-Barber",
    text = "<b>Battlecry:</b> Give your weapon +1 Attack.",
    set = "GVG",
    type = "minion",
    class = "rogue",
    rarity = "common",
    cost = 2,
    attack = 3,
    health = 2,
    tags = { "mech" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local weapon = ctx:player(ctx:controller(self)).weapon
        if weapon ~= nil then ctx:buff(weapon, 1, 0) end
    end,
}
