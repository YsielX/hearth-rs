return {
    api_version = 1,
    id = "BRM_019",
    name = "Grim Patron",
    text = "After this minion survives damage, summon another Grim Patron.",
    set = "BRM",
    type = "minion",
    rarity = "rare",
    cost = 5,
    attack = 3,
    health = 3,
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self
                    and event.amount > 0
                    and ctx:entity(self).health > 0
            end,
            effect = function(ctx, self)
                ctx:summon(ctx:controller(self), "BRM_019")
            end,
        },
    },
}
