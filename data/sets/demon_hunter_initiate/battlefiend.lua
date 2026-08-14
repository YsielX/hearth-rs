return {
    api_version = 1,
    id = "BT_351",
    name = "Battlefiend",
    text = "After your hero attacks, gain +1 Attack.",
    set = "DEMON_HUNTER_INITIATE",
    type = "minion",
    class = "demon_hunter",
    cost = 1,
    attack = 1,
    health = 2,
    tags = { "demon" },
    triggers = {
        {
            event = "attack",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.attacker == ctx:player(ctx:controller(self)).hero
            end,
            effect = function(ctx, self, event)
                ctx:buff(self, 1, 0)
            end,
        },
    },
}
