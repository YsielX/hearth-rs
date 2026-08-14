return {
    api_version = 1,
    id = "GVG_100",
    name = "Floating Watcher",
    text = "Whenever your hero takes damage on your turn, gain +2/+2.",
    set = "GVG",
    type = "minion",
    class = "warlock",
    rarity = "common",
    cost = 5,
    attack = 4,
    health = 4,
    tags = { "demon" },
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                local player = ctx:controller(self)
                return ctx:active_player() == player
                    and event.target == ctx:player(player).hero
                    and event.amount > 0
            end,
            effect = function(ctx, self)
                ctx:buff(self, 2, 2)
            end,
        },
    },
}
