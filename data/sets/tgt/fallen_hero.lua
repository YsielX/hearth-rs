return {
    api_version = 1, id = "AT_003", name = "Fallen Hero",
    text = "Your Hero Power deals 1 extra damage.", set = "TGT", type = "minion",
    class = "mage", rarity = "rare", cost = 2, attack = 3, health = 2,
    tags = { "undead" },
    triggers = {
        {
            event = "damaged", timing = "before", active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.source == ctx:player(ctx:controller(self)).hero_power
            end,
            effect = function(ctx, self, event)
                ctx:set_event_amount(event, event.amount + 1)
            end,
        },
    },
}
