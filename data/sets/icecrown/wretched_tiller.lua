return {
    api_version = 1,
    id = "ICC_468",
    name = "Wretched Tiller",
    text = "Whenever this minion attacks, deal 2 damage to the enemy hero.",
    set = "ICECROWN",
    type = "minion",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 1,
    tags = { "undead" },
    triggers = {
        {
            event = "attack",
            timing = "before",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.attacker == self
            end,
            effect = function(ctx, self)
                local enemy = ctx:opponent(ctx:controller(self))
                cardlib.effects.damage(ctx, ctx:player(enemy).hero, 2)
            end,
        },
    },
}
