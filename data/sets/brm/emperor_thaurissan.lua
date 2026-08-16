return {
    api_version = 1,
    id = "BRM_028",
    name = "Emperor Thaurissan",
    text = "At the end of your turn, reduce the Cost of cards\nin your hand by (1).",
    set = "BRM",
    type = "minion",
    rarity = "legendary",
    cost = 5,
    attack = 4,
    health = 4,
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                for _, card in ipairs(ctx:hand(ctx:controller(self))) do
                    cardlib.effects.modify(ctx, card, {
                        stat = "cost",
                        operation = "add",
                        value = -1,
                    })
                end
            end,
        },
    },
}
