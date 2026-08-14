local card = {
    api_version = 1,
    id = "BRM_022",
    name = "Dragon Egg",
    text = "Whenever this minion takes damage, summon a 2/1 Whelp.",
    set = "BRM",
    type = "minion",
    rarity = "rare",
    cost = 1,
    attack = 0,
    health = 2,
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > 0
            end,
            effect = function(ctx, self)
                ctx:summon(ctx:controller(self), "BRM_022t")
            end,
        },
    },
}

card.tokens = {
    {
        id = "BRM_022t",
        name = "Black Whelp",
        text = "",
        set = "BRM",
        type = "minion",
        cost = 1,
        attack = 2,
        health = 1,
        tags = { "dragon" },
    },
}

return card
