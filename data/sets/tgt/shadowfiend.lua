return {
    api_version = 1,
    id = "AT_014",
    name = "Shadowfiend",
    text = "Whenever you draw a card, reduce its Cost by (1).",
    set = "TGT",
    type = "minion",
    class = "priest",
    rarity = "epic",
    cost = 2,
    attack = 2,
    health = 3,
    triggers = {
        {
            event = "card_drawn",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                ctx:modify(event.entity, { stat = "cost", operation = "add", value = -1 })
            end,
        },
    },
}
