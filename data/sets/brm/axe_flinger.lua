return {
    api_version = 1,
    id = "BRM_016",
    name = "Axe Flinger",
    text = "Whenever this minion takes damage, deal 2 damage to the enemy hero.",
    set = "BRM",
    type = "minion",
    class = "warrior",
    rarity = "common",
    cost = 4,
    attack = 2,
    health = 6,
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > 0
            end,
            effect = function(ctx, self)
                local enemy = ctx:opponent(ctx:controller(self))
                ctx:damage(ctx:player(enemy).hero, 2)
            end,
        },
    },
}
