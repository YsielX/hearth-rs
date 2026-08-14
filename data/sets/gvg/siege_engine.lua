return {
    api_version = 1,
    id = "GVG_086",
    name = "Siege Engine",
    text = "Whenever you gain Armor, give this minion +1 Attack.",
    set = "GVG",
    type = "minion",
    class = "warrior",
    rarity = "rare",
    cost = 5,
    attack = 5,
    health = 5,
    tags = { "mech" },
    triggers = {
        {
            event = "armor_gained",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                local hero = ctx:player(ctx:controller(self)).hero
                return event.target == hero and event.amount > 0
            end,
            effect = function(ctx, self)
                ctx:buff(self, 1, 0)
            end,
        },
    },
}
