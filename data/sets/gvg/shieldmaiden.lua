return {
    api_version = 1,
    id = "GVG_053",
    name = "Shieldmaiden",
    text = "<b>Battlecry:</b> Gain 5 Armor.",
    set = "GVG",
    type = "minion",
    class = "warrior",
    rarity = "rare",
    cost = 5,
    attack = 5,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:gain_armor(ctx:controller(self), 5)
    end,
}
