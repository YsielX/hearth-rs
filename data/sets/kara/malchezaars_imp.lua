return {
    api_version = 1,
    id = "KAR_089",
    name = "Malchezaar's Imp",
    text = "Whenever you discard a card, draw a card.",
    set = "KARA",
    type = "minion",
    class = "warlock",
    rarity = "common",
    cost = 2,
    attack = 1,
    health = 3,
    tags = { "demon" },
    triggers = {{
        event = "card_discarded",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
        end,
        effect = function(ctx, self)
            ctx:draw(ctx:controller(self), 1)
        end,
    }},
}
