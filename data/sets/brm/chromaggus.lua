return {
    api_version = 1,
    id = "BRM_031",
    name = "Chromaggus",
    text = "Whenever you draw a card, put another copy into your hand.",
    set = "BRM",
    type = "minion",
    rarity = "legendary",
    cost = 8,
    attack = 6,
    health = 8,
    tags = { "dragon" },
    triggers = {
        {
            event = "card_drawn",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                ctx:give_card(
                    ctx:controller(self),
                    ctx:entity(event.entity).card_id
                )
            end,
        },
    },
}
