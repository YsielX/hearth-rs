local card = {
    api_version = 1,
    id = "BRM_006",
    name = "Imp Gang Boss",
    text = "Whenever this minion takes damage, summon a 1/1 Imp.",
    set = "BRM",
    type = "minion",
    class = "warlock",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 4,
    tags = { "demon" },
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > 0
            end,
            effect = function(ctx, self)
                ctx:summon(ctx:controller(self), "BRM_006t")
            end,
        },
    },
}

card.tokens = {
    {
        id = "BRM_006t",
        name = "Imp",
        text = "",
        set = "BRM",
        type = "minion",
        class = "warlock",
        cost = 1,
        attack = 1,
        health = 1,
        tags = { "demon" },
    },
}

return card
