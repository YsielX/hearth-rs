return {
    api_version = 1,
    id = "LOE_118",
    name = "Cursed Blade",
    text = "Double all damage dealt to your hero.",
    set = "LOE",
    type = "weapon",
    class = "warrior",
    rarity = "rare",
    cost = 1,
    attack = 2,
    health = 3,
    triggers = {
        {
            event = "damaged",
            timing = "before",
            active_zones = { "weapon" },
            condition = function(ctx, self, event)
                local player = ctx:controller(self)
                return event.target == ctx:player(player).hero and event.amount > 0
            end,
            effect = function(ctx, self, event)
                cardlib.effects.set_event_amount(ctx, event, event.amount * 2)
            end,
        },
    },
}
